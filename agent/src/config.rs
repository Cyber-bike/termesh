//! Agent configuration file (doc 7.3).
//!
//! Holds the device token, so the file is created 0600 inside a 0700 directory
//! on Unix. The token is never accepted from argv.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::AgentError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShellConfig {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "deviceToken")]
    pub device_token: String,
    #[serde(rename = "deviceName")]
    pub device_name: String,
    #[serde(rename = "relayUrl")]
    pub relay_url: String,
    #[serde(rename = "receiveRoot")]
    pub receive_root: PathBuf,
    pub shell: ShellConfig,
}

impl Config {
    pub fn default_shell() -> ShellConfig {
        if cfg!(windows) {
            ShellConfig { program: "powershell.exe".into(), args: vec![] }
        } else {
            ShellConfig { program: "/bin/bash".into(), args: vec![] }
        }
    }

    pub fn default_receive_root() -> PathBuf {
        home_dir().join("TermyReceive")
    }

    pub fn load(path: &Path) -> Result<Self, AgentError> {
        let raw = std::fs::read_to_string(path).map_err(|e| {
            AgentError::Config(format!(
                "cannot read {}: {e}. Run `termy-agent bind --code <pairing-code>` first",
                path.display()
            ))
        })?;

        let config: Config = serde_json::from_str(&raw)
            .map_err(|e| AgentError::Config(format!("{} is not valid: {e}", path.display())))?;

        config.validate()?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<(), AgentError> {
        self.validate()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
            harden_dir(parent)?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| AgentError::Config(format!("cannot serialise config: {e}")))?;

        // Write to a temp file and rename, so an interrupted save cannot leave a
        // half-written config that locks the agent out of its own device.
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json.as_bytes())?;
        harden_file(&tmp)?;
        std::fs::rename(&tmp, path)?;

        Ok(())
    }

    fn validate(&self) -> Result<(), AgentError> {
        if uuid::Uuid::parse_str(&self.device_id).is_err() {
            return Err(AgentError::Config("deviceId is not a UUID".into()));
        }
        if self.device_token.is_empty() {
            return Err(AgentError::Config("deviceToken is empty".into()));
        }
        if !self.relay_url.starts_with("wss://") {
            return Err(AgentError::Config(
                "relayUrl must start with wss://; the agent verifies certificates against \
                 webpki-roots and will not connect over ws://"
                    .into(),
            ));
        }
        if self.device_name.is_empty() || self.device_name.chars().count() > 64 {
            return Err(AgentError::Config("deviceName must be 1..64 characters".into()));
        }
        if !self.receive_root.is_absolute() {
            return Err(AgentError::Config("receiveRoot must be an absolute path".into()));
        }
        if self.shell.program.is_empty() {
            return Err(AgentError::Config("shell.program is empty".into()));
        }
        Ok(())
    }

    /// Doc 7.3: the receive root must exist (or be creatable) and be writable by
    /// the current user. Checked at startup rather than at first transfer so the
    /// failure shows up in `status`, not halfway through a file.
    pub fn ensure_receive_root(&self) -> Result<(), AgentError> {
        std::fs::create_dir_all(&self.receive_root).map_err(|e| {
            AgentError::Config(format!(
                "receiveRoot {} cannot be created: {e}",
                self.receive_root.display()
            ))
        })?;

        let probe = self.receive_root.join(".termy-write-probe");
        std::fs::write(&probe, b"").map_err(|e| {
            AgentError::Config(format!(
                "receiveRoot {} is not writable: {e}",
                self.receive_root.display()
            ))
        })?;
        let _ = std::fs::remove_file(&probe);
        Ok(())
    }
}

/// Doc 7.3: `%APPDATA%\TermyAgent` on Windows, `$XDG_CONFIG_HOME/termy-agent`
/// (defaulting to `~/.config/termy-agent`) elsewhere.
pub fn config_dir() -> PathBuf {
    if cfg!(windows) {
        match std::env::var_os("APPDATA") {
            Some(appdata) => PathBuf::from(appdata).join("TermyAgent"),
            None => home_dir().join("AppData").join("Roaming").join("TermyAgent"),
        }
    } else {
        match std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
            Some(base) => PathBuf::from(base).join("termy-agent"),
            None => home_dir().join(".config").join("termy-agent"),
        }
    }
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(unix)]
pub fn harden_dir(path: &Path) -> Result<(), AgentError> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    Ok(())
}

#[cfg(unix)]
pub fn harden_file(path: &Path) -> Result<(), AgentError> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}

/// On Windows the file inherits the user profile's ACL, which is already
/// restricted to the current user; there is no chmod equivalent worth emulating.
#[cfg(not(unix))]
pub fn harden_dir(_path: &Path) -> Result<(), AgentError> {
    Ok(())
}

#[cfg(not(unix))]
pub fn harden_file(_path: &Path) -> Result<(), AgentError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(root: &Path) -> Config {
        Config {
            device_id: "3d594650-3436-4c7a-9a15-9b5c3f0f4a11".into(),
            device_token: "a".repeat(43),
            device_name: "build-server".into(),
            relay_url: "wss://relay.example.com/v1/agent/ws".into(),
            receive_root: root.join("TermyReceive"),
            shell: Config::default_shell(),
        }
    }

    #[test]
    fn round_trips_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("config.json");
        let config = sample(dir.path());

        config.save(&path).unwrap();
        assert_eq!(Config::load(&path).unwrap(), config);
    }

    #[cfg(unix)]
    #[test]
    fn permissions_are_restricted() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("agent").join("config.json");
        sample(dir.path()).save(&path).unwrap();

        let file_mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        let dir_mode =
            std::fs::metadata(path.parent().unwrap()).unwrap().permissions().mode() & 0o777;

        assert_eq!(file_mode, 0o600, "config holds the device token");
        assert_eq!(dir_mode, 0o700);
    }

    #[test]
    fn rejects_a_plaintext_relay_url() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = sample(dir.path());
        config.relay_url = "ws://relay.example.com".into();
        assert!(config.save(&dir.path().join("c.json")).is_err());
    }

    #[test]
    fn rejects_a_relative_receive_root() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = sample(dir.path());
        config.receive_root = PathBuf::from("relative/dir");
        assert!(config.save(&dir.path().join("c.json")).is_err());
    }

    #[test]
    fn receive_root_is_created_and_probed() {
        let dir = tempfile::tempdir().unwrap();
        let config = sample(dir.path());
        config.ensure_receive_root().unwrap();
        assert!(config.receive_root.is_dir());
        // The probe file must not be left behind.
        assert!(!config.receive_root.join(".termy-write-probe").exists());
    }

    #[test]
    fn a_partial_write_cannot_corrupt_an_existing_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        sample(dir.path()).save(&path).unwrap();

        let mut broken = sample(dir.path());
        broken.relay_url = "ws://nope".into();
        assert!(broken.save(&path).is_err());

        // The original survives because validation runs before any write.
        assert_eq!(Config::load(&path).unwrap().relay_url, "wss://relay.example.com/v1/agent/ws");
    }
}
