//! Pairing codes and device management, per doc 6.2 and 6.3.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::{AuthUser, ClientIp};
use crate::crypto;
use crate::db::MAX_UNCONSUMED_PAIRING_CODES;
use crate::error::AppError;
use crate::ratelimit::limits;

// --- POST /v1/devices/pairing-codes -----------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatePairingCodeRequest {}

#[derive(Serialize)]
pub struct PairingCodeCreated {
    #[serde(rename = "pairingCodeId")]
    pub pairing_code_id: String,
    /// Doc 6.3.5: returned exactly once, never stored in plaintext.
    #[serde(rename = "pairingCode")]
    pub pairing_code: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub revoked: bool,
}

pub async fn create_pairing_code(
    State(state): State<AppState>,
    user: AuthUser,
    Json(_body): Json<CreatePairingCodeRequest>,
) -> Result<(StatusCode, Json<PairingCodeCreated>), AppError> {
    state.limiter.check(
        &format!("pairing-create:{}", user.user_id),
        limits::CREATE_PAIRING_CODE,
    )?;

    let outstanding = state
        .db
        .count_unconsumed_pairing_codes(user.user_id)
        .await?;
    if outstanding >= MAX_UNCONSUMED_PAIRING_CODES {
        return Err(AppError::conflict(format!(
            "Account already has the maximum of {MAX_UNCONSUMED_PAIRING_CODES} unconsumed pairing codes"
        )));
    }

    let code = crypto::generate_pairing_code();
    let digest = crypto::digest_secret(&state.config.pepper, &code);
    let (id, created_at) = state.db.create_pairing_code(user.user_id, &digest).await?;

    // The code itself is never logged (doc 6.3.7); its id is enough to trace.
    tracing::info!(user_id = %user.user_id, pairing_code_id = %id, "pairing code created");

    Ok((
        StatusCode::CREATED,
        Json(PairingCodeCreated {
            pairing_code_id: id.to_string(),
            pairing_code: code,
            created_at: created_at.to_rfc3339(),
            revoked: false,
        }),
    ))
}

// --- DELETE /v1/devices/pairing-codes/{id} ----------------------------------

pub async fn revoke_pairing_code(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if state.db.revoke_pairing_code(user.user_id, id).await? {
        tracing::info!(user_id = %user.user_id, pairing_code_id = %id, "pairing code revoked");
        Ok(StatusCode::NO_CONTENT)
    } else {
        // Another account's code is reported as missing rather than forbidden so
        // the response does not confirm that the id exists.
        Err(AppError::not_found("Pairing code not found"))
    }
}

// --- POST /v1/devices/register ----------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterRequest {
    #[serde(rename = "pairingCode")]
    pub pairing_code: String,
    #[serde(rename = "deviceName")]
    pub device_name: String,
    pub platform: String,
    #[serde(rename = "agentVersion")]
    pub agent_version: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "deviceToken")]
    pub device_token: String,
    #[serde(rename = "relayUrl")]
    pub relay_url: String,
}

pub async fn register_device(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), AppError> {
    // Two independent limits (doc 6.5): by address, to blunt a spray from one
    // host, and by code digest, to blunt distributed guessing at one code.
    state
        .limiter
        .check(&format!("register-ip:{ip}"), limits::REGISTER_BY_IP)?;

    validate_register(&body)?;

    let code_digest = crypto::digest_secret(&state.config.pepper, &body.pairing_code);
    let code_key = hex_key(&code_digest);
    state.limiter.check(
        &format!("register-code:{code_key}"),
        limits::REGISTER_BY_CODE,
    )?;

    let token = crypto::generate_device_token();
    let token_digest = crypto::digest_secret(&state.config.pepper, &token);

    let (device_id, user_id) = state
        .db
        .consume_pairing_code_and_create_device(
            &code_digest,
            &body.device_name,
            &body.platform,
            &body.agent_version,
            &token_digest,
        )
        .await?;

    tracing::info!(
        user_id = %user_id,
        device_id = %device_id,
        platform = %body.platform,
        "device registered"
    );

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            device_id: device_id.to_string(),
            device_token: token,
            relay_url: state.config.relay_url.clone(),
        }),
    ))
}

fn validate_register(body: &RegisterRequest) -> Result<(), AppError> {
    if body.device_name.is_empty() || body.device_name.chars().count() > 64 {
        return Err(AppError::bad_request("deviceName must be 1..64 characters"));
    }
    if body.device_name.chars().any(char::is_control) {
        return Err(AppError::bad_request(
            "deviceName must not contain control characters",
        ));
    }
    if !matches!(body.platform.as_str(), "windows-x64" | "ubuntu-x64") {
        return Err(AppError::bad_request(
            "platform must be windows-x64 or ubuntu-x64",
        ));
    }
    if !is_semver(&body.agent_version) {
        return Err(AppError::bad_request("agentVersion must be semver"));
    }
    // 27 unpadded Base64URL characters, per doc 6.3.5. Checking the shape before
    // hitting the database turns malformed input into a cheap 400.
    if body.pairing_code.len() != 27
        || !body
            .pairing_code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::not_found("Pairing code is invalid"));
    }
    Ok(())
}

fn is_semver(value: &str) -> bool {
    let core = value.split(['-', '+']).next().unwrap_or("");
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.chars().all(|c| c.is_ascii_digit())
                && (p.len() == 1 || !p.starts_with('0'))
        })
}

fn hex_key(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// --- GET /v1/devices --------------------------------------------------------

#[derive(Serialize)]
pub struct DeviceView {
    pub id: String,
    pub name: String,
    pub platform: String,
    #[serde(rename = "agentVersion")]
    pub agent_version: String,
    pub online: bool,
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: Option<String>,
}

#[derive(Serialize)]
pub struct DeviceList {
    pub devices: Vec<DeviceView>,
}

pub async fn list_devices(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<DeviceList>, AppError> {
    state
        .limiter
        .check(&format!("devices:{}", user.user_id), limits::LIST_DEVICES)?;

    let devices = state.db.list_devices(user.user_id).await?;

    let devices = devices
        .into_iter()
        .map(|d| DeviceView {
            id: d.id.to_string(),
            name: d.name,
            platform: d.platform,
            agent_version: d.agent_version,
            // Doc 4.11: online status comes from the in-memory registry, which
            // is why the plugin polls this endpoint instead of being pushed to.
            online: state.registry.is_online(d.id),
            last_seen_at: d.last_seen_at.map(|t| t.to_rfc3339()),
        })
        .collect();

    Ok(Json(DeviceList { devices }))
}

// --- DELETE /v1/devices/{id} ------------------------------------------------

pub async fn delete_device(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if state.db.delete_device(user.user_id, id).await? {
        tracing::info!(user_id = %user.user_id, device_id = %id, "device unbound");
        Ok(StatusCode::NO_CONTENT)
    } else if state.db.device_exists(id).await? {
        // The device exists but belongs to someone else. Doc 6.2 maps that to
        // 403 for this route, unlike pairing codes, because the plugin only ever
        // sends ids it just read from its own device list - a mismatch means a
        // real ownership problem worth surfacing.
        Err(AppError::forbidden())
    } else {
        Err(AppError::not_found("Device not found"))
    }
}
