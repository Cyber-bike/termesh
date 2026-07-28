//! HTTPS API surface, per doc 6.2.
//!
//! Route paths, status codes and body shapes are fixed by
//! `protocol/openapi.yaml`; `tests/api.rs` checks the handlers against it.

pub mod auth;
pub mod devices;

use std::sync::Arc;

use axum::routing::{delete, get, post};
use axum::Router;

use crate::config::Config;
use crate::db::Db;
use crate::gateway::registry::Registry;
use crate::ratelimit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub config: Arc<Config>,
    pub limiter: Arc<RateLimiter>,
    /// Live connections, sessions and transfer routes. Single node, in memory
    /// only (doc 11.1).
    pub registry: Arc<Registry>,
}

impl AppState {
    pub fn new(db: Db, config: Config) -> Self {
        Self {
            db,
            config: Arc::new(config),
            limiter: Arc::new(RateLimiter::new()),
            registry: Arc::new(Registry::new()),
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/auth/login", post(auth::login))
        .route("/v1/devices/pairing-codes", post(devices::create_pairing_code))
        .route("/v1/devices/pairing-codes/{id}", delete(devices::revoke_pairing_code))
        .route("/v1/devices/register", post(devices::register_device))
        .route("/v1/devices", get(devices::list_devices))
        .route("/v1/devices/{id}", delete(devices::delete_device))
        .route("/v1/control/ws", get(crate::gateway::control::control_ws))
        .route("/v1/agent/ws", get(crate::gateway::agent::agent_ws))
        .with_state(state)
}
