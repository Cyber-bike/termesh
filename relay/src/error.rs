//! Error types and their wire mapping.
//!
//! Doc 6.2 fixes the HTTP status mapping and doc 12 fixes which error code each
//! failure carries; both are encoded here so no handler invents its own.
//!
//! Messages are written for the operator reading a log or the user reading a
//! toast. They never include a token, a pairing code, a password or file
//! content (doc 13.2).

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// The error codes from doc 12 that the HTTPS surface can produce. The WSS-only
/// codes live with the gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    AuthExpired,
    PairingCodeInvalid,
    DeviceForbidden,
    QuotaExceeded,
    RateLimited,
    ProtocolError,
    Internal,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AuthExpired => "AUTH_EXPIRED",
            Self::PairingCodeInvalid => "PAIRING_CODE_INVALID",
            Self::DeviceForbidden => "DEVICE_FORBIDDEN",
            Self::QuotaExceeded => "QUOTA_EXCEEDED",
            Self::RateLimited => "RATE_LIMITED",
            Self::ProtocolError => "PROTOCOL_ERROR",
            // Doc 12 has no code for a server fault; 500 responses carry this
            // one so the body shape stays uniform.
            Self::Internal => "INTERNAL_ERROR",
        }
    }
}

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub code: ErrorCode,
    pub message: String,
    /// Seconds, only set on 429 (doc 6.2 requires the header).
    pub retry_after: Option<u32>,
    /// Detail for the server log only; never serialised to the client.
    pub internal: Option<String>,
}

impl AppError {
    fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            retry_after: None,
            internal: None,
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::ProtocolError, message)
    }

    /// Wrong credentials, missing or expired token. Deliberately one variant:
    /// distinguishing "no such user" from "wrong password" would enumerate accounts.
    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            ErrorCode::AuthExpired,
            "Authentication failed or the access token has expired",
        )
    }

    pub fn forbidden() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            ErrorCode::DeviceForbidden,
            "The object does not belong to the authenticated account",
        )
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            ErrorCode::PairingCodeInvalid,
            message,
        )
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, ErrorCode::QuotaExceeded, message)
    }

    pub fn rate_limited(retry_after: u32) -> Self {
        let mut err = Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::RateLimited,
            "Too many requests",
        );
        err.retry_after = Some(retry_after);
        err
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        let mut err = Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "Internal server error",
        );
        err.internal = Some(detail.into());
        err
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::internal(format!("database error: {err}"))
    }
}

/// Needed so the operator CLI can propagate these with `?` into
/// `Box<dyn std::error::Error>`.
///
/// The internal detail is included here on purpose: this rendering only reaches
/// an operator's terminal or the server log, never an HTTP response. The client
/// body is built in `into_response`, which uses `message` alone.
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.internal {
            Some(detail) => write!(f, "{} ({detail})", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for AppError {}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorPayload,
}

#[derive(Serialize)]
struct ErrorPayload {
    code: &'static str,
    message: String,
    #[serde(rename = "requestId")]
    request_id: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let request_id = uuid::Uuid::new_v4().to_string();

        if let Some(detail) = &self.internal {
            tracing::error!(request_id = %request_id, code = self.code.as_str(), "{detail}");
        } else if self.status.is_client_error() {
            tracing::debug!(request_id = %request_id, code = self.code.as_str(), "{}", self.message);
        }

        let body = Json(ErrorBody {
            error: ErrorPayload {
                code: self.code.as_str(),
                message: self.message,
                request_id,
            },
        });

        let mut response = (self.status, body).into_response();
        if let Some(secs) = self.retry_after {
            if let Ok(value) = secs.to_string().parse() {
                response
                    .headers_mut()
                    .insert(axum::http::header::RETRY_AFTER, value);
            }
        }
        response
    }
}
