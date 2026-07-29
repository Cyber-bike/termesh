//! HTTPS API integration tests.
//!
//! These drive the real router, so they cover the status codes and body shapes
//! `protocol/openapi.yaml` freezes, plus the account-isolation rules doc 11.2
//! requires. Account isolation gets its own tests on every route that takes an
//! id: it is the property MVP completion item 3 turns on.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use termy_relay::api::{router, AppState};
use termy_relay::config::Config;
use termy_relay::db::Db;

struct Harness {
    _dir: tempfile::TempDir,
    state: AppState,
}

async fn harness() -> Harness {
    harness_with_token_ttl(900).await
}

async fn harness_with_token_ttl(access_token_ttl_secs: i64) -> Harness {
    let dir = tempfile::tempdir().unwrap();
    let db = Db::connect(&dir.path().join("relay.db")).await.unwrap();

    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        database_path: dir.path().join("relay.db"),
        pepper: b"test-pepper-at-least-32-bytes-long!!".to_vec(),
        jwt_secret: b"test-jwt-secret-at-least-32-bytes!!!".to_vec(),
        relay_url: "wss://relay.test/v1/agent/ws".to_string(),
        access_token_ttl_secs,
    };

    Harness {
        _dir: dir,
        state: AppState::new(db, config),
    }
}

impl Harness {
    fn app(&self) -> axum::Router {
        router(self.state.clone())
    }

    async fn create_user(&self, login: &str, password: &str) {
        let digest = termy_relay::crypto::hash_password(password).unwrap();
        self.state.db.create_user(login, &digest).await.unwrap();
    }

    async fn token_for(&self, login: &str, password: &str) -> String {
        let (status, body) = self
            .post(
                "/v1/auth/login",
                json!({"login": login, "password": password}),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::OK, "login failed: {body}");
        body["accessToken"].as_str().unwrap().to_string()
    }

    async fn request(&self, req: Request<Body>) -> (StatusCode, Value) {
        let response = self.app().oneshot(req).await.unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };
        (status, value)
    }

    async fn post(&self, path: &str, body: Value, token: Option<&str>) -> (StatusCode, Value) {
        let mut req = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json");
        if let Some(t) = token {
            req = req.header("authorization", format!("Bearer {t}"));
        }
        self.request(req.body(Body::from(body.to_string())).unwrap())
            .await
    }

    async fn get(&self, path: &str, token: Option<&str>) -> (StatusCode, Value) {
        let mut req = Request::builder().method("GET").uri(path);
        if let Some(t) = token {
            req = req.header("authorization", format!("Bearer {t}"));
        }
        self.request(req.body(Body::empty()).unwrap()).await
    }

    async fn delete(&self, path: &str, token: Option<&str>) -> (StatusCode, Value) {
        let mut req = Request::builder().method("DELETE").uri(path);
        if let Some(t) = token {
            req = req.header("authorization", format!("Bearer {t}"));
        }
        self.request(req.body(Body::empty()).unwrap()).await
    }

    /// Creates a pairing code and registers a device with it.
    async fn register_device(&self, token: &str, name: &str) -> (String, String) {
        let (status, body) = self
            .post("/v1/devices/pairing-codes", json!({}), Some(token))
            .await;
        assert_eq!(status, StatusCode::CREATED);
        let code = body["pairingCode"].as_str().unwrap().to_string();

        let (status, body) = self
            .post(
                "/v1/devices/register",
                json!({
                    "pairingCode": code,
                    "deviceName": name,
                    "platform": "ubuntu-x64",
                    "agentVersion": "1.0.0"
                }),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "register failed: {body}");
        (
            body["deviceId"].as_str().unwrap().to_string(),
            body["deviceToken"].as_str().unwrap().to_string(),
        )
    }
}

// --- login ------------------------------------------------------------------

#[tokio::test]
async fn login_returns_a_usable_token() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;

    let (status, body) = h
        .post(
            "/v1/auth/login",
            json!({"login": "alice", "password": "hunter2hunter2"}),
            None,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["tokenType"], "Bearer");
    assert_eq!(body["expiresIn"], 900);
    assert_eq!(body["user"]["login"], "alice");
    assert!(body["accessToken"].as_str().unwrap().len() > 20);

    let token = body["accessToken"].as_str().unwrap();
    let (status, _) = h.get("/v1/devices", Some(token)).await;
    assert_eq!(status, StatusCode::OK);
}

/// The relay used to hard-code 900. Deployments now set the lifetime through
/// TERMY_ACCESS_TOKEN_TTL_SECS, and both the response field and the token's own
/// exp claim have to follow it - otherwise a client trusts expiresIn and keeps
/// using a token the relay has already stopped accepting, or discards one that
/// is still good.
#[tokio::test]
async fn login_reports_the_configured_token_lifetime() {
    let h = harness_with_token_ttl(43_200).await;
    h.create_user("alice", "hunter2hunter2").await;

    let (status, body) = h
        .post(
            "/v1/auth/login",
            json!({"login": "alice", "password": "hunter2hunter2"}),
            None,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["expiresIn"], 43_200);
}

