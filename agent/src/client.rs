//! Outbound WSS client and the message loop (doc 7.6, 8.2, 9, 10).

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::config::Config;
use crate::pty::{PtySession, SessionSlot};
use crate::state::{self, AgentState, ConnectionState};
use crate::transfer::{Entry, TransferSession};
use crate::AgentError;
use termy_protocol::frame;

const HEARTBEAT: Duration = Duration::from_secs(20);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Doc 7.6: 1 s doubling to a 30 s ceiling, with jitter, retried forever.
const BACKOFF_MIN: Duration = Duration::from_secs(1);
const BACKOFF_MAX: Duration = Duration::from_secs(30);

pub const AGENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// What a connection attempt concluded, which decides whether to retry.
enum Disconnect {
    /// Transport level; retry after backoff.
    Retryable(String),
    /// The relay rejected our credentials. Retrying cannot help.
    Unauthorized,
}

/// Runs forever, reconnecting with backoff (doc 7.6).
pub async fn run(config: Config) -> Result<(), AgentError> {
    let state_path = state::state_path();
    let mut backoff = BACKOFF_MIN;

    let mut agent_state = AgentState::new();
    let _ = state::write(&state_path, &agent_state);

    loop {
        agent_state.connection = ConnectionState::Connecting;
        let _ = state::write(&state_path, &agent_state);

        match connect_and_serve(&config, &state_path, &mut agent_state).await {
            Ok(Disconnect::Unauthorized) => {
                agent_state.connection = ConnectionState::Disconnected;
                agent_state.needs_rebind = true;
                let _ = state::write(&state_path, &agent_state);
                return Err(AgentError::Config(
                    "the relay rejected this device token. Run `termy-agent bind --code <pairing-code>` again"
                        .into(),
                ));
            }
            Ok(Disconnect::Retryable(reason)) | Err(AgentError::Protocol(reason)) => {
                tracing::warn!("disconnected: {reason}");
            }
            Err(e) => {
                tracing::warn!("connection failed: {e}");
            }
        }

        agent_state.connection = ConnectionState::Disconnected;
        agent_state.session_active = false;
        agent_state.last_disconnected_at = Some(chrono::Utc::now().to_rfc3339());
        let _ = state::write(&state_path, &agent_state);

        let wait = with_jitter(backoff);
        tracing::info!("reconnecting in {:.1}s", wait.as_secs_f32());
        tokio::time::sleep(wait).await;
        backoff = (backoff * 2).min(BACKOFF_MAX);
    }
}

/// +/-20% so a fleet restarted together does not reconnect in lockstep.
fn with_jitter(base: Duration) -> Duration {
    use rand::Rng;
    let factor = rand::thread_rng().gen_range(0.8_f64..1.2_f64);
    Duration::from_secs_f64(base.as_secs_f64() * factor)
}

async fn connect_and_serve(
    config: &Config,
    state_path: &std::path::Path,
    agent_state: &mut AgentState,
) -> Result<Disconnect, AgentError> {
    let mut request = config
        .relay_url
        .as_str()
        .into_client_request()
        .map_err(|e| AgentError::Config(format!("relayUrl is not a valid URL: {e}")))?;

    request.headers_mut().insert(
        "authorization",
        format!("Device {}.{}", config.device_id, config.device_token)
            .parse()
            .map_err(|_| AgentError::Config("device credentials are not header-safe".into()))?,
    );
    request
        .headers_mut()
        .insert("sec-websocket-protocol", "termy.v1".parse().unwrap());

    let connect = tokio_tungstenite::connect_async(request);
    let (socket, _) = match tokio::time::timeout(CONNECT_TIMEOUT, connect).await {
        Err(_) => return Ok(Disconnect::Retryable("connect timed out".into())),
        Ok(Err(e)) => {
            let text = e.to_string();
            // The relay answers a bad device token with 401 before the upgrade.
            if text.contains("401") {
                return Ok(Disconnect::Unauthorized);
            }
            return Ok(Disconnect::Retryable(text));
        }
        Ok(Ok(pair)) => pair,
    };

    tracing::info!("connected to the relay");
    agent_state.connection = ConnectionState::Connected;
    agent_state.last_connected_at = Some(chrono::Utc::now().to_rfc3339());
    agent_state.needs_rebind = false;
    let _ = state::write(state_path, agent_state);

    serve(socket, config, state_path, agent_state).await
}

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// One connection's lifetime.
struct Session {
    id: Uuid,
    pty: PtySession,
    output_offset: u64,
    input_offset: u64,
}

