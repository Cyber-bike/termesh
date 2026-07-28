//! WSS gateway integration tests against a real bound server.
//!
//! These are the tests that actually exercise routing, ownership checks and the
//! connection-uniqueness rules. Everything runs over a real TCP socket with a
//! real WebSocket client, so handshake-level behaviour (subprotocol, auth
//! headers, close codes) is covered too.

use std::net::SocketAddr;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use uuid::Uuid;

use termy_relay::api::{router, AppState};
use termy_relay::config::Config;
use termy_relay::db::Db;

type Socket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

struct Server {
    addr: SocketAddr,
    state: AppState,
    _dir: tempfile::TempDir,
}

async fn start() -> Server {
    let dir = tempfile::tempdir().unwrap();
    let db = Db::connect(&dir.path().join("relay.db")).await.unwrap();

    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        database_path: dir.path().join("relay.db"),
        pepper: b"test-pepper-at-least-32-bytes-long!!".to_vec(),
        jwt_secret: b"test-jwt-secret-at-least-32-bytes!!!".to_vec(),
        relay_url: "wss://relay.test/v1/agent/ws".to_string(),
        access_token_ttl_secs: 900,
    };

    let state = AppState::new(db, config);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = router(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });

    Server { addr, state, _dir: dir }
}

impl Server {
    fn http(&self, path: &str) -> String {
        format!("http://{}{path}", self.addr)
    }

    async fn post(&self, path: &str, body: Value, token: Option<&str>) -> (u16, Value) {
        let client = reqwest_lite::Client;
        client.request("POST", &self.http(path), Some(body), token).await
    }

    async fn get(&self, path: &str, token: Option<&str>) -> (u16, Value) {
        reqwest_lite::Client.request("GET", &self.http(path), None, token).await
    }

    async fn seed_account(&self, login: &str) -> String {
        let digest = termy_relay::crypto::hash_password("hunter2hunter2").unwrap();
        self.state.db.create_user(login, &digest).await.unwrap();

        let (status, body) = self
            .post("/v1/auth/login", json!({"login": login, "password": "hunter2hunter2"}), None)
            .await;
        assert_eq!(status, 200, "login failed: {body}");
        body["accessToken"].as_str().unwrap().to_string()
    }

    async fn seed_device(&self, token: &str) -> (Uuid, String) {
        let (_, body) = self.post("/v1/devices/pairing-codes", json!({}), Some(token)).await;
        let code = body["pairingCode"].as_str().unwrap().to_string();

        let (status, body) = self
            .post(
                "/v1/devices/register",
                json!({"pairingCode": code, "deviceName": "box", "platform": "ubuntu-x64", "agentVersion": "1.0.0"}),
                None,
            )
            .await;
        assert_eq!(status, 201, "register failed: {body}");

        (
            Uuid::parse_str(body["deviceId"].as_str().unwrap()).unwrap(),
            body["deviceToken"].as_str().unwrap().to_string(),
        )
    }

    async fn connect_agent(&self, device_id: Uuid, device_token: &str) -> Socket {
        let mut request = format!("ws://{}/v1/agent/ws", self.addr).into_client_request().unwrap();
        request.headers_mut().insert(
            "authorization",
            format!("Device {device_id}.{device_token}").parse().unwrap(),
        );
        request
            .headers_mut()
            .insert("sec-websocket-protocol", "termy.v1".parse().unwrap());

        let (socket, _) = connect_async(request).await.expect("agent handshake failed");
        socket
    }

    async fn connect_control(&self, token: &str) -> Socket {
        let mut request =
            format!("ws://{}/v1/control/ws", self.addr).into_client_request().unwrap();
        request
            .headers_mut()
            .insert("authorization", format!("Bearer {token}").parse().unwrap());
        request
            .headers_mut()
            .insert("sec-websocket-protocol", "termy.v1".parse().unwrap());

        let (socket, _) = connect_async(request).await.expect("control handshake failed");
        socket
    }

    /// Brings an agent online and waits for its helloAck, so later assertions do
    /// not race the registration.
    async fn online_agent(&self, device_id: Uuid, token: &str) -> Socket {
        let mut agent = self.connect_agent(device_id, token).await;
        send_json(
            &mut agent,
            json!({
                "protocolVersion": 1,
                "type": "agent.hello",
                "requestId": Uuid::new_v4().to_string(),
                "deviceId": device_id.to_string(),
                "sessionId": null,
                "payload": {
                    "agentVersion": "1.0.0",
                    "platform": "ubuntu-x64",
                    "capabilities": ["terminal", "file-transfer"]
                }
            }),
        )
        .await;

        let ack = recv_json(&mut agent).await;
        assert_eq!(ack["type"], "agent.helloAck");
        assert_eq!(ack["payload"]["heartbeatIntervalMs"], 20000);
        agent
    }
}

