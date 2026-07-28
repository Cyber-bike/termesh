//! Plugin-facing WSS endpoint: `/v1/control/ws` (doc 8.2).

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use futures_util::StreamExt;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::{AuthUser, ClientIp};
use crate::error::AppError;
use crate::gateway::registry::{ConnHandle, Outbound, Route, SendError};
use crate::gateway::{close, close_reason, writer_task, MAX_CONTROL_FRAME_BYTES, SUBPROTOCOL};
use crate::ratelimit::limits;
use termy_protocol::frame;

pub async fn control_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    user: AuthUser,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    state.limiter.check(&format!("ws-upgrade:{ip}"), limits::WS_UPGRADE)?;
    require_subprotocol(&headers)?;

    let user_id = user.user_id;
    Ok(ws
        .protocols([SUBPROTOCOL])
        .on_upgrade(move |socket| handle(socket, state, user_id)))
}

/// Doc 8.2 requires the subprotocol to be validated before the 101, so a client
/// speaking a future protocol version fails at the handshake instead of getting
/// a connection it cannot use.
pub fn require_subprotocol(headers: &HeaderMap) -> Result<(), AppError> {
    let offered = headers
        .get(axum::http::header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    if offered.split(',').map(str::trim).any(|p| p == SUBPROTOCOL) {
        Ok(())
    } else {
        Err(AppError::bad_request(format!("the {SUBPROTOCOL} subprotocol is required")))
    }
}

async fn handle(socket: WebSocket, state: AppState, user_id: Uuid) {
    let (sink, mut stream) = socket.split();
    let (handle, control_rx, file_rx) = ConnHandle::channel();
    let writer = tokio::spawn(writer_task(sink, control_rx, file_rx));

    if let Some(displaced) = state.registry.register_control(user_id, handle.clone()) {
        // Doc 8.2: the newest control connection wins.
        let _ = displaced
            .try_send_control(Outbound::Close(close::CONFLICT, close_reason("CONTROL_REPLACED")));
    }
    tracing::info!(%user_id, "control connection established");

    while let Some(Ok(message)) = stream.next().await {
        let outcome = match message {
            Message::Text(text) => on_text(&state, user_id, &handle, text.as_str()).await,
            Message::Binary(bytes) => on_binary(&state, user_id, &handle, &bytes).await,
            Message::Ping(_) | Message::Pong(_) => Ok(()),
            Message::Close(_) => break,
        };

        if let Err(fault) = outcome {
            let _ = handle.try_send_control(Outbound::Close(fault.code, close_reason(&fault.reason)));
            break;
        }
    }

    cleanup(&state, user_id, &handle).await;
    drain_writer(handle, writer).await;
    tracing::info!(%user_id, "control connection closed");
}

/// Lets the writer flush anything already queued - crucially a Close frame -
/// before the task is torn down.
///
/// Aborting the writer straight after queueing a Close loses it, and the peer
/// sees a TCP reset instead of the documented close code. All senders are gone
/// once the registry entry and the local handle are dropped, so the writer ends
/// on its own; the timeout only covers a clone still held by a task that is
/// being cancelled.
pub async fn drain_writer(handle: ConnHandle, writer: tokio::task::JoinHandle<()>) {
    drop(handle);
    match tokio::time::timeout(std::time::Duration::from_secs(2), writer).await {
        Ok(_) => {}
        Err(_) => tracing::debug!("writer did not finish within the drain timeout"),
    }
}


struct Fault {
    code: u16,
    reason: String,
}

fn fault(code: u16, reason: &str) -> Fault {
    Fault { code, reason: reason.to_string() }
}

async fn on_text(
    state: &AppState,
    user_id: Uuid,
    handle: &ConnHandle,
    text: &str,
) -> Result<(), Fault> {
    if text.len() > MAX_CONTROL_FRAME_BYTES {
        return Err(fault(close::TOO_LARGE, "CONTROL_FRAME_TOO_LARGE"));
    }

    let value: Value =
        serde_json::from_str(text).map_err(|_| fault(close::PROTOCOL, "PROTOCOL_ERROR"))?;

    // The relay validates against the same schema the plugin and agent use, so a
    // malformed message never reaches the agent.
    if let Err(errors) = termy_protocol::validate(&value) {
        tracing::debug!(%user_id, "control message rejected: {}", errors.join("; "));
        return Err(fault(close::PROTOCOL, "PROTOCOL_ERROR"));
    }

    let msg_type = value["type"].as_str().unwrap_or_default().to_string();

    match msg_type.as_str() {
        "terminal.open" | "transfer.start" => {
            let device_id = envelope_uuid(&value, "deviceId")?;
            let agent = authorize_device(state, user_id, device_id, handle).await?;

            if msg_type == "transfer.start" {
                let transfer_id = payload_uuid(&value, "transferId")?;
                state
                    .registry
                    .open_transfer(transfer_id, Route { user_id, device_id });
            }

            forward_control(&agent, &value)
        }
        "terminal.resize" | "terminal.close" => {
            let session_id = envelope_uuid(&value, "sessionId")?;
            let route = state
                .registry
                .session_route(session_id)
                .filter(|r| r.user_id == user_id)
                .ok_or_else(|| fault(close::FORBIDDEN, "DEVICE_FORBIDDEN"))?;

            let agent = state
                .registry
                .agent_handle(route.device_id)
                .ok_or_else(|| fault(close::FORBIDDEN, "DEVICE_OFFLINE"))?;

            if msg_type == "terminal.close" {
                state.registry.close_session(session_id);
            }
            forward_control(&agent, &value)
        }
        "transfer.fileEnd" | "transfer.complete" | "transfer.abort" => {
            let transfer_id = payload_uuid(&value, "transferId")?;
            let agent = transfer_agent(state, user_id, transfer_id)?;

            if msg_type == "transfer.abort" {
                state.registry.close_transfer(transfer_id);
            }
            forward_control(&agent, &value)
        }
        // Everything else in the enum travels Agent -> plugin. Receiving one
        // here means the peer is confused about its own role.
        _ => Err(fault(close::PROTOCOL, "PROTOCOL_ERROR")),
    }
}

async fn on_binary(
    state: &AppState,
    user_id: Uuid,
    _handle: &ConnHandle,
    bytes: &[u8],
) -> Result<(), Fault> {
    let decoded = frame::decode(bytes).map_err(|e| {
        tracing::debug!(%user_id, "binary frame rejected: {e}");
        fault(close::PROTOCOL, "PROTOCOL_ERROR")
    })?;

    let stream_id = Uuid::from_bytes(decoded.stream_id);

    match decoded.kind {
        frame::KIND_TERMINAL_INPUT => {
            let route = state
                .registry
                .session_route(stream_id)
                .filter(|r| r.user_id == user_id)
                .ok_or_else(|| fault(close::FORBIDDEN, "DEVICE_FORBIDDEN"))?;

            let agent = state
                .registry
                .agent_handle(route.device_id)
                .ok_or_else(|| fault(close::FORBIDDEN, "DEVICE_OFFLINE"))?;

            match agent.try_send_control(Outbound::Binary(bytes.to_vec())) {
                Ok(()) => Ok(()),
                Err(SendError::Closed) => Err(fault(close::FORBIDDEN, "DEVICE_OFFLINE")),
                Err(SendError::Full) => Err(fault(close::TOO_LARGE, "BACKPRESSURE_LIMIT")),
            }
        }
        frame::KIND_FILE_CHUNK => {
            let agent = transfer_agent(state, user_id, stream_id)?;

            // The file lane is bounded at the 16 MiB hard limit. A sender that
            // honours its credit window cannot fill it, so a full lane means the
            // window was ignored - doc 8.6 says close rather than buffer.
            match agent.try_send_file(Outbound::Binary(bytes.to_vec())) {
                Ok(()) => Ok(()),
                Err(SendError::Closed) => Err(fault(close::FORBIDDEN, "DEVICE_OFFLINE")),
                Err(SendError::Full) => Err(fault(close::TOO_LARGE, "BACKPRESSURE_LIMIT")),
            }
        }
        // Terminal output only ever flows Agent -> plugin.
        _ => Err(fault(close::PROTOCOL, "PROTOCOL_ERROR")),
    }
}

/// Doc 11.2.3: ownership and liveness are re-checked at the moment a session or
/// transfer starts, not just when the device list was fetched.
async fn authorize_device(
    state: &AppState,
    user_id: Uuid,
    device_id: Uuid,
    _handle: &ConnHandle,
) -> Result<ConnHandle, Fault> {
    let owned = state
        .db
        .list_devices(user_id)
        .await
        .map_err(|_| fault(close::INTERNAL, "RELAY_INTERNAL"))?
        .into_iter()
        .any(|d| d.id == device_id);

    if !owned {
        return Err(fault(close::FORBIDDEN, "DEVICE_FORBIDDEN"));
    }

    state
        .registry
        .agent_handle(device_id)
        .ok_or_else(|| fault(close::FORBIDDEN, "DEVICE_OFFLINE"))
}

fn transfer_agent(state: &AppState, user_id: Uuid, transfer_id: Uuid) -> Result<ConnHandle, Fault> {
    let route = state
        .registry
        .transfer_route(transfer_id)
        .filter(|r| r.user_id == user_id)
        .ok_or_else(|| fault(close::FORBIDDEN, "DEVICE_FORBIDDEN"))?;

    state
        .registry
        .agent_handle(route.device_id)
        .ok_or_else(|| fault(close::FORBIDDEN, "DEVICE_OFFLINE"))
}

fn forward_control(agent: &ConnHandle, value: &Value) -> Result<(), Fault> {
    let text = serde_json::to_string(value).map_err(|_| fault(close::INTERNAL, "RELAY_INTERNAL"))?;
    match agent.try_send_control(Outbound::Text(text)) {
        Ok(()) => Ok(()),
        Err(SendError::Closed) => Err(fault(close::FORBIDDEN, "DEVICE_OFFLINE")),
        Err(SendError::Full) => Err(fault(close::TOO_LARGE, "BACKPRESSURE_LIMIT")),
    }
}

fn envelope_uuid(value: &Value, field: &str) -> Result<Uuid, Fault> {
    value[field]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| fault(close::PROTOCOL, "PROTOCOL_ERROR"))
}