async fn serve(
    socket: Socket,
    config: &Config,
    state_path: &std::path::Path,
    agent_state: &mut AgentState,
) -> Result<Disconnect, AgentError> {
    let (mut sink, mut stream) = socket.split();
    let (out_tx, mut out_rx) = mpsc::channel::<Message>(256);

    let writer = tokio::spawn(async move {
        while let Some(message) = out_rx.recv().await {
            if sink.send(message).await.is_err() {
                break;
            }
        }
    });

    // PTY output arrives on a blocking thread; this channel carries it back into
    // the async loop.
    let (pty_tx, mut pty_rx) = mpsc::channel::<PtyEvent>(256);

    let slot = SessionSlot::new();
    let mut session: Option<Session> = None;
    let mut transfer: Option<(Uuid, TransferSession)> = None;

    send_json(
        &out_tx,
        json!({
            "protocolVersion": 1,
            "type": "agent.hello",
            "requestId": Uuid::new_v4().to_string(),
            "deviceId": config.device_id,
            "sessionId": null,
            "payload": {
                "agentVersion": AGENT_VERSION,
                "platform": platform(),
                "capabilities": ["terminal", "file-transfer"]
            }
        }),
    )
    .await;

    let mut heartbeat = tokio::time::interval(HEARTBEAT);
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    heartbeat.tick().await; // the first tick fires immediately

    let outcome = loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                send_json(&out_tx, json!({
                    "protocolVersion": 1,
                    "type": "agent.heartbeat",
                    "requestId": null,
                    "deviceId": config.device_id,
                    "sessionId": null,
                    "payload": { "timestamp": chrono::Utc::now().to_rfc3339() }
                })).await;
            }

            Some(event) = pty_rx.recv() => {
                if let Some(active) = session.as_mut() {
                    match event {
                        PtyEvent::Output(bytes) => {
                            let encoded = frame::encode(&frame::Frame {
                                kind: frame::KIND_TERMINAL_OUTPUT,
                                stream_id: *active.id.as_bytes(),
                                file_index: frame::TERMINAL_FILE_INDEX,
                                offset: active.output_offset,
                                payload: bytes.clone(),
                            }).map_err(|e| AgentError::Protocol(e.to_string()))?;
                            active.output_offset += bytes.len() as u64;
                            if out_tx.send(Message::Binary(encoded.into())).await.is_err() {
                                break Disconnect::Retryable("writer stopped".into());
                            }
                        }
                        PtyEvent::Shell { name, source, exit_code } => {
                            send_json(&out_tx, json!({
                                "protocolVersion": 1,
                                "type": "terminal.shellEvent",
                                "requestId": null,
                                "deviceId": config.device_id,
                                "sessionId": active.id.to_string(),
                                "payload": { "type": name, "source": source, "exitCode": exit_code }
                            })).await;
                        }
                        PtyEvent::Exited(code) => {
                            send_json(&out_tx, json!({
                                "protocolVersion": 1,
                                "type": "terminal.close",
                                "requestId": null,
                                "deviceId": config.device_id,
                                "sessionId": active.id.to_string(),
                                "payload": { "reason": "shell_exited", "exitCode": code }
                            })).await;
                            slot.release(active.id);
                            session = None;
                            agent_state.session_active = false;
                            let _ = state::write(state_path, agent_state);
                        }
                    }
                }
            }

            incoming = stream.next() => {
                let Some(message) = incoming else {
                    break Disconnect::Retryable("relay closed the connection".into());
                };

                match message {
                    Err(e) => break Disconnect::Retryable(e.to_string()),
                    Ok(Message::Close(frame)) => {
                        let code = frame.as_ref().map(|f| u16::from(f.code)).unwrap_or(1000);
                        // Doc 8.7: 4401 means the token is no longer valid.
                        break if code == 4401 {
                            Disconnect::Unauthorized
                        } else {
                            Disconnect::Retryable(format!("relay closed with {code}"))
                        };
                    }
                    Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
                    Ok(Message::Text(text)) => {
                        if let Err(e) = handle_control(
                            &text, config, &out_tx, &slot, &mut session, &mut transfer,
                            &pty_tx, state_path, agent_state,
                        ).await {
                            tracing::warn!("control message failed: {e}");
                        }
                    }
                    Ok(Message::Binary(bytes)) => {
                        if let Err(e) = handle_binary(
                            &bytes, config, &out_tx, &mut session, &mut transfer,
                        ).await {
                            tracing::warn!("binary frame failed: {e}");
                        }
                    }
                    Ok(_) => {}
                }
            }
        }
    };

    // Doc 7.6: the PTY dies with the connection; a reconnect does not resume it.
    if let Some(mut active) = session.take() {
        active.pty.terminate();
    }
    if let Some((_, mut active)) = transfer.take() {
        active.abort();
    }
    slot.release_any();
    agent_state.session_active = false;
    let _ = state::write(state_path, agent_state);

    drop(out_tx);
    let _ = tokio::time::timeout(Duration::from_secs(2), writer).await;

    Ok(outcome)
}