/// A very small HTTP client so the tests do not pull in a full one.
mod reqwest_lite {
    use serde_json::Value;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    pub struct Client;

    impl Client {
        pub async fn request(
            &self,
            method: &str,
            url: &str,
            body: Option<Value>,
            token: Option<&str>,
        ) -> (u16, Value) {
            let rest = url.strip_prefix("http://").unwrap();
            let (authority, path) = rest.split_once('/').unwrap();
            let path = format!("/{path}");

            let mut stream = tokio::net::TcpStream::connect(authority).await.unwrap();

            let payload = body.map(|b| b.to_string()).unwrap_or_default();
            let mut request = format!(
                "{method} {path} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n"
            );
            if let Some(t) = token {
                request.push_str(&format!("Authorization: Bearer {t}\r\n"));
            }
            if !payload.is_empty() {
                request.push_str("Content-Type: application/json\r\n");
                request.push_str(&format!("Content-Length: {}\r\n", payload.len()));
            }
            request.push_str("\r\n");
            request.push_str(&payload);

            stream.write_all(request.as_bytes()).await.unwrap();

            let mut raw = Vec::new();
            stream.read_to_end(&mut raw).await.unwrap();
            let text = String::from_utf8_lossy(&raw).to_string();

            let status: u16 = text
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let body = text
                .split_once("\r\n\r\n")
                .map(|(_, b)| b)
                .unwrap_or("")
                .trim()
                .to_string();

            // Chunked responses are not produced by axum for these small bodies,
            // but strip a trailing chunk terminator just in case.
            let body = body.trim_end_matches("0\r\n\r\n").trim().to_string();
            let value = serde_json::from_str(&body).unwrap_or(Value::Null);
            (status, value)
        }
    }
}

async fn send_json(socket: &mut Socket, value: Value) {
    socket.send(Message::Text(value.to_string().into())).await.unwrap();
}

/// Reads the next JSON message, skipping pings and pongs the gateway may send.
async fn recv_json(socket: &mut Socket) -> Value {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(5), socket.next())
            .await
            .expect("timed out waiting for a message")
            .expect("socket closed")
            .expect("socket error");

        match message {
            Message::Text(text) => return serde_json::from_str(&text).unwrap(),
            Message::Ping(_) | Message::Pong(_) => continue,
            Message::Close(frame) => panic!("unexpected close: {frame:?}"),
            other => panic!("expected text, got {other:?}"),
        }
    }
}

async fn recv_binary(socket: &mut Socket) -> Vec<u8> {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(5), socket.next())
            .await
            .expect("timed out waiting for a frame")
            .expect("socket closed")
            .expect("socket error");

        match message {
            Message::Binary(bytes) => return bytes.to_vec(),
            Message::Ping(_) | Message::Pong(_) => continue,
            other => panic!("expected binary, got {other:?}"),
        }
    }
}

async fn recv_close(socket: &mut Socket) -> (u16, String) {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(5), socket.next())
            .await
            .expect("timed out waiting for close")
            .expect("socket closed without a close frame")
            .expect("socket error");

        match message {
            Message::Close(Some(frame)) => return (frame.code.into(), frame.reason.to_string()),
            Message::Ping(_) | Message::Pong(_) => continue,
            other => panic!("expected close, got {other:?}"),
        }
    }
}

fn frame_bytes(kind: u8, stream: Uuid, file_index: u32, offset: u64, payload: &[u8]) -> Vec<u8> {
    termy_protocol::frame::encode(&termy_protocol::frame::Frame {
        kind,
        stream_id: *stream.as_bytes(),
        file_index,
        offset,
        payload: payload.to_vec(),
    })
    .unwrap()
}

// --- handshake --------------------------------------------------------------

