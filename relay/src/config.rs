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
    /// Doc 6.2: user access tokens live 15 minutes and there is no refresh token.
    pub access_token_ttl_secs: i64,
}

impl Config {
    pub fn from_env() -> Result<Self, StartupError> {
        let bind = env_or("TERMY_BIND", "0.0.0.0:8080")
            .parse::<SocketAddr>()
            .map_err(|e| StartupError::Config(format!("TERMY_BIND is not a socket address: {e}")))?;

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

        Ok(Self {
            bind,
            database_path,
            pepper,
            jwt_secret,
            relay_url,
            access_token_ttl_secs: 900,
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
    let engines: [&dyn Fn(&str) -> Result<Vec<u8>, base64::DecodeError>; 2] = [
        &|s| base64::engine::general_purpose::STANDARD_NO_PAD.decode(s.trim_end_matches('=')),
        &|s| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(s.trim_end_matches('=')),
    ];

    let mut decoded = None;
    for engine in engines {
        if let Ok(bytes) = engine(&raw) {
            decoded = Some(bytes);
            break;
        }
    }

    let bytes = decoded
        .ok_or_else(|| StartupError::Config(format!("{key} is not valid base64")))?;

    if bytes.len() < MIN_SECRET_BYTES {
        return Err(StartupError::Config(format!(
            "{key} decodes to {} bytes, at least {MIN_SECRET_BYTES} are required",
            bytes.len()
        )));
    }

    Ok(bytes)
}