enum PtyEvent {
    Output(Vec<u8>),
    Shell {
        name: &'static str,
        source: &'static str,
        exit_code: Option<i32>,
    },
    Exited(i32),
}

#[allow(clippy::too_many_arguments)]
async fn handle_control(
    text: &str,
    config: &Config,
    out_tx: &mpsc::Sender<Message>,
    slot: &SessionSlot,
    session: &mut Option<Session>,
    transfer: &mut Option<(Uuid, TransferSession)>,
    pty_tx: &mpsc::Sender<PtyEvent>,
    state_path: &std::path::Path,
    agent_state: &mut AgentState,
) -> Result<(), AgentError> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| AgentError::Protocol(e.to_string()))?;
    termy_protocol::validate(&value).map_err(|errors| AgentError::Protocol(errors.join("; ")))?;

    let msg_type = value["type"].as_str().unwrap_or_default();
    let request_id = value["requestId"].clone();

    match msg_type {
        "agent.helloAck" => Ok(()),

        "terminal.open" => {
            let cols = value["payload"]["cols"].as_u64().unwrap_or(80) as u16;
            let rows = value["payload"]["rows"].as_u64().unwrap_or(24) as u16;

            let session_id = Uuid::new_v4();
            if !slot.claim(session_id) {
                // Doc 7.5: one PTY at a time, refuse rather than queue.
                send_json(out_tx, json!({
                    "protocolVersion": 1,
                    "type": "terminal.error",
                    "requestId": request_id,
                    "deviceId": config.device_id,
                    "sessionId": null,
                    "payload": { "code": "DEVICE_BUSY", "message": "A remote terminal is already running" }
                })).await;
                return Ok(());
            }

            match PtySession::spawn(
                &config.shell.program,
                &config.shell.args,
                Some(&crate::config::home_dir()),
                cols,
                rows,
            ) {
                Ok((pty, reader)) => {
                    let shell = pty.shell.clone();
                    spawn_output_pump(reader, pty_tx.clone());

                    *session = Some(Session {
                        id: session_id,
                        pty,
                        output_offset: 0,
                        input_offset: 0,
                    });
                    agent_state.session_active = true;
                    let _ = state::write(state_path, agent_state);

                    send_json(
                        out_tx,
                        json!({
                            "protocolVersion": 1,
                            "type": "terminal.opened",
                            "requestId": request_id,
                            "deviceId": config.device_id,
                            "sessionId": session_id.to_string(),
                            "payload": { "shell": shell }
                        }),
                    )
                    .await;
                }
                Err(e) => {
                    slot.release(session_id);
                    send_json(out_tx, json!({
                        "protocolVersion": 1,
                        "type": "terminal.error",
                        "requestId": request_id,
                        "deviceId": config.device_id,
                        "sessionId": null,
                        // Redacted: the raw error can contain a filesystem path.
                        "payload": { "code": "SHELL_START_FAILED", "message": redact(&e.to_string()) }
                    })).await;
                }
            }
            Ok(())
        }

        "terminal.resize" => {
            if let Some(active) = session.as_ref() {
                let cols = value["payload"]["cols"].as_u64().unwrap_or(80) as u16;
                let rows = value["payload"]["rows"].as_u64().unwrap_or(24) as u16;
                let _ = active.pty.resize(cols, rows);
            }
            Ok(())
        }

        "terminal.close" => {
            if let Some(mut active) = session.take() {
                active.pty.terminate();
                slot.release(active.id);
                agent_state.session_active = false;
                let _ = state::write(state_path, agent_state);
            }
            Ok(())
        }

        "transfer.start" => {
            let transfer_id = uuid_at(&value, "transferId")?;
            let root_note = value["payload"]["rootNote"]
                .as_str()
                .unwrap_or_default()
                .to_string();

            let entries: Vec<Entry> = value["payload"]["entries"]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .map(|e| Entry {
                            index: e["index"].as_u64().unwrap_or_default() as usize,
                            relative_path: e["relativePath"].as_str().unwrap_or_default().into(),
                            size: e["size"].as_u64().unwrap_or_default(),
                        })
                        .collect()
                })
                .unwrap_or_default();

            const INITIAL_CREDIT: u64 = 4 * 1024 * 1024;

            match TransferSession::new(
                config.receive_root.clone(),
                entries,
                &root_note,
                INITIAL_CREDIT,
            ) {
                Ok(active) => {
                    *transfer = Some((transfer_id, active));
                    send_json(
                        out_tx,
                        json!({
                            "protocolVersion": 1,
                            "type": "transfer.accepted",
                            "requestId": request_id,
                            "deviceId": config.device_id,
                            "sessionId": null,
                            "payload": {
                                "transferId": transfer_id.to_string(),
                                "grantedBytes": INITIAL_CREDIT
                            }
                        }),
                    )
                    .await;
                }
                Err(e) => {
                    send_result(
                        out_tx,
                        config,
                        transfer_id,
                        false,
                        Some("INVALID_PATH"),
                        &redact(&e.to_string()),
                        None,
                    )
                    .await;
                }
            }
            Ok(())
        }

        "transfer.fileEnd" => {
            let transfer_id = uuid_at(&value, "transferId")?;
            let file_index = value["payload"]["fileIndex"].as_u64().unwrap_or_default() as usize;
            let sent_size = value["payload"]["sentSize"].as_u64().unwrap_or_default();

            if let Some((id, active)) = transfer.as_mut() {
                if *id == transfer_id {
                    if let Err(e) = active.finish_file(file_index, sent_size) {
                        active.abort();
                        let message = redact(&e.to_string());
                        transfer.take();
                        send_result(
                            out_tx,
                            config,
                            transfer_id,
                            false,
                            Some("TRANSFER_FAILED"),
                            &message,
                            None,
                        )
                        .await;
                    }
                }
            }
            Ok(())
        }

        "transfer.complete" => {
            let transfer_id = uuid_at(&value, "transferId")?;
            if let Some((id, mut active)) = transfer.take() {
                if id == transfer_id {
                    match active.complete() {
                        Ok(()) => {
                            let destination = active.destination_path().to_string_lossy();
                            send_result(
                                out_tx,
                                config,
                                transfer_id,
                                true,
                                None,
                                "",
                                Some(&destination),
                            )
                            .await;
                        }
                        Err(e) => {
                            active.abort();
                            send_result(
                                out_tx,
                                config,
                                transfer_id,
                                false,
                                Some("TRANSFER_FAILED"),
                                &redact(&e.to_string()),
                                None,
                            )
                            .await;
                        }
                    }
                } else {
                    *transfer = Some((id, active));
                }
            }
            Ok(())
        }

        "transfer.abort" => {
            let transfer_id = uuid_at(&value, "transferId")?;
            if let Some((id, mut active)) = transfer.take() {
                if id == transfer_id {
                    active.abort();
                    send_result(
                        out_tx,
                        config,
                        transfer_id,
                        false,
                        Some("TRANSFER_FAILED"),
                        "Transfer aborted by the sender; partial files may remain",
                        None,
                    )
                    .await;
                } else {
                    *transfer = Some((id, active));
                }
            }
            Ok(())
        }

        other => Err(AgentError::Protocol(format!(
            "unexpected message type {other}"
        ))),
    }
}

