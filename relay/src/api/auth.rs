//! POST /v1/auth/login

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::auth::{issue_access_token, ClientIp};
use crate::crypto;
use crate::error::AppError;
use crate::ratelimit::limits;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "tokenType")]
    pub token_type: &'static str,
    #[serde(rename = "expiresIn")]
    pub expires_in: i64,
    pub user: UserSummary,
}

#[derive(Serialize)]
pub struct UserSummary {
    pub id: String,
    pub login: String,
}

pub async fn login(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // Doc 6.5 keys this limit on login + source IP, so one attacker cannot lock
    // a victim out by hammering their account from elsewhere, and one address
    // cannot spray many accounts.
    state
        .limiter
        .check(&format!("login:{}:{ip}", body.login), limits::LOGIN)?;

    if body.login.is_empty() || body.login.chars().count() > 254 {
        return Err(AppError::bad_request("login must be 1..254 characters"));
    }
    if body.password.len() < 8 || body.password.len() > 1024 {
        return Err(AppError::bad_request("password must be 8..1024 bytes"));
    }

    let user = state.db.find_user_by_login(&body.login).await?;

    // Verify a dummy hash when the user does not exist so a missing account and
    // a wrong password take comparable time; otherwise the response latency
    // enumerates valid logins.
    let Some(user) = user else {
        let _ = crypto::verify_password(DUMMY_PHC, &body.password);
        return Err(AppError::invalid_credentials());
    };

    if !crypto::verify_password(&user.password_digest, &body.password) {
        return Err(AppError::invalid_credentials());
    }

    let token = issue_access_token(
        &state.config.jwt_secret,
        user.id,
        state.config.access_token_ttl_secs,
    )?;

    tracing::info!(user_id = %user.id, "login succeeded");

    Ok(Json(LoginResponse {
        access_token: token,
        token_type: "Bearer",
        expires_in: state.config.access_token_ttl_secs,
        user: UserSummary {
            id: user.id.to_string(),
            login: user.login,
        },
    }))
}

/// A real Argon2id hash of a value nobody knows, used only to burn comparable
/// CPU time on the unknown-user path.
const DUMMY_PHC: &str = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHRzb21lc2FsdA$\
    XOaVQKHhE8SsxKh0FGr4uw0GRQC4rQOsxwUOZ7C6cGs";