#[tokio::test]
async fn login_rejects_bad_credentials_identically() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;

    let (wrong_pw, body_a) = h
        .post(
            "/v1/auth/login",
            json!({"login": "alice", "password": "wrongpassword"}),
            None,
        )
        .await;
    let (no_user, body_b) = h
        .post(
            "/v1/auth/login",
            json!({"login": "nobody", "password": "hunter2hunter2"}),
            None,
        )
        .await;

    assert_eq!(wrong_pw, StatusCode::UNAUTHORIZED);
    assert_eq!(no_user, StatusCode::UNAUTHORIZED);
    // The bodies must be indistinguishable, otherwise the response enumerates
    // which logins exist.
    assert_eq!(body_a["error"]["code"], body_b["error"]["code"]);
    assert_eq!(body_a["error"]["message"], body_b["error"]["message"]);
}

#[tokio::test]
async fn login_validates_field_bounds() {
    let h = harness().await;
    let (status, _) = h
        .post(
            "/v1/auth/login",
            json!({"login": "alice", "password": "short"}),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = h
        .post(
            "/v1/auth/login",
            json!({"login": "", "password": "hunter2hunter2"}),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn login_rejects_unknown_fields() {
    let h = harness().await;
    let (status, _) = h
        .post(
            "/v1/auth/login",
            json!({"login": "a", "password": "hunter2hunter2", "extra": 1}),
            None,
        )
        .await;
    // Doc 8.1 closes every object; serde's deny_unknown_fields turns that into
    // a deserialization failure, which axum reports as 422.
    assert!(
        status.is_client_error(),
        "unknown fields must be rejected, got {status}"
    );
}

#[tokio::test]
async fn login_is_rate_limited() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;

    for _ in 0..5 {
        let (status, _) = h
            .post(
                "/v1/auth/login",
                json!({"login": "alice", "password": "wrongpassword"}),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    let req = Request::builder()
        .method("POST")
        .uri("/v1/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"login": "alice", "password": "wrongpassword"}).to_string(),
        ))
        .unwrap();
    let response = h.app().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        response
            .headers()
            .get("retry-after")
            .unwrap()
            .to_str()
            .unwrap(),
        "60",
        "doc 6.2 requires Retry-After on 429"
    );
}

// --- auth guard -------------------------------------------------------------

#[tokio::test]
async fn protected_routes_require_a_bearer_token() {
    let h = harness().await;

    for (method, path) in [
        ("GET", "/v1/devices"),
        ("POST", "/v1/devices/pairing-codes"),
    ] {
        let (status, _) = if method == "GET" {
            h.get(path, None).await
        } else {
            h.post(path, json!({}), None).await
        };
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "{method} {path} must require auth"
        );
    }
}

#[tokio::test]
async fn a_forged_token_is_rejected() {
    let h = harness().await;
    let forged = termy_relay::auth::issue_access_token(
        b"a-completely-different-secret-32b!!!",
        uuid::Uuid::new_v4(),
        900,
    )
    .unwrap();

    let (status, _) = h.get("/v1/devices", Some(&forged)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// --- pairing codes ----------------------------------------------------------

#[tokio::test]
async fn pairing_code_is_created_with_the_documented_shape() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    let token = h.token_for("alice", "hunter2hunter2").await;

    let (status, body) = h
        .post("/v1/devices/pairing-codes", json!({}), Some(&token))
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["revoked"], false);
    let code = body["pairingCode"].as_str().unwrap();
    assert_eq!(code.len(), 27, "160 bits of entropy as unpadded Base64URL");
    assert!(code
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
}

/// The 16-code quota cannot be reached by looping over the endpoint: doc 6.5
/// also caps creation at 10 per hour, so the rate limiter fires first and an
/// account needs more than an hour to accumulate 16 outstanding codes. The quota
/// is still real, so it is seeded directly and then probed with a single call.
#[tokio::test]
async fn pairing_code_quota_is_enforced() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    let token = h.token_for("alice", "hunter2hunter2").await;

    let user = h
        .state
        .db
        .find_user_by_login("alice")
        .await
        .unwrap()
        .unwrap();
    for i in 0..16 {
        let digest =
            termy_relay::crypto::digest_secret(&h.state.config.pepper, &format!("seed-{i}"));
        h.state
            .db
            .create_pairing_code(user.id, &digest)
            .await
            .unwrap();
    }

    let (status, body) = h
        .post("/v1/devices/pairing-codes", json!({}), Some(&token))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "QUOTA_EXCEEDED");
}