async fn handle_binary(
    bytes: &[u8],
    config: &Config,
    out_tx: &mpsc::Sender<Message>,
    session: &mut Option<Session>,
    transfer: &mut Option<(Uuid, TransferSession)>,
) -> Result<(), AgentError> {
    let decoded = frame::decode(bytes).map_err(|e| AgentError::Protocol(e.to_string()))?;
    let stream_id = Uuid::from_bytes(decoded.stream_id);

    match decoded.kind {
        frame::KIND_TERMINAL_INPUT => {
            let Some(active) = session.as_mut() else {
                return Ok(());
            };
            if active.id != stream_id {
                return Err(AgentError::Protocol("input for an unknown session".into()));
            }
            if decoded.offset != active.input_offset {
                // Doc 8.5: terminal offsets are reported, not fatal.
                tracing::debug!(
                    "terminal input offset {} where {} was expected",
                    decoded.offset,
                    active.input_offset
                );
            }
            active.input_offset = decoded.offset + decoded.payload.len() as u64;
            active.pty.write_input(&decoded.payload)?;
            Ok(())
        }

        frame::KIND_FILE_CHUNK => {
            let Some((id, active)) = transfer.as_mut() else {
                return Ok(());
            };
            if *id != stream_id {
                return Err(AgentError::Protocol("chunk for an unknown transfer".into()));
            }

            match active.write_chunk(
                decoded.file_index as usize,
                decoded.offset,
                &decoded.payload,
            ) {
                Ok(Some(granted)) => {
                    send_json(
                        out_tx,
                        json!({
                            "protocolVersion": 1,
                            "type": "transfer.credit",
                            "requestId": null,
                            "deviceId": config.device_id,
                            "sessionId": null,
                            "payload": { "transferId": id.to_string(), "grantedBytes": granted }
                        }),
                    )
                    .await;
                    Ok(())
                }
                Ok(None) => Ok(()),
                Err(e) => {
                    active.abort();
                    let message = redact(&e.to_string());
                    let transfer_id = *id;
                    transfer.take();
                    send_result(
                        out_tx,
                        config,
                        transfer_id,
                        false,
                        Some("TRANSFER_FAILED"),
                        &message,
                        None,
                    )
                    .await;
                    Ok(())
                }
            }
        }

        _ => Err(AgentError::Protocol(
            "agents do not receive terminal output".into(),
        )),
    }
}

