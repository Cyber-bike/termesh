//! Runtime configuration, read from the environment.
//!
//! Secrets never come from CLI arguments (doc 7.3 states the rule for the agent;
//! the same applies here, argv is world-readable via /proc).

use std::net::SocketAddr;
use std::path::PathBuf;

use base64::Engine as _;

use crate::error::StartupError;

/// Minimum accepted length for the pepper and the JWT secret. 32 bytes matches
/// the HMAC-SHA-256 block security level; anything shorter is a misconfiguration
/// rather than a weak-but-workable choice, so it is rejected outright.
const MIN_SECRET_BYTES: usize = 32;

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_path: PathBuf,
    /// Server pepper for HMAC-SHA-256 digests of pairing codes and device tokens.
    pub pepper: Vec<u8>,
    /// Signing key for user access tokens.
    pub jwt_secret: Vec<u8>,
    /// Advertised to agents at registration time, e.g. wss://relay.example.com/v1/agent/ws
    pub relay_url: String,
    /// Doc 6.2 sets this to 15 minutes with no refresh token. That is unusable
    /// for an operator whose client stores no credentials: the password has to
    /// be retyped every quarter hour. Deployments may raise it - the tradeoff
    /// is that a stolen token stays valid for longer - so the value is read
    /// from the environment and only the default follows the document.
    pub access_token_ttl_secs: i64,
}

impl Config {
    pub fn from_env() -> Result<Self, StartupError> {
        let bind = env_or("TERMY_BIND", "0.0.0.0:8080")
            .parse::<SocketAddr>()
            .map_err(|e| {
                StartupError::Config(format!("TERMY_BIND is not a socket address: {e}"))
            })?;

        let database_path = PathBuf::from(env_or("TERMY_DB_PATH", "/var/lib/termy-relay/relay.db"));

        let pepper = decode_secret("TERMY_PEPPER")?;
        let jwt_secret = decode_secret("TERMY_JWT_SECRET")?;

        let relay_url = require("TERMY_RELAY_URL")?;
        if !relay_url.starts_with("wss://") {
            return Err(StartupError::Config(
                "TERMY_RELAY_URL must start with wss:// - agents verify certificates against \
                 webpki-roots and will not connect over ws://"
                    .into(),
            ));
        }

        let access_token_ttl_secs =
            parse_access_token_ttl(&env_or("TERMY_ACCESS_TOKEN_TTL_SECS", "900"))?;

        Ok(Self {
            bind,
            database_path,
            pepper,
            jwt_secret,
            relay_url,
            access_token_ttl_secs,
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn require(key: &str) -> Result<String, StartupError> {
    std::env::var(key).map_err(|_| StartupError::Config(format!("{key} is required")))
}

/// Secrets are supplied base64 (standard or URL-safe, padded or not) so that raw
/// bytes survive environment transport intact.
fn decode_secret(key: &str) -> Result<Vec<u8>, StartupError> {
    let raw = require(key)?;
    let trimmed = raw.trim_end_matches('=');

    let bytes = base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(trimmed)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(trimmed))
        .map_err(|_| StartupError::Config(format!("{key} is not valid base64")))?;

    if bytes.len() < MIN_SECRET_BYTES {
        return Err(StartupError::Config(format!(
            "{key} decodes to {} bytes, at least {MIN_SECRET_BYTES} are required",
            bytes.len()
        )));
    }

    Ok(bytes)
}

/// The lower bound keeps a typo from issuing tokens that expire before they can
/// be used; the upper bound is a day, past which a token stops being a
/// short-lived access credential and becomes a second password.
fn parse_access_token_ttl(raw: &str) -> Result<i64, StartupError> {
    let secs = raw.trim().parse::<i64>().map_err(|e| {
        StartupError::Config(format!("TERMY_ACCESS_TOKEN_TTL_SECS is not a number: {e}"))
    })?;
    if !(60..=86_400).contains(&secs) {
        return Err(StartupError::Config(format!(
            "TERMY_ACCESS_TOKEN_TTL_SECS must be between 60 and 86400, got {secs}"
        )));
    }
    Ok(secs)
}

#[cfg(test)]
mod tests {
    use super::parse_access_token_ttl;

    #[test]
    fn the_documented_default_is_accepted() {
        assert_eq!(parse_access_token_ttl("900").unwrap(), 900);
    }

    #[test]
    fn twelve_hours_is_accepted() {
        assert_eq!(parse_access_token_ttl("43200").unwrap(), 43_200);
    }

    #[test]
    fn surrounding_whitespace_is_tolerated() {
        assert_eq!(parse_access_token_ttl(" 43200\n").unwrap(), 43_200);
    }

    #[test]
    fn a_ttl_shorter_than_a_minute_is_rejected() {
        assert!(parse_access_token_ttl("30").is_err());
    }

    #[test]
    fn a_ttl_longer_than_a_day_is_rejected() {
        assert!(parse_access_token_ttl("86401").is_err());
    }

    #[test]
    fn a_non_numeric_value_is_rejected() {
        assert!(parse_access_token_ttl("12h").is_err());
    }
}
