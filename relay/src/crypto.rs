//! Credential handling, per doc 6.3 and 13.
//!
//! Two different primitives on purpose:
//!   * user passwords are low-entropy and get Argon2id, which is deliberately slow;
//!   * pairing codes and device tokens are generated with 160/256 bits of entropy,
//!     so a keyed hash is enough and must stay fast - these are checked on every
//!     agent reconnect. The server pepper means a stolen database alone does not
//!     let an attacker verify guesses offline.
//!
//! Nothing here ever returns or logs a plaintext secret.

use argon2::password_hash::rand_core::OsRng as PwOsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use base64::Engine as _;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::error::AppError;

type HmacSha256 = Hmac<Sha256>;

/// Doc 6.3.5: 160 bits, encoded as 27 unpadded Base64URL characters.
const PAIRING_CODE_BYTES: usize = 20;
/// Doc 6.2: 256-bit device token, 43 unpadded Base64URL characters.
const DEVICE_TOKEN_BYTES: usize = 32;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut PwOsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::internal(format!("password hashing failed: {e}")))
}

/// Verifies a password against a stored PHC string.
///
/// A malformed stored hash is treated as "does not match" rather than an error:
/// it must not be distinguishable from a wrong password by the caller.
pub fn verify_password(stored_phc: &str, password: &str) -> bool {
    match PasswordHash::new(stored_phc) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

pub fn generate_pairing_code() -> String {
    random_base64url(PAIRING_CODE_BYTES)
}

pub fn generate_device_token() -> String {
    random_base64url(DEVICE_TOKEN_BYTES)
}

fn random_base64url(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&buf)
}

/// Keyed digest of a high-entropy secret. Deterministic, so it can be looked up
/// by an indexed equality query.
pub fn digest_secret(pepper: &[u8], secret: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(pepper).expect("HMAC accepts keys of any length");
    mac.update(secret.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

/// Constant-time digest comparison, for the paths where a candidate digest is
/// compared against one already in hand rather than looked up by the database.
pub fn digests_match(a: &[u8], b: &[u8]) -> bool {
    a.ct_eq(b).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_round_trip() {
        let phc = hash_password("correct horse battery staple").unwrap();
        assert!(verify_password(&phc, "correct horse battery staple"));
        assert!(!verify_password(&phc, "Correct horse battery staple"));
        assert!(!verify_password(&phc, ""));
    }

    #[test]
    fn password_hashes_are_salted() {
        let a = hash_password("same password").unwrap();
        let b = hash_password("same password").unwrap();
        assert_ne!(
            a, b,
            "identical passwords must not produce identical hashes"
        );
    }

    #[test]
    fn malformed_stored_hash_does_not_match() {
        assert!(!verify_password("not-a-phc-string", "anything"));
        assert!(!verify_password("", "anything"));
    }

    #[test]
    fn generated_secrets_have_the_documented_shape() {
        let code = generate_pairing_code();
        assert_eq!(code.len(), 27, "160 bits is 27 unpadded Base64URL chars");
        assert!(code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));

        let token = generate_device_token();
        assert_eq!(token.len(), 43, "256 bits is 43 unpadded Base64URL chars");
        assert!(token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn generated_secrets_do_not_repeat() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..256 {
            assert!(
                seen.insert(generate_pairing_code()),
                "pairing code repeated"
            );
        }
    }

    #[test]
    fn digest_is_deterministic_and_pepper_dependent() {
        let pepper_a = b"pepper-a-at-least-32-bytes-long!!";
        let pepper_b = b"pepper-b-at-least-32-bytes-long!!";

        let d1 = digest_secret(pepper_a, "secret");
        let d2 = digest_secret(pepper_a, "secret");
        let d3 = digest_secret(pepper_b, "secret");
        let d4 = digest_secret(pepper_a, "secreT");

        assert_eq!(d1, d2);
        assert_ne!(d1, d3, "a different pepper must produce a different digest");
        assert_ne!(d1, d4);
        assert_eq!(d1.len(), 32);
        assert!(digests_match(&d1, &d2));
        assert!(!digests_match(&d1, &d3));
    }
}