/// Streams PTY bytes back into the async loop and derives shell events from the
/// same stream, so the two can never disagree about ordering.
fn spawn_output_pump(mut reader: Box<dyn std::io::Read + Send>, tx: mpsc::Sender<PtyEvent>) {
    std::thread::spawn(move || {
        let mut scanner = crate::osc::OscScanner::new();
        let mut buf = vec![0u8; 32 * 1024];

        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    for event in scanner.scan(&buf[..n]) {
                        let _ = tx.blocking_send(PtyEvent::Shell {
                            name: event.event_name(),
                            source: event.source_name(),
                            exit_code: event.exit_code(),
                        });
                    }
                    if tx
                        .blocking_send(PtyEvent::Output(buf[..n].to_vec()))
                        .is_err()
                    {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        // EOF on the master means the shell is gone.
        let _ = tx.blocking_send(PtyEvent::Exited(0));
    });
}

async fn send_json(tx: &mpsc::Sender<Message>, value: Value) {
    let _ = tx.send(Message::Text(value.to_string().into())).await;
}

async fn send_result(
    tx: &mpsc::Sender<Message>,
    config: &Config,
    transfer_id: Uuid,
    success: bool,
    code: Option<&str>,
    message: &str,
    destination_path: Option<&str>,
) {
    send_json(
        tx,
        json!({
            "protocolVersion": 1,
            "type": "transfer.result",
            "requestId": null,
            "deviceId": config.device_id,
            "sessionId": null,
            "payload": {
                "transferId": transfer_id.to_string(),
                "success": success,
                "code": code,
                "message": message,
                "destinationPath": destination_path
            }
        }),
    )
    .await;
}

fn uuid_at(value: &Value, field: &str) -> Result<Uuid, AgentError> {
    value["payload"][field]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AgentError::Protocol(format!("{field} is not a UUID")))
}

