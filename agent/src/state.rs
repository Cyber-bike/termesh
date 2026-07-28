//! Status file (doc 7.3).
//!
//! `termy-agent status` does not talk to the running process; it reads this
//! file. That avoids adding an IPC channel just to answer "am I connected?",
//! which on a headless box is the only question an operator has.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{config_dir, harden_file};
use crate::AgentError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub pid: u32,
    pub connection: ConnectionState,
    #[serde(rename = "lastConnectedAt")]
    pub last_connected_at: Option<String>,
    #[serde(rename = "lastDisconnectedAt")]
    pub last_disconnected_at: Option<String>,
    #[serde(rename = "sessionActive")]
    pub session_active: bool,
    /// Set when the relay rejected the device token, so `status` can say the
    /// agent needs re-binding rather than just "disconnected" forever.
    #[serde(rename = "needsRebind", default)]
    pub needs_rebind: bool,
}

impl AgentState {
    pub fn new() -> Self {
        Self {
            pid: std::process::id(),
            connection: ConnectionState::Connecting,
            last_connected_at: None,
            last_disconnected_at: None,
            session_active: false,
            needs_rebind: false,
        }
    }
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn state_path() -> PathBuf {
    config_dir().join("agent.state.json")
}

/// Writes atomically: `status` reading concurrently must never see a half file.
/// The state deliberately contains no device token (doc 7.3).
pub fn write(path: &Path, state: &AgentState) -> Result<(), AgentError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(state)
        .map_err(|e| AgentError::Config(format!("cannot serialise state: {e}")))?;

    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json.as_bytes())?;
    harden_file(&tmp)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn read(path: &Path) -> Option<AgentState> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("agent.state.json");

        let mut state = AgentState::new();
        state.connection = ConnectionState::Connected;
        state.last_connected_at = Some("2026-07-28T09:00:00Z".into());
        state.session_active = true;

        write(&path, &state).unwrap();

        let loaded = read(&path).unwrap();
        assert_eq!(loaded.connection, ConnectionState::Connected);
        assert!(loaded.session_active);
        assert_eq!(loaded.pid, std::process::id());
    }

    #[test]
    fn missing_or_corrupt_state_reads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read(&dir.path().join("absent.json")).is_none());

        let broken = dir.path().join("broken.json");
        std::fs::write(&broken, b"not json").unwrap();
        assert!(read(&broken).is_none());
    }

    #[test]
    fn the_state_file_never_contains_a_token() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("agent.state.json");
        write(&path, &AgentState::new()).unwrap();

        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.to_lowercase().contains("token"));
    }
}