fn payload_uuid(value: &Value, field: &str) -> Result<Uuid, Fault> {
    value["payload"][field]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| fault(close::PROTOCOL, "PROTOCOL_ERROR"))
}

/// Doc 11.2.4: when the plugin goes away, its sessions and transfers die with
/// it and the agent is told so it can tear down the PTY.
async fn cleanup(state: &AppState, user_id: Uuid, handle: &ConnHandle) {
    if !state.registry.unregister_control_if_current(user_id, handle) {
        // A newer connection already replaced this one; its routes are still live.
        return;
    }

    let dropped = state.registry.drop_routes_for_user(user_id);

    for (session_id, route) in dropped.sessions {
        let Some(agent) = state.registry.agent_handle(route.device_id) else {
            continue;
        };
        let _ = agent.try_send_control(Outbound::Text(
            json!({
                "protocolVersion": 1,
                "type": "terminal.close",
                "requestId": null,
                "deviceId": route.device_id.to_string(),
                "sessionId": session_id.to_string(),
                "payload": { "reason": "peer_disconnected", "exitCode": null }
            })
            .to_string(),
        ));
    }

    for (transfer_id, route) in dropped.transfers {
        // MVP has no transfer.abort from the relay, so the agent falls back to
        // its 30 s idle timeout and reports TRANSFER_FAILED itself.
        tracing::debug!(%user_id, %transfer_id, device_id = %route.device_id,
            "transfer dropped with its control connection");
    }
}