#[tokio::test]
async fn agent_handshake_requires_a_valid_device_credential() {
    let server = start().await;
    let token = server.seed_account("alice").await;
    let (device_id, device_token) = server.seed_device(&token).await;

    // Wrong token.
    let mut request =
        format!("ws://{}/v1/agent/ws", server.addr).into_client_request().unwrap();
    request.headers_mut().insert(
        "authorization",
        format!("Device {device_id}.{}", "x".repeat(43)).parse().unwrap(),
    );
    request.headers_mut().insert("sec-websocket-protocol", "termy.v1".parse().unwrap());
    assert!(connect_async(request).await.is_err(), "a bad device token must not upgrade");

    // Right token but somebody else's device id.
    let mut request =
        format!("ws://{}/v1/agent/ws", server.addr).into_client_request().unwrap();
    request.headers_mut().insert(
        "authorization",
        format!("Device {}.{device_token}", Uuid::new_v4()).parse().unwrap(),
    );
    request.headers_mut().insert("sec-websocket-protocol", "termy.v1".parse().unwrap());
    assert!(connect_async(request).await.is_err(), "device id and token must agree");
}

#[tokio::test]
async fn the_subprotocol_is_required() {
    let server = start().await;
    let token = server.seed_account("alice").await;

    let mut request =
        format!("ws://{}/v1/control/ws", server.addr).into_client_request().unwrap();
    request.headers_mut().insert("authorization", format!("Bearer {token}").parse().unwrap());
    request.headers_mut().remove("sec-websocket-protocol");

    assert!(connect_async(request).await.is_err(), "termy.v1 must be offered");
}

#[tokio::test]
async fn an_online_agent_shows_up_in_the_device_list() {
    let server = start().await;
    let token = server.seed_account("alice").await;
    let (device_id, device_token) = server.seed_device(&token).await;

    let (_, body) = server.get("/v1/devices", Some(&token)).await;
    assert_eq!(body["devices"][0]["online"], false);

    let _agent = server.online_agent(device_id, &device_token).await;

    let (_, body) = server.get("/v1/devices", Some(&token)).await;
    assert_eq!(body["devices"][0]["online"], true);
    assert!(body["devices"][0]["lastSeenAt"].is_string(), "hello should stamp lastSeenAt");
}

// --- terminal routing -------------------------------------------------------

#[tokio::test]
async fn a_terminal_session_routes_end_to_end() {
    let server = start().await;
    let token = server.seed_account("alice").await;
    let (device_id, device_token) = server.seed_device(&token).await;

    let mut agent = server.online_agent(device_id, &device_token).await;
    let mut control = server.connect_control(&token).await;

    // Plugin opens a terminal.
    let request_id = Uuid::new_v4();
    send_json(
        &mut control,
        json!({
            "protocolVersion": 1,
            "type": "terminal.open",
            "requestId": request_id.to_string(),
            "deviceId": device_id.to_string(),
            "sessionId": null,
            "payload": { "cols": 120, "rows": 30 }
        }),
    )
    .await;

    let forwarded = recv_json(&mut agent).await;
    assert_eq!(forwarded["type"], "terminal.open");
    assert_eq!(forwarded["requestId"], request_id.to_string());
    assert_eq!(forwarded["payload"]["cols"], 120);

    // Agent answers with the session it just created.
    let session_id = Uuid::new_v4();
    send_json(
        &mut agent,
        json!({
            "protocolVersion": 1,
            "type": "terminal.opened",
            "requestId": request_id.to_string(),
            "deviceId": device_id.to_string(),
            "sessionId": session_id.to_string(),
            "payload": { "shell": "/bin/bash" }
        }),
    )
    .await;

    let opened = recv_json(&mut control).await;
    assert_eq!(opened["type"], "terminal.opened");
    assert_eq!(opened["sessionId"], session_id.to_string());

    // Input travels plugin -> agent as a binary frame.
    let input = frame_bytes(
        termy_protocol::frame::KIND_TERMINAL_INPUT,
        session_id,
        termy_protocol::frame::TERMINAL_FILE_INDEX,
        0,
        b"ls -la\n",
    );
    control.send(Message::Binary(input.clone().into())).await.unwrap();
    assert_eq!(recv_binary(&mut agent).await, input);

    // Output travels agent -> plugin.
    let output = frame_bytes(
        termy_protocol::frame::KIND_TERMINAL_OUTPUT,
        session_id,
        termy_protocol::frame::TERMINAL_FILE_INDEX,
        0,
        b"total 0\r\n",
    );
    agent.send(Message::Binary(output.clone().into())).await.unwrap();
    assert_eq!(recv_binary(&mut control).await, output);

    // Shell integration events reach the plugin, which is what keeps cwd
    // tracking working in remote mode.
    send_json(
        &mut agent,
        json!({
            "protocolVersion": 1,
            "type": "terminal.shellEvent",
            "requestId": null,
            "deviceId": device_id.to_string(),
            "sessionId": session_id.to_string(),
            "payload": { "type": "command_end", "source": "osc133", "exitCode": 0 }
        }),
    )
    .await;
    let event = recv_json(&mut control).await;
    assert_eq!(event["type"], "terminal.shellEvent");
    assert_eq!(event["payload"]["type"], "command_end");
    assert_eq!(event["payload"]["source"], "osc133");
}

