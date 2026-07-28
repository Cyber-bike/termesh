//! Agent-facing WSS endpoint: `/v1/agent/ws` (doc 8.2).

use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use futures_util::StreamExt;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::ClientIp;
use crate::crypto;
use crate::error::AppError;
use crate::gateway::control::require_subprotocol;
use crate::gateway::registry::{ConnHandle, Outbound, Route, SendError};
use crate::gateway::{
    close, close_reason, writer_task, AGENT_OFFLINE_TIMEOUT, HEARTBEAT_INTERVAL_MS,
    MAX_CONTROL_FRAME_BYTES, SUBPROTOCOL,
};
use crate::ratelimit::limits;
use termy_protocol::frame;

/// How often the monitor checks liveness and considers flushing lastSeenAt.
const MONITOR_TICK: Duration = Duration::from_secs(5);
/// Doc 11.1: lastSeenAt is flushed at most this often, not on every 20 s heartbeat.
const LAST_SEEN_FLUSH_INTERVAL: Duration = Duration::from_secs(60);

pub async fn agent_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    state.limiter.check(&format!("ws-upgrade:{ip}"), limits::WS_UPGRADE)?;
    require_subprotocol(&headers)?;

    let (device_id, user_id) = authenticate(&state, &headers).await?;

    Ok(ws
        .protocols([SUBPROTOCOL])
        .on_upgrade(move |socket| handle(socket, state, device_id, user_id)))
}

/// `Authorization: Device <deviceId>.<deviceToken>` (doc 8.2).
///
/// The token is looked up by its keyed digest, then the device id in the header
/// is checked against the row that digest resolved to. Presenting someone else's
/// id with your own token therefore fails, and the token never appears in a log.
async fn authenticate(state: &AppState, headers: &HeaderMap) -> Result<(Uuid, Uuid), AppError> {
    let header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(AppError::unauthorized)?;

    let credential = header.strip_prefix("Device ").ok_or_else(AppError::unauthorized)?.trim();
    let (claimed_id, token) = credential.split_once('.').ok_or_else(AppError::unauthorized)?;

    let claimed_id = Uuid::parse_str(claimed_id).map_err(|_| AppError::unauthorized())?;

    let digest = crypto::digest_secret(&state.config.pepper, token);
    let device = state
        .db
        .find_device_by_token_digest(&digest)
        .await?
        .ok_or_else(AppError::unauthorized)?;

    if device.id != claimed_id {
        return Err(AppError::unauthorized());
    }

    Ok((device.id, device.user_id))
}

async fn handle(socket: WebSocket, state: AppState, device_id: Uuid, user_id: Uuid) {
    let (sink, mut stream) = socket.split();
    let (handle, control_rx, file_rx) = ConnHandle::channel();
    let writer = tokio::spawn(writer_task(sink, control_rx, file_rx));

    if let Some(displaced) = state.registry.register_agent(device_id, user_id, handle.clone()) {
        // Doc 8.2: one connection per device, newest wins. Without this an agent
        // restarted by systemd would fight its own stale connection.
        let _ = displaced
            .try_send_control(Outbound::Close(close::CONFLICT, close_reason("AGENT_REPLACED")));
    }
    tracing::info!(%device_id, %user_id, "agent connected");

    let monitor = tokio::spawn(monitor_liveness(state.clone(), device_id, handle.clone()));

    while let Some(Ok(message)) = stream.next().await {
        state.registry.touch_agent(device_id);

        let outcome = match message {
            Message::Text(text) => {
                on_text(&state, device_id, user_id, text.as_str()).await
            }
            Message::Binary(bytes) => on_binary(&state, device_id, user_id, &bytes).await,
            Message::Ping(_) | Message::Pong(_) => Ok(()),
            Message::Close(_) => break,
        };

        if let Err(f) = outcome {
            let _ = handle.try_send_control(Outbound::Close(f.code, close_reason(&f.reason)));
            break;
        }
    }

    monitor.abort();
    cleanup(&state, device_id, user_id, &handle).await;
    crate::gateway::control::drain_writer(handle, writer).await;
    tracing::info!(%device_id, "agent disconnected");
}