/// Doc 8.8.6 and 13.2: error text crossing the wire must not leak local paths
/// or anything else about the host.
fn redact(message: &str) -> String {
    let mut out: String = message
        .split_whitespace()
        .map(|word| {
            if word.contains('/') || word.contains('\\') {
                "<path>".to_string()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    out.truncate(512);
    out
}

pub fn platform() -> &'static str {
    if cfg!(windows) {
        "windows-x64"
    } else {
        "ubuntu-x64"
    }
}

/// Registers this machine with the relay and writes the config (doc 6.4).
pub async fn bind(
    relay_base: &str,
    pairing_code: &str,
    device_name: &str,
) -> Result<Config, AgentError> {
    let url = format!("{}/v1/devices/register", relay_base.trim_end_matches('/'));

    let body = json!({
        "pairingCode": pairing_code,
        "deviceName": device_name,
        "platform": platform(),
        "agentVersion": AGENT_VERSION
    });

    let response = http_post_json(&url, &body).await?;

    Ok(Config {
        device_id: response["deviceId"]
            .as_str()
            .ok_or_else(|| AgentError::Protocol("register response has no deviceId".into()))?
            .to_string(),
        device_token: response["deviceToken"]
            .as_str()
            .ok_or_else(|| AgentError::Protocol("register response has no deviceToken".into()))?
            .to_string(),
        device_name: device_name.to_string(),
        relay_url: response["relayUrl"]
            .as_str()
            .ok_or_else(|| AgentError::Protocol("register response has no relayUrl".into()))?
            .to_string(),
        receive_root: Config::default_receive_root(),
        shell: Config::default_shell(),
    })
}

/// A minimal TLS POST. The agent needs exactly one HTTPS call in its life, so a
/// full HTTP client would be a large dependency for a single request.
async fn http_post_json(url: &str, body: &Value) -> Result<Value, AgentError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let insecure = url.starts_with("http://") && crate::config::allow_insecure();
    if insecure {
        tracing::warn!("registering over plaintext HTTP because TERMY_AGENT_ALLOW_INSECURE=1");
    }

    let rest = url
        .strip_prefix("https://")
        .or_else(|| {
            if insecure {
                url.strip_prefix("http://")
            } else {
                None
            }
        })
        .ok_or_else(|| {
            AgentError::Config(
                "the relay base URL must be https://; set TERMY_AGENT_ALLOW_INSECURE=1 for local \
                 development against a plaintext relay"
                    .into(),
            )
        })?;
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    let host = authority.split(':').next().unwrap_or(authority).to_string();
    let port: u16 = authority
        .split(':')
        .nth(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(if insecure { 80 } else { 443 });

    let payload = body.to_string();
    let request = format!(
        "POST /{path} HTTP/1.1\r\nHost: {authority}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        payload.len()
    );

    let mut roots = tokio_rustls::rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let tls_config = tokio_rustls::rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let connector = tokio_rustls::TlsConnector::from(Arc::new(tls_config));

    let server_name = tokio_rustls::rustls::pki_types::ServerName::try_from(host.clone())
        .map_err(|_| AgentError::Config(format!("{host} is not a valid server name")))?;

    let tcp = tokio::net::TcpStream::connect((host.as_str(), port)).await?;

    let mut raw = Vec::new();
    if insecure {
        let mut stream = tcp;
        stream.write_all(request.as_bytes()).await?;
        stream.read_to_end(&mut raw).await?;
    } else {
        let mut stream = connector.connect(server_name, tcp).await?;
        stream.write_all(request.as_bytes()).await?;
        stream.read_to_end(&mut raw).await?;
    }
    let text = String::from_utf8_lossy(&raw).to_string();

    let status: u16 = text
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let body_text = text
        .split_once("\r\n\r\n")
        .map(|(_, b)| b)
        .unwrap_or("")
        .trim()
        .to_string();
    let parsed: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);

    if !(200..300).contains(&status) {
        let code = parsed["error"]["code"].as_str().unwrap_or("HTTP_ERROR");
        let message = parsed["error"]["message"]
            .as_str()
            .unwrap_or("registration failed");
        return Err(AgentError::Protocol(format!("{code}: {message}")));
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redaction_strips_paths() {
        let redacted = redact("cannot start /usr/local/bin/weird-shell: No such file");
        assert!(!redacted.contains("/usr/local"));
        assert!(redacted.contains("<path>"));
        assert!(redacted.contains("No such file"));
    }

    #[test]
    fn redaction_bounds_length() {
        assert!(redact(&"x".repeat(4096)).len() <= 512);
    }

    #[test]
    fn platform_matches_the_protocol_enum() {
        assert!(matches!(platform(), "windows-x64" | "ubuntu-x64"));
    }

    #[test]
    fn backoff_jitter_stays_in_range() {
        for _ in 0..100 {
            let jittered = with_jitter(Duration::from_secs(10));
            assert!(jittered >= Duration::from_secs(8));
            assert!(jittered <= Duration::from_secs(12));
        }
    }
}