/// Guards the interaction above so the numbers cannot drift into a state where
/// the quota becomes unreachable outright.
#[tokio::test]
async fn pairing_code_creation_is_rate_limited_before_the_quota() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    let token = h.token_for("alice", "hunter2hunter2").await;

    for _ in 0..10 {
        let (status, _) = h
            .post("/v1/devices/pairing-codes", json!({}), Some(&token))
            .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let (status, body) = h
        .post("/v1/devices/pairing-codes", json!({}), Some(&token))
        .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "RATE_LIMITED");
}

#[tokio::test]
async fn pairing_code_revocation() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    let token = h.token_for("alice", "hunter2hunter2").await;

    let (_, body) = h
        .post("/v1/devices/pairing-codes", json!({}), Some(&token))
        .await;
    let id = body["pairingCodeId"].as_str().unwrap();

    let (status, _) = h
        .delete(&format!("/v1/devices/pairing-codes/{id}"), Some(&token))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = h
        .delete(&format!("/v1/devices/pairing-codes/{id}"), Some(&token))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn another_account_cannot_revoke_a_pairing_code() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    h.create_user("mallory", "hunter2hunter2").await;
    let alice = h.token_for("alice", "hunter2hunter2").await;
    let mallory = h.token_for("mallory", "hunter2hunter2").await;

    let (_, body) = h
        .post("/v1/devices/pairing-codes", json!({}), Some(&alice))
        .await;
    let id = body["pairingCodeId"].as_str().unwrap();

    let (status, _) = h
        .delete(&format!("/v1/devices/pairing-codes/{id}"), Some(&mallory))
        .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "another account's code must not be confirmed to exist"
    );

    let (status, _) = h
        .delete(&format!("/v1/devices/pairing-codes/{id}"), Some(&alice))
        .await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "the owner can still revoke it"
    );
}

#[tokio::test]
async fn a_consumed_code_cannot_be_revoked() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    let token = h.token_for("alice", "hunter2hunter2").await;

    let (_, body) = h
        .post("/v1/devices/pairing-codes", json!({}), Some(&token))
        .await;
    let id = body["pairingCodeId"].as_str().unwrap().to_string();
    let code = body["pairingCode"].as_str().unwrap().to_string();

    let (status, _) = h
        .post(
            "/v1/devices/register",
            json!({"pairingCode": code, "deviceName": "box", "platform": "ubuntu-x64", "agentVersion": "1.0.0"}),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = h
        .delete(&format!("/v1/devices/pairing-codes/{id}"), Some(&token))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

// --- registration -----------------------------------------------------------

#[tokio::test]
async fn registration_returns_a_device_token_and_relay_url() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    let token = h.token_for("alice", "hunter2hunter2").await;

    let (_, body) = h
        .post("/v1/devices/pairing-codes", json!({}), Some(&token))
        .await;
    let code = body["pairingCode"].as_str().unwrap().to_string();

    let (status, body) = h
        .post(
            "/v1/devices/register",
            json!({"pairingCode": code, "deviceName": "build-server", "platform": "ubuntu-x64", "agentVersion": "1.2.3"}),
            None,
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        body["deviceToken"].as_str().unwrap().len(),
        43,
        "256-bit token"
    );
    assert_eq!(body["relayUrl"], "wss://relay.test/v1/agent/ws");
    assert!(uuid::Uuid::parse_str(body["deviceId"].as_str().unwrap()).is_ok());
}

#[tokio::test]
async fn registration_rejects_a_reused_code() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    let token = h.token_for("alice", "hunter2hunter2").await;

    let (_, body) = h
        .post("/v1/devices/pairing-codes", json!({}), Some(&token))
        .await;
    let code = body["pairingCode"].as_str().unwrap().to_string();

    let payload = json!({"pairingCode": code, "deviceName": "box", "platform": "ubuntu-x64", "agentVersion": "1.0.0"});

    let (first, _) = h.post("/v1/devices/register", payload.clone(), None).await;
    assert_eq!(first, StatusCode::CREATED);

    let (second, body) = h.post("/v1/devices/register", payload, None).await;
    assert_eq!(second, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "PAIRING_CODE_INVALID");
}