#[tokio::test]
async fn a_transfer_routes_end_to_end() {
    let server = start().await;
    let token = server.seed_account("alice").await;
    let (device_id, device_token) = server.seed_device(&token).await;

    let mut agent = server.online_agent(device_id, &device_token).await;
    let mut control = server.connect_control(&token).await;

    let transfer_id = Uuid::new_v4();
    send_json(
        &mut control,
        json!({
            "protocolVersion": 1,
            "type": "transfer.start",
            "requestId": Uuid::new_v4().to_string(),
            "deviceId": device_id.to_string(),
            "sessionId": null,
            "payload": {
                "transferId": transfer_id.to_string(),
                "rootNote": "notes/demo.md",
                "entries": [{ "index": 0, "relativePath": "notes/demo.md", "size": 11 }]
            }
        }),
    )
    .await;

    let started = recv_json(&mut agent).await;
    assert_eq!(started["type"], "transfer.start");

    send_json(
        &mut agent,
        json!({
            "protocolVersion": 1,
            "type": "transfer.accepted",
            "requestId": started["requestId"].clone(),
            "deviceId": device_id.to_string(),
            "sessionId": null,
            "payload": { "transferId": transfer_id.to_string(), "grantedBytes": 4194304 }
        }),
    )
    .await;
    assert_eq!(recv_json(&mut control).await["type"], "transfer.accepted");

    // One chunk, then the end-of-file marker and completion.
    let chunk =
        frame_bytes(termy_protocol::frame::KIND_FILE_CHUNK, transfer_id, 0, 0, b"hello world");
    control.send(Message::Binary(chunk.clone().into())).await.unwrap();
    assert_eq!(recv_binary(&mut agent).await, chunk);

    send_json(
        &mut control,
        json!({
            "protocolVersion": 1,
            "type": "transfer.fileEnd",
            "requestId": null,
            "deviceId": device_id.to_string(),
            "sessionId": null,
            "payload": { "transferId": transfer_id.to_string(), "fileIndex": 0, "sentSize": 11 }
        }),
    )
    .await;
    assert_eq!(recv_json(&mut agent).await["type"], "transfer.fileEnd");

    send_json(
        &mut agent,
        json!({
            "protocolVersion": 1,
            "type": "transfer.result",
            "requestId": null,
            "deviceId": device_id.to_string(),
            "sessionId": null,
            "payload": { "transferId": transfer_id.to_string(), "success": true, "code": null, "message": "" }
        }),
    )
    .await;
    let result = recv_json(&mut control).await;
    assert_eq!(result["type"], "transfer.result");
    assert_eq!(result["payload"]["success"], true);
}

// --- isolation and uniqueness -----------------------------------------------

#[tokio::test]
async fn another_account_cannot_open_a_terminal_on_your_device() {
    let server = start().await;
    let alice = server.seed_account("alice").await;
    let mallory = server.seed_account("mallory").await;
    let (device_id, device_token) = server.seed_device(&alice).await;

    let _agent = server.online_agent(device_id, &device_token).await;
    let mut intruder = server.connect_control(&mallory).await;

    send_json(
        &mut intruder,
        json!({
            "protocolVersion": 1,
            "type": "terminal.open",
            "requestId": Uuid::new_v4().to_string(),
            "deviceId": device_id.to_string(),
            "sessionId": null,
            "payload": { "cols": 80, "rows": 24 }
        }),
    )
    .await;

    let (code, reason) = recv_close(&mut intruder).await;
    assert_eq!(code, 4403, "doc 8.7 maps an ownership failure to 4403");
    assert_eq!(reason, "DEVICE_FORBIDDEN");
}

#[tokio::test]
async fn a_second_agent_connection_displaces_the_first() {
    let server = start().await;
    let token = server.seed_account("alice").await;
    let (device_id, device_token) = server.seed_device(&token).await;

    let mut first = server.online_agent(device_id, &device_token).await;
    let _second = server.online_agent(device_id, &device_token).await;

    let (code, reason) = recv_close(&mut first).await;
    assert_eq!(code, 4409);
    assert_eq!(reason, "AGENT_REPLACED");
}

