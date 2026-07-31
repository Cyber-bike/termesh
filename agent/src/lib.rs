//! termy-agent library surface.
//!
//! The binary is a thin CLI over these modules so the interesting behaviour -
//! path safety, PTY teardown, transfer bookkeeping - is reachable from tests
//! without spawning a process.

pub mod client;
pub mod config;
pub mod identity;
pub mod lock;
pub mod osc;
pub mod paths;
pub mod pty;
pub mod session_table;
pub mod state;
pub mod termstream;
pub mod transfer;

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("another termy-agent instance already holds {0}")]
    AlreadyRunning(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("pty error: {0}")]
    Pty(String),
    #[error("transfer error: {0}")]
    Transfer(String),
    #[error("SESSION_LIMIT_REACHED: at most {0} concurrent sessions")]
    SessionLimitReached(usize),
}