#[tokio::test]
async fn registration_validates_its_input() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    let token = h.token_for("alice", "hunter2hunter2").await;
    let (_, body) = h
        .post("/v1/devices/pairing-codes", json!({}), Some(&token))
        .await;
    let code = body["pairingCode"].as_str().unwrap().to_string();

    let cases = [
        (
            json!({"pairingCode": code, "deviceName": "", "platform": "ubuntu-x64", "agentVersion": "1.0.0"}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"pairingCode": code, "deviceName": "box", "platform": "macos-arm64", "agentVersion": "1.0.0"}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"pairingCode": code, "deviceName": "box", "platform": "ubuntu-x64", "agentVersion": "not-semver"}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"pairingCode": "too-short", "deviceName": "box", "platform": "ubuntu-x64", "agentVersion": "1.0.0"}),
            StatusCode::NOT_FOUND,
        ),
    ];

    for (payload, expected) in cases {
        let (status, _) = h.post("/v1/devices/register", payload.clone(), None).await;
        assert_eq!(
            status, expected,
            "payload {payload} should map to {expected}"
        );
    }

    // None of the rejections may have consumed the code.
    let (status, _) = h
        .post(
            "/v1/devices/register",
            json!({"pairingCode": code, "deviceName": "box", "platform": "ubuntu-x64", "agentVersion": "1.0.0"}),
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "a failed attempt must not consume the code"
    );
}

#[tokio::test]
async fn registration_does_not_accept_a_user_token_instead_of_a_code() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    let token = h.token_for("alice", "hunter2hunter2").await;

    // Doc 6.2: register takes the pairing code in the body and nothing else may
    // stand in for it.
    let (status, _) = h
        .post(
            "/v1/devices/register",
            json!({"pairingCode": token, "deviceName": "box", "platform": "ubuntu-x64", "agentVersion": "1.0.0"}),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// --- device list and unbind -------------------------------------------------

#[tokio::test]
async fn device_list_shows_only_your_own_devices() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    h.create_user("mallory", "hunter2hunter2").await;
    let alice = h.token_for("alice", "hunter2hunter2").await;
    let mallory = h.token_for("mallory", "hunter2hunter2").await;

    h.register_device(&alice, "alice-box").await;

    let (status, body) = h.get("/v1/devices", Some(&alice)).await;
    assert_eq!(status, StatusCode::OK);
    let devices = body["devices"].as_array().unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0]["name"], "alice-box");
    assert_eq!(devices[0]["platform"], "ubuntu-x64");
    assert_eq!(devices[0]["online"], false);
    assert!(devices[0]["lastSeenAt"].is_null());

    let (status, body) = h.get("/v1/devices", Some(&mallory)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["devices"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn unbinding_a_device() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    let alice = h.token_for("alice", "hunter2hunter2").await;
    let (device_id, _) = h.register_device(&alice, "box").await;

    let (status, _) = h
        .delete(&format!("/v1/devices/{device_id}"), Some(&alice))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = h.get("/v1/devices", Some(&alice)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["devices"].as_array().unwrap().is_empty());

    let (status, _) = h
        .delete(&format!("/v1/devices/{device_id}"), Some(&alice))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn another_account_cannot_unbind_your_device() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    h.create_user("mallory", "hunter2hunter2").await;
    let alice = h.token_for("alice", "hunter2hunter2").await;
    let mallory = h.token_for("mallory", "hunter2hunter2").await;

    let (device_id, _) = h.register_device(&alice, "box").await;

    let (status, body) = h
        .delete(&format!("/v1/devices/{device_id}"), Some(&mallory))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "DEVICE_FORBIDDEN");

    // And the device is still there.
    let (_, body) = h.get("/v1/devices", Some(&alice)).await;
    assert_eq!(body["devices"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn error_bodies_carry_a_request_id() {
    let h = harness().await;
    let (status, body) = h.get("/v1/devices", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body["error"]["code"].is_string());
    assert!(body["error"]["message"].is_string());
    assert!(
        uuid::Uuid::parse_str(body["error"]["requestId"].as_str().unwrap()).is_ok(),
        "requestId should be traceable in the server log"
    );
}

#[tokio::test]
async fn secrets_never_appear_in_responses_beyond_their_one_delivery() {
    let h = harness().await;
    h.create_user("alice", "hunter2hunter2").await;
    let token = h.token_for("alice", "hunter2hunter2").await;

    let (_, created) = h
        .post("/v1/devices/pairing-codes", json!({}), Some(&token))
        .await;
    let code = created["pairingCode"].as_str().unwrap().to_string();

    let (_, body) = h
        .post(
            "/v1/devices/register",
            json!({"pairingCode": code.clone(), "deviceName": "box", "platform": "ubuntu-x64", "agentVersion": "1.0.0"}),
            None,
        )
        .await;
    let device_token = body["deviceToken"].as_str().unwrap().to_string();

    // Neither secret may reappear in any later response (doc 6.3.5).
    let (_, list) = h.get("/v1/devices", Some(&token)).await;
    let serialised = list.to_string();
    assert!(
        !serialised.contains(&code),
        "pairing code leaked into the device list"
    );
    assert!(
        !serialised.contains(&device_token),
        "device token leaked into the device list"
    );
}

// Keeps the unused-import warning away when the module compiles without the
// Arc-based helpers.
const _: Option<Arc<()>> = None;