#[tokio::test]
async fn a_second_control_connection_displaces_the_first() {
    let server = start().await;
    let token = server.seed_account("alice").await;

    let mut first = server.connect_control(&token).await;
    let _second = server.connect_control(&token).await;

    let (code, reason) = recv_close(&mut first).await;
    assert_eq!(code, 4409);
    assert_eq!(reason, "CONTROL_REPLACED");
}

#[tokio::test]
async fn malformed_messages_close_the_connection() {
    let server = start().await;
    let token = server.seed_account("alice").await;
    let mut control = server.connect_control(&token).await;

    // Wrong protocol version: rejected by the shared schema before routing.
    send_json(
        &mut control,
        json!({
            "protocolVersion": 2,
            "type": "terminal.open",
            "requestId": Uuid::new_v4().to_string(),
            "deviceId": Uuid::new_v4().to_string(),
            "sessionId": null,
            "payload": { "cols": 80, "rows": 24 }
        }),
    )
    .await;

    let (code, reason) = recv_close(&mut control).await;
    assert_eq!(code, 4400);
    assert_eq!(reason, "PROTOCOL_ERROR");
}

#[tokio::test]
async fn a_plugin_may_not_send_agent_messages() {
    let server = start().await;
    let token = server.seed_account("alice").await;
    let mut control = server.connect_control(&token).await;

    send_json(
        &mut control,
        json!({
            "protocolVersion": 1,
            "type": "agent.heartbeat",
            "requestId": null,
            "deviceId": Uuid::new_v4().to_string(),
            "sessionId": null,
            "payload": { "timestamp": "2026-07-28T09:00:00Z" }
        }),
    )
    .await;

    let (code, _) = recv_close(&mut control).await;
    assert_eq!(code, 4400, "role confusion is a protocol error");
}

#[tokio::test]
async fn an_agent_disconnect_tears_down_the_session() {
    let server = start().await;
    let token = server.seed_account("alice").await;
    let (device_id, device_token) = server.seed_device(&token).await;

    let mut agent = server.online_agent(device_id, &device_token).await;
    let mut control = server.connect_control(&token).await;

    let request_id = Uuid::new_v4();
    send_json(
        &mut control,
        json!({
            "protocolVersion": 1,
            "type": "terminal.open",
            "requestId": request_id.to_string(),
            "deviceId": device_id.to_string(),
            "sessionId": null,
            "payload": { "cols": 80, "rows": 24 }
        }),
    )
    .await;
    let _ = recv_json(&mut agent).await;

    let session_id = Uuid::new_v4();
    send_json(
        &mut agent,
        json!({
            "protocolVersion": 1,
            "type": "terminal.opened",
            "requestId": request_id.to_string(),
            "deviceId": device_id.to_string(),
            "sessionId": session_id.to_string(),
            "payload": { "shell": "/bin/bash" }
        }),
    )
    .await;
    let _ = recv_json(&mut control).await;

    // Agent vanishes.
    agent.close(None).await.unwrap();
    drop(agent);

    let closed = recv_json(&mut control).await;
    assert_eq!(closed["type"], "terminal.close");
    assert_eq!(closed["sessionId"], session_id.to_string());
    assert_eq!(closed["payload"]["reason"], "peer_disconnected");

    // And the device drops out of the online set.
    tokio::time::sleep(Duration::from_millis(100)).await;
    let (_, body) = server.get("/v1/devices", Some(&token)).await;
    assert_eq!(body["devices"][0]["online"], false);
}

#[tokio::test]
async fn input_for_an_unknown_session_is_refused() {
    let server = start().await;
    let token = server.seed_account("alice").await;
    let mut control = server.connect_control(&token).await;

    let stray = frame_bytes(
        termy_protocol::frame::KIND_TERMINAL_INPUT,
        Uuid::new_v4(),
        termy_protocol::frame::TERMINAL_FILE_INDEX,
        0,
        b"whoami\n",
    );
    control.send(Message::Binary(stray.into())).await.unwrap();

    let (code, _) = recv_close(&mut control).await;
    assert_eq!(code, 4403);
}

#[tokio::test]
async fn a_malformed_binary_frame_is_refused() {
    let server = start().await;
    let token = server.seed_account("alice").await;
    let mut control = server.connect_control(&token).await;

    control.send(Message::Binary(vec![0x00; 10].into())).await.unwrap();

    let (code, reason) = recv_close(&mut control).await;
    assert_eq!(code, 4400);
    assert_eq!(reason, "PROTOCOL_ERROR");
}