/// Doc 8.6: no heartbeat or pong within 50 s means the device is offline and its
/// PTY must end. Also handles the throttled lastSeenAt flush from doc 11.1.
async fn monitor_liveness(state: AppState, device_id: Uuid, handle: ConnHandle) {
    let mut last_flush = std::time::Instant::now();

    loop {
        tokio::time::sleep(MONITOR_TICK).await;

        let Some(last_seen) = state.registry.agent_last_seen(device_id) else {
            return;
        };

        if last_seen.elapsed() > AGENT_OFFLINE_TIMEOUT {
            tracing::warn!(%device_id, "agent heartbeat timed out");
            let _ = handle
                .try_send_control(Outbound::Close(close::TIMEOUT, close_reason("SESSION_TIMEOUT")));
            return;
        }

        if last_flush.elapsed() >= LAST_SEEN_FLUSH_INTERVAL {
            if let Err(e) = state.db.touch_last_seen(device_id).await {
                tracing::warn!(%device_id, "lastSeenAt flush failed: {e}");
            }
            last_flush = std::time::Instant::now();
        }

        // A WebSocket Ping keeps NAT and proxy idle timers open and gives the
        // 50 s window something to observe when the agent has nothing to say.
        // Doc 8.4 makes agent.helloAck a one-shot reply to agent.hello, so it
        // must not be reused as a keepalive.
        let _ = handle.try_send_control(Outbound::Ping);
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
    device_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<(), Fault> {
    if text.len() > MAX_CONTROL_FRAME_BYTES {
        return Err(fault(close::TOO_LARGE, "CONTROL_FRAME_TOO_LARGE"));
    }

    let mut value: Value =
        serde_json::from_str(text).map_err(|_| fault(close::PROTOCOL, "PROTOCOL_ERROR"))?;

    // Doc 8.3: the relay must not trust a client-declared deviceId. Overwriting
    // it with the authenticated identity means a compromised agent cannot
    // impersonate another device downstream.
    value["deviceId"] = json!(device_id.to_string());

    if let Err(errors) = termy_protocol::validate(&value) {
        tracing::debug!(%device_id, "agent message rejected: {}", errors.join("; "));
        return Err(fault(close::PROTOCOL, "PROTOCOL_ERROR"));
    }

    let msg_type = value["type"].as_str().unwrap_or_default().to_string();

    match msg_type.as_str() {
        "agent.hello" => {
            let ack = json!({
                "protocolVersion": 1,
                "type": "agent.helloAck",
                "requestId": value["requestId"].clone(),
                "deviceId": device_id.to_string(),
                "sessionId": null,
                "payload": {
                    "serverTime": chrono::Utc::now().to_rfc3339(),
                    "heartbeatIntervalMs": HEARTBEAT_INTERVAL_MS
                }
            });

            let handle = state
                .registry
                .agent_handle(device_id)
                .ok_or_else(|| fault(close::INTERNAL, "RELAY_INTERNAL"))?;
            handle
                .try_send_control(Outbound::Text(ack.to_string()))
                .map_err(|_| fault(close::INTERNAL, "RELAY_INTERNAL"))?;

            if let Err(e) = state.db.touch_last_seen(device_id).await {
                tracing::warn!(%device_id, "lastSeenAt update failed: {e}");
            }
            Ok(())
        }
        // Heartbeats terminate at the relay; touch_agent already ran.
        "agent.heartbeat" => Ok(()),

        "terminal.opened" => {
            let session_id = envelope_uuid(&value, "sessionId")?;
            state.registry.open_session(session_id, Route { user_id, device_id });
            forward_to_control(state, user_id, &value)
        }
        "terminal.close" => {
            let session_id = envelope_uuid(&value, "sessionId")?;
            state.registry.close_session(session_id);
            forward_to_control(state, user_id, &value)
        }
        "terminal.error" | "terminal.shellEvent" => forward_to_control(state, user_id, &value),

        "transfer.accepted" | "transfer.credit" => forward_to_control(state, user_id, &value),
        "transfer.result" => {
            let transfer_id = payload_uuid(&value, "transferId")?;
            state.registry.close_transfer(transfer_id);
            forward_to_control(state, user_id, &value)
        }

        // Plugin -> agent messages must not arrive on this socket.
        _ => Err(fault(close::PROTOCOL, "PROTOCOL_ERROR")),
    }
}

async fn on_binary(
    state: &AppState,
    device_id: Uuid,
    user_id: Uuid,
    bytes: &[u8],
) -> Result<(), Fault> {
    let decoded = frame::decode(bytes).map_err(|e| {
        tracing::debug!(%device_id, "binary frame rejected: {e}");
        fault(close::PROTOCOL, "PROTOCOL_ERROR")
    })?;

    // MVP has no file return path, so terminal output is the only thing an agent
    // may push (doc 2.2).
    if decoded.kind != frame::KIND_TERMINAL_OUTPUT {
        return Err(fault(close::PROTOCOL, "PROTOCOL_ERROR"));
    }

    let session_id = Uuid::from_bytes(decoded.stream_id);
    let route = state
        .registry
        .session_route(session_id)
        .filter(|r| r.device_id == device_id)
        .ok_or_else(|| fault(close::PROTOCOL, "PROTOCOL_ERROR"))?;

    let Some(control) = state.registry.control_handle(route.user_id) else {
        // Nobody is listening. Dropping the bytes is correct: doc 8.8.5 says no
        // buffering once a peer is gone.
        return Ok(());
    };

    // Downstream watermark (doc 8.6): awaiting here stops this read loop, which
    // stops draining the agent socket and ultimately throttles the remote
    // process. Only safe downstream, where no file body shares the connection.
    match control.send_control(Outbound::Binary(bytes.to_vec())).await {
        Ok(()) => Ok(()),
        Err(SendError::Closed) => Ok(()),
        Err(SendError::Full) => Err(fault(close::TOO_LARGE, "BACKPRESSURE_LIMIT")),
    }
    .map(|_| {
        let _ = user_id;
    })
}

fn forward_to_control(state: &AppState, user_id: Uuid, value: &Value) -> Result<(), Fault> {
    let Some(control) = state.registry.control_handle(user_id) else {
        return Ok(());
    };

    let text = serde_json::to_string(value).map_err(|_| fault(close::INTERNAL, "RELAY_INTERNAL"))?;
    match control.try_send_control(Outbound::Text(text)) {
        Ok(()) | Err(SendError::Closed) => Ok(()),
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

/// Doc 11.2.1 and 11.2.4: drop the online entry, tear down every route this
/// device carried and tell the plugin, then persist lastSeenAt.
async fn cleanup(state: &AppState, device_id: Uuid, user_id: Uuid, handle: &ConnHandle) {
    if !state.registry.unregister_agent_if_current(device_id, handle) {
        // Displaced by a newer connection, which now owns these routes.
        return;
    }

    let dropped = state.registry.drop_routes_for_device(device_id);

    if let Some(control) = state.registry.control_handle(user_id) {
        for (session_id, _) in dropped.sessions {
            let _ = control.try_send_control(Outbound::Text(
                json!({
                    "protocolVersion": 1,
                    "type": "terminal.close",
                    "requestId": null,
                    "deviceId": device_id.to_string(),
                    "sessionId": session_id.to_string(),
                    "payload": { "reason": "peer_disconnected", "exitCode": null }
                })
                .to_string(),
            ));
        }

        for (transfer_id, _) in dropped.transfers {
            let _ = control.try_send_control(Outbound::Text(
                json!({
                    "protocolVersion": 1,
                    "type": "transfer.result",
                    "requestId": null,
                    "deviceId": device_id.to_string(),
                    "sessionId": null,
                    "payload": {
                        "transferId": transfer_id.to_string(),
                        "success": false,
                        "code": "DEVICE_OFFLINE",
                        "message": "Agent disconnected during the transfer"
                    }
                })
                .to_string(),
            ));
        }
    }

    if let Err(e) = state.db.touch_last_seen(device_id).await {
        tracing::warn!(%device_id, "final lastSeenAt flush failed: {e}");
    }
}
