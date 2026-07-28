//! User access tokens and the Bearer extractor.
//!
//! Doc 6.2: 15-minute JWT, no refresh token. Doc 8.2: the control WSS validates
//! this token once at handshake and never again for the life of the connection,
//! so a session may outlive the token - see doc 13.9 for that trade-off.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// User id.
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
}

pub fn issue_access_token(secret: &[u8], user_id: Uuid, ttl_secs: i64) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now,
        exp: now + ttl_secs,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| AppError::internal(format!("token signing failed: {e}")))
}

pub fn verify_access_token(secret: &[u8], token: &str) -> Result<Uuid, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    // No leeway: the token lifetime is already generous relative to clock skew
    // on a single node, and accepting expired tokens silently widens the window
    // doc 6.2 deliberately keeps narrow.
    validation.leeway = 0;

    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map_err(|_| AppError::unauthorized())?;

    Uuid::parse_str(&data.claims.sub).map_err(|_| AppError::unauthorized())
}

/// Authenticated user, extracted from `Authorization: Bearer <jwt>`.
pub struct AuthUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    crate::api::AppState: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum::extract::FromRef;
        let app_state = crate::api::AppState::from_ref(state);

        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(AppError::unauthorized)?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(AppError::unauthorized)?
            .trim();

        let user_id = verify_access_token(&app_state.config.jwt_secret, token)?;
        Ok(AuthUser { user_id })
    }
}

/// Client address for rate limiting.
///
/// Doc 6.1 puts the relay behind a reverse proxy on the same controlled host, so
/// `X-Forwarded-For` is trusted here. That is only safe because nothing else can
/// reach the port; if the relay is ever exposed directly this must change, since
/// the header is attacker-controlled and would let one client spread its quota
/// across unlimited synthetic addresses.
pub struct ClientIp(pub String);

impl<S> FromRequestParts<S> for ClientIp
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(forwarded) = parts
            .headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            return Ok(ClientIp(forwarded.to_string()));
        }

        let addr = parts
            .extensions
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.0.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(ClientIp(addr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"jwt-secret-at-least-32-bytes-long!!";

    #[test]
    fn token_round_trip() {
        let user_id = Uuid::new_v4();
        let token = issue_access_token(SECRET, user_id, 900).unwrap();
        assert_eq!(verify_access_token(SECRET, &token).unwrap(), user_id);
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let token = issue_access_token(SECRET, Uuid::new_v4(), 900).unwrap();
        let other = b"another-secret-at-least-32-bytes!!!!";
        assert!(verify_access_token(other, &token).is_err());
    }

    #[test]
    fn expired_token_is_rejected() {
        let token = issue_access_token(SECRET, Uuid::new_v4(), -1).unwrap();
        assert!(verify_access_token(SECRET, &token).is_err());
    }

    #[test]
    fn garbage_is_rejected() {
        assert!(verify_access_token(SECRET, "").is_err());
        assert!(verify_access_token(SECRET, "not.a.jwt").is_err());
    }
}
