//! Integration tests for the SQLite layer.
//!
//! The queries are runtime-checked, so these tests are what catch a wrong column
//! name or a broken predicate. The atomic-consumption test in particular is not
//! a formality: doc 6.3.2 and 11.1 require that two agents racing on one pairing
//! code cannot both bind, and that guarantee lives entirely in one conditional
//! UPDATE inside a transaction.

use std::path::PathBuf;

use termy_relay::crypto;
use termy_relay::db::{Db, MAX_DEVICES_PER_USER};

const PEPPER: &[u8] = b"test-pepper-at-least-32-bytes-long!!";

struct TempDb {
    _dir: tempfile::TempDir,
    path: PathBuf,
}

fn temp_db_path() -> TempDb {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("relay.db");
    TempDb { _dir: dir, path }
}

async fn setup() -> (TempDb, Db) {
    let tmp = temp_db_path();
    let db = Db::connect(&tmp.path).await.unwrap();
    (tmp, db)
}

async fn seed_user(db: &Db, login: &str) -> uuid::Uuid {
    let digest = crypto::hash_password("a-good-password").unwrap();
    db.create_user(login, &digest).await.unwrap()
}

#[tokio::test]
async fn migrations_run_and_are_idempotent() {
    let tmp = temp_db_path();
    let db = Db::connect(&tmp.path).await.unwrap();
    drop(db);
    // Reconnecting runs the migrator again; it must be a no-op, not an error.
    let db = Db::connect(&tmp.path).await.unwrap();
    assert!(db.find_user_by_login("nobody").await.unwrap().is_none());
}

#[tokio::test]
async fn user_create_and_lookup() {
    let (_tmp, db) = setup().await;
    let id = seed_user(&db, "alice").await;

    let found = db.find_user_by_login("alice").await.unwrap().unwrap();
    assert_eq!(found.id, id);
    assert_eq!(found.login, "alice");
    assert!(crypto::verify_password(&found.password_digest, "a-good-password"));

    assert!(db.find_user_by_login("bob").await.unwrap().is_none());
}

