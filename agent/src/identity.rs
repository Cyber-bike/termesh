//! Device identity (v2.0 doc 5.1): an Ed25519 keypair generated on first run
//! and persisted next to the agent config, replacing the V1 device token.
//!
//! This module only owns key material and its life cycle (generate, persist,
//! rotate). Turning the public key into an iroh `NodeId`/`NodeTicket` and
//! printing a connection code is the job of the not-yet-landed connection
//! module - kept separate so identity persistence has its own tests before the
//! iroh integration (doc 14 phase A0/A) is wired in and can be verified.

use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};

use crate::config::{config_dir, harden_dir, harden_file};
use crate::AgentError;

const SEED_LEN: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredIdentity {
    /// Base64 (standard, padded) encoding of the 32-byte Ed25519 seed. Never
    /// logged and never accepted from argv, same rule as the V1 device token
    /// (doc 7.3).
    seed: String,
}

pub struct DeviceIdentity {
    signing_key: SigningKey,
}

impl DeviceIdentity {
    pub fn generate() -> Self {
        Self {
            signing_key: SigningKey::generate(&mut rand::rngs::OsRng),
        }
    }

    fn from_seed(seed: [u8; SEED_LEN]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&seed),
        }
    }

    pub fn public_key_bytes(&self) -> [u8; SEED_LEN] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// Short hex prefix of the public key, for logs and `status` output only.
    /// This is not the connection code: the code is an iroh `NodeTicket`,
    /// produced once the networking layer encodes this key as an iroh
    /// `SecretKey` (doc 4 decision 3, doc 5.1).
    pub fn fingerprint(&self) -> String {
        self.public_key_bytes()
            .iter()
            .take(8)
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// Loads the persisted identity, generating and saving a new one on first
    /// run (doc 5.1: "首次运行生成身份").
    pub fn load_or_create(path: &Path) -> Result<Self, AgentError> {
        match std::fs::read_to_string(path) {
            Ok(raw) => {
                let stored: StoredIdentity = serde_json::from_str(&raw).map_err(|e| {
                    AgentError::Config(format!("{} is not valid: {e}", path.display()))
                })?;
                Self::decode(&stored)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let identity = Self::generate();
                identity.save(path)?;
                Ok(identity)
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Regenerates the identity in place (doc 5.3 `rotate-identity`). The old
    /// key is discarded rather than archived: doc 5.3 requires the old
    /// `NodeId` to stop being usable immediately, and keeping a copy around
    /// would only recreate the thing rotation exists to prevent.
    pub fn rotate(path: &Path) -> Result<Self, AgentError> {
        let identity = Self::generate();
        identity.save(path)?;
        Ok(identity)
    }

    fn save(&self, path: &Path) -> Result<(), AgentError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
            harden_dir(parent)?;
        }

        let stored = StoredIdentity {
            seed: BASE64.encode(self.signing_key.to_bytes()),
        };
        let json = serde_json::to_string_pretty(&stored)
            .map_err(|e| AgentError::Config(format!("cannot serialise identity: {e}")))?;

        // Atomic write: an interrupted save must never leave the agent with a
        // missing or half-written identity (same reasoning as Config::save).
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json.as_bytes())?;
        harden_file(&tmp)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    fn decode(stored: &StoredIdentity) -> Result<Self, AgentError> {
        let bytes = BASE64
            .decode(&stored.seed)
            .map_err(|e| AgentError::Config(format!("identity seed is not valid base64: {e}")))?;
        let seed: [u8; SEED_LEN] = bytes
            .try_into()
            .map_err(|_| AgentError::Config("identity seed is not 32 bytes".into()))?;
        Ok(Self::from_seed(seed))
    }
}

pub fn identity_path() -> PathBuf {
    config_dir().join("identity.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_run_generates_and_persists_an_identity() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("identity.json");

        assert!(!path.exists());
        let identity = DeviceIdentity::load_or_create(&path).unwrap();
        assert!(path.exists());

        let reloaded = DeviceIdentity::load_or_create(&path).unwrap();
        assert_eq!(identity.public_key_bytes(), reloaded.public_key_bytes());
    }

    #[test]
    fn rotate_replaces_the_public_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("identity.json");

        let original = DeviceIdentity::load_or_create(&path).unwrap();
        let rotated = DeviceIdentity::rotate(&path).unwrap();

        assert_ne!(original.public_key_bytes(), rotated.public_key_bytes());

        // The file on disk now backs the new key, not the old one.
        let reloaded = DeviceIdentity::load_or_create(&path).unwrap();
        assert_eq!(rotated.public_key_bytes(), reloaded.public_key_bytes());
    }

    #[test]
    fn a_corrupt_identity_file_is_rejected_rather_than_silently_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("identity.json");
        std::fs::write(&path, b"not json").unwrap();

        assert!(DeviceIdentity::load_or_create(&path).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn permissions_are_restricted() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("agent").join("identity.json");
        DeviceIdentity::load_or_create(&path).unwrap();

        let file_mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        let dir_mode = std::fs::metadata(path.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;

        assert_eq!(file_mode, 0o600, "identity file holds the private key seed");
        assert_eq!(dir_mode, 0o700);
    }

    #[test]
    fn the_identity_file_never_contains_the_word_token() {
        // Guards against someone later renaming `seed` back to something that
        // reads like the V1 device token in a log grep.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("identity.json");
        DeviceIdentity::load_or_create(&path).unwrap();

        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.to_lowercase().contains("token"));
    }

    #[test]
    fn fingerprint_is_stable_for_the_same_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("identity.json");
        let identity = DeviceIdentity::load_or_create(&path).unwrap();
        let reloaded = DeviceIdentity::load_or_create(&path).unwrap();

        assert_eq!(identity.fingerprint(), reloaded.fingerprint());
        assert_eq!(identity.fingerprint().len(), 16);
    }
}