#[tokio::test]
async fn duplicate_login_is_a_conflict() {
    let (_tmp, db) = setup().await;
    seed_user(&db, "alice").await;

    let digest = crypto::hash_password("another-password").unwrap();
    let err = db.create_user("alice", &digest).await.unwrap_err();
    assert_eq!(err.status, axum::http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn password_can_be_changed() {
    let (_tmp, db) = setup().await;
    seed_user(&db, "alice").await;

    let new_digest = crypto::hash_password("a-new-password").unwrap();
    assert!(db.set_password_digest("alice", &new_digest).await.unwrap());

    let found = db.find_user_by_login("alice").await.unwrap().unwrap();
    assert!(crypto::verify_password(&found.password_digest, "a-new-password"));
    assert!(!crypto::verify_password(&found.password_digest, "a-good-password"));

    assert!(!db.set_password_digest("nobody", &new_digest).await.unwrap());
}

#[tokio::test]
async fn pairing_code_lifecycle() {
    let (_tmp, db) = setup().await;
    let user_id = seed_user(&db, "alice").await;

    assert_eq!(db.count_unconsumed_pairing_codes(user_id).await.unwrap(), 0);

    let code = crypto::generate_pairing_code();
    let digest = crypto::digest_secret(PEPPER, &code);
    let (code_id, _created) = db.create_pairing_code(user_id, &digest).await.unwrap();

    assert_eq!(db.count_unconsumed_pairing_codes(user_id).await.unwrap(), 1);

    assert!(db.revoke_pairing_code(user_id, code_id).await.unwrap());
    assert_eq!(db.count_unconsumed_pairing_codes(user_id).await.unwrap(), 0);

    // A revoked code can no longer be used to register.
    let err = db
        .consume_pairing_code_and_create_device(&digest, "box", "ubuntu-x64", "1.0.0", b"tok")
        .await
        .unwrap_err();
    assert_eq!(err.status, axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn revoking_someone_elses_code_reports_not_found() {
    let (_tmp, db) = setup().await;
    let alice = seed_user(&db, "alice").await;
    let mallory = seed_user(&db, "mallory").await;

    let code = crypto::generate_pairing_code();
    let digest = crypto::digest_secret(PEPPER, &code);
    let (code_id, _) = db.create_pairing_code(alice, &digest).await.unwrap();

    // Not "forbidden": doc 6.2 maps another account's object to 404 so the
    // existence of the code is not disclosed.
    assert!(!db.revoke_pairing_code(mallory, code_id).await.unwrap());
    assert!(db.revoke_pairing_code(alice, code_id).await.unwrap());
}

#[tokio::test]
async fn consumed_code_cannot_be_revoked() {
    let (_tmp, db) = setup().await;
    let user_id = seed_user(&db, "alice").await;

    let code = crypto::generate_pairing_code();
    let digest = crypto::digest_secret(PEPPER, &code);
    let (code_id, _) = db.create_pairing_code(user_id, &digest).await.unwrap();

    db.consume_pairing_code_and_create_device(&digest, "box", "ubuntu-x64", "1.0.0", b"tok")
        .await
        .unwrap();

    let err = db.revoke_pairing_code(user_id, code_id).await.unwrap_err();
    assert_eq!(err.status, axum::http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn registration_consumes_the_code_exactly_once() {
    let (_tmp, db) = setup().await;
    let user_id = seed_user(&db, "alice").await;

    let code = crypto::generate_pairing_code();
    let digest = crypto::digest_secret(PEPPER, &code);
    db.create_pairing_code(user_id, &digest).await.unwrap();

    let (device_id, owner) = db
        .consume_pairing_code_and_create_device(
            &digest,
            "build-server",
            "ubuntu-x64",
            "1.0.0",
            b"token-digest-1",
        )
        .await
        .unwrap();
    assert_eq!(owner, user_id);
    assert!(db.device_exists(device_id).await.unwrap());

    let err = db
        .consume_pairing_code_and_create_device(
            &digest,
            "second",
            "ubuntu-x64",
            "1.0.0",
            b"token-digest-2",
        )
        .await
        .unwrap_err();
    assert_eq!(err.status, axum::http::StatusCode::NOT_FOUND);
}

/// Doc 6.3.2: consumption is atomic. Two agents racing on one code must produce
/// exactly one device.
#[tokio::test]
async fn concurrent_registration_binds_only_one_device() {
    let (_tmp, db) = setup().await;
    let user_id = seed_user(&db, "alice").await;

    let code = crypto::generate_pairing_code();
    let digest = crypto::digest_secret(PEPPER, &code);
    db.create_pairing_code(user_id, &digest).await.unwrap();

    let mut handles = Vec::new();
    for i in 0..8 {
        let db = db.clone();
        let digest = digest.clone();
        handles.push(tokio::spawn(async move {
            db.consume_pairing_code_and_create_device(
                &digest,
                &format!("racer-{i}"),
                "ubuntu-x64",
                "1.0.0",
                format!("token-digest-{i}").as_bytes(),
            )
            .await
        }));
    }

    let mut succeeded = 0;
    for handle in handles {
        if handle.await.unwrap().is_ok() {
            succeeded += 1;
        }
    }

    assert_eq!(succeeded, 1, "exactly one racer may consume the code");
    assert_eq!(db.list_devices(user_id).await.unwrap().len(), 1);
}

#[tokio::test]
async fn device_quota_is_enforced() {
    let (_tmp, db) = setup().await;
    let user_id = seed_user(&db, "alice").await;

    for i in 0..MAX_DEVICES_PER_USER {
        let code = crypto::generate_pairing_code();
        let digest = crypto::digest_secret(PEPPER, &code);
        db.create_pairing_code(user_id, &digest).await.unwrap();
        db.consume_pairing_code_and_create_device(
            &digest,
            &format!("device-{i}"),
            "ubuntu-x64",
            "1.0.0",
            format!("digest-{i}").as_bytes(),
        )
        .await
        .unwrap();
    }

    let code = crypto::generate_pairing_code();
    let digest = crypto::digest_secret(PEPPER, &code);
    db.create_pairing_code(user_id, &digest).await.unwrap();
    let err = db
        .consume_pairing_code_and_create_device(
            &digest,
            "one-too-many",
            "ubuntu-x64",
            "1.0.0",
            b"digest-overflow",
        )
        .await
        .unwrap_err();
    assert_eq!(err.status, axum::http::StatusCode::CONFLICT);

    // The rejected registration must not have consumed the code.
    assert_eq!(db.count_unconsumed_pairing_codes(user_id).await.unwrap(), 1);
}

#[tokio::test]
async fn devices_are_scoped_to_their_owner() {
    let (_tmp, db) = setup().await;
    let alice = seed_user(&db, "alice").await;
    let mallory = seed_user(&db, "mallory").await;

    let code = crypto::generate_pairing_code();
    let digest = crypto::digest_secret(PEPPER, &code);
    db.create_pairing_code(alice, &digest).await.unwrap();
    let (device_id, _) = db
        .consume_pairing_code_and_create_device(&digest, "box", "windows-x64", "1.0.0", b"tok")
        .await
        .unwrap();

    assert_eq!(db.list_devices(alice).await.unwrap().len(), 1);
    assert!(db.list_devices(mallory).await.unwrap().is_empty());

    assert!(!db.delete_device(mallory, device_id).await.unwrap());
    assert!(db.device_exists(device_id).await.unwrap());

    assert!(db.delete_device(alice, device_id).await.unwrap());
    assert!(!db.device_exists(device_id).await.unwrap());
}

#[tokio::test]
async fn device_lookup_by_token_digest() {
    let (_tmp, db) = setup().await;
    let user_id = seed_user(&db, "alice").await;

    let code = crypto::generate_pairing_code();
    let code_digest = crypto::digest_secret(PEPPER, &code);
    db.create_pairing_code(user_id, &code_digest).await.unwrap();

    let token = crypto::generate_device_token();
    let token_digest = crypto::digest_secret(PEPPER, &token);
    let (device_id, _) = db
        .consume_pairing_code_and_create_device(
            &code_digest,
            "box",
            "ubuntu-x64",
            "1.2.3",
            &token_digest,
        )
        .await
        .unwrap();

    let found = db
        .find_device_by_token_digest(&token_digest)
        .await
        .unwrap()
        .expect("device should be found by its token digest");
    assert_eq!(found.id, device_id);
    assert_eq!(found.agent_version, "1.2.3");
    assert!(found.last_seen_at.is_none());

    let wrong = crypto::digest_secret(PEPPER, "not-the-token");
    assert!(db.find_device_by_token_digest(&wrong).await.unwrap().is_none());
}

#[tokio::test]
async fn last_seen_is_recorded() {
    let (_tmp, db) = setup().await;
    let user_id = seed_user(&db, "alice").await;

    let code_digest = crypto::digest_secret(PEPPER, &crypto::generate_pairing_code());
    db.create_pairing_code(user_id, &code_digest).await.unwrap();
    let (device_id, _) = db
        .consume_pairing_code_and_create_device(&code_digest, "box", "ubuntu-x64", "1.0.0", b"tok")
        .await
        .unwrap();

    db.touch_last_seen(device_id).await.unwrap();

    let device = db.list_devices(user_id).await.unwrap().pop().unwrap();
    assert!(device.last_seen_at.is_some());
}

#[tokio::test]
async fn deleting_a_user_cascades_to_devices_and_codes() {
    let (_tmp, db) = setup().await;
    let user_id = seed_user(&db, "alice").await;

    let code_digest = crypto::digest_secret(PEPPER, &crypto::generate_pairing_code());
    db.create_pairing_code(user_id, &code_digest).await.unwrap();
    db.consume_pairing_code_and_create_device(&code_digest, "box", "ubuntu-x64", "1.0.0", b"tok")
        .await
        .unwrap();

    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .execute(db.pool())
        .await
        .unwrap();

    assert!(db.list_devices(user_id).await.unwrap().is_empty());
    assert_eq!(db.count_unconsumed_pairing_codes(user_id).await.unwrap(), 0);
}
