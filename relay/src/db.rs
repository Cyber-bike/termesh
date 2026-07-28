//! SQLite access layer.
//!
//! Queries are runtime-checked rather than using the `query!` macros: the macros
//! need a live database at compile time, which would make the build depend on
//! developer-local state. The trade-off is that column mistakes surface in the
//! integration tests rather than at compile time, so every query below has one.

use std::path::Path;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, StartupError};

/// Doc 6.5 quotas.
pub const MAX_DEVICES_PER_USER: i64 = 32;
pub const MAX_UNCONSUMED_PAIRING_CODES: i64 = 16;

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

pub struct User {
    pub id: Uuid,
    pub login: String,
    pub password_digest: String,
}

pub struct Device {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub platform: String,
    pub agent_version: String,
    pub last_seen_at: Option<DateTime<Utc>>,
}

impl Db {
    pub async fn connect(path: &Path) -> Result<Self, StartupError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            // WAL per doc 6.1; NORMAL is the standard companion - with WAL it is
            // durable across process crashes, only a host crash can lose the
            // last commits.
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_secs(5));

        // Single node, single process: a small pool is plenty and keeps SQLite
        // writer contention predictable.
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    // --- users --------------------------------------------------------------

    pub async fn create_user(&self, login: &str, password_digest: &str) -> Result<Uuid, AppError> {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, login, password_digest, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(login)
        .bind(password_digest)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db) if db.is_unique_violation() => {
                AppError::conflict(format!("login {login} already exists"))
            }
            _ => AppError::from(e),
        })?;

        Ok(id)
    }

    pub async fn find_user_by_login(&self, login: &str) -> Result<Option<User>, AppError> {
        let row: Option<(String, String, String)> =
            sqlx::query_as("SELECT id, login, password_digest FROM users WHERE login = ?")
                .bind(login)
                .fetch_optional(&self.pool)
                .await?;

        row.map(|(id, login, password_digest)| {
            Ok(User { id: parse_uuid(&id)?, login, password_digest })
        })
        .transpose()
    }

    pub async fn set_password_digest(&self, login: &str, digest: &str) -> Result<bool, AppError> {
        let result = sqlx::query("UPDATE users SET password_digest = ? WHERE login = ?")
            .bind(digest)
            .bind(login)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() == 1)
    }

    // --- pairing codes ------------------------------------------------------

    pub async fn count_unconsumed_pairing_codes(&self, user_id: Uuid) -> Result<i64, AppError> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM pairing_codes \
             WHERE user_id = ? AND consumed_at IS NULL AND revoked_at IS NULL",
        )
        .bind(user_id.to_string())
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    pub async fn create_pairing_code(
        &self,
        user_id: Uuid,
        code_digest: &[u8],
    ) -> Result<(Uuid, DateTime<Utc>), AppError> {
        let id = Uuid::new_v4();
        let created_at = Utc::now();

        sqlx::query(
            "INSERT INTO pairing_codes (id, user_id, code_digest, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .bind(code_digest)
        .bind(created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok((id, created_at))
    }

    /// Returns Ok(true) when a code was revoked, Ok(false) when it does not
    /// exist or belongs to another account, and Err(conflict) when it has
    /// already been consumed - doc 6.2 maps those to 404 and 409 respectively.
    pub async fn revoke_pairing_code(&self, user_id: Uuid, id: Uuid) -> Result<bool, AppError> {
        let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT consumed_at, revoked_at FROM pairing_codes WHERE id = ? AND user_id = ?",
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        let Some((consumed_at, _revoked_at)) = row else {
            return Ok(false);
        };
        if consumed_at.is_some() {
            return Err(AppError::conflict("Pairing code has already been consumed"));
        }

        let result = sqlx::query(
            "UPDATE pairing_codes SET revoked_at = ? \
             WHERE id = ? AND user_id = ? AND consumed_at IS NULL AND revoked_at IS NULL",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    /// Atomic consumption, per doc 6.3.2 and 11.1.
    ///
    /// The whole exchange runs in one transaction and the UPDATE carries the
    /// `consumed_at IS NULL AND revoked_at IS NULL` predicate, so two agents
    /// racing on the same code cannot both bind: exactly one UPDATE reports a
    /// changed row and the loser rolls back.
    pub async fn consume_pairing_code_and_create_device(
        &self,
        code_digest: &[u8],
        name: &str,
        platform: &str,
        agent_version: &str,
        token_digest: &[u8],
    ) -> Result<(Uuid, Uuid), AppError> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now().to_rfc3339();

        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT id, user_id FROM pairing_codes \
             WHERE code_digest = ? AND consumed_at IS NULL AND revoked_at IS NULL",
        )
        .bind(code_digest)
        .fetch_optional(&mut *tx)
        .await?;

        let Some((code_id, user_id)) = row else {
            return Err(AppError::not_found("Pairing code is invalid"));
        };

        let updated = sqlx::query(
            "UPDATE pairing_codes SET consumed_at = ? \
             WHERE id = ? AND consumed_at IS NULL AND revoked_at IS NULL",
        )
        .bind(&now)
        .bind(&code_id)
        .execute(&mut *tx)
        .await?;

        if updated.rows_affected() != 1 {
            // Another registration won the race between the SELECT and here.
            return Err(AppError::not_found("Pairing code is invalid"));
        }

        let user_uuid = parse_uuid(&user_id)?;
        let (device_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM devices WHERE user_id = ?")
                .bind(&user_id)
                .fetch_one(&mut *tx)
                .await?;

        if device_count >= MAX_DEVICES_PER_USER {
            return Err(AppError::conflict(format!(
                "Account already has the maximum of {MAX_DEVICES_PER_USER} devices"
            )));
        }

        let device_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO devices \
             (id, user_id, name, platform, agent_version, token_digest, created_at, pairing_code_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(device_id.to_string())
        .bind(&user_id)
        .bind(name)
        .bind(platform)
        .bind(agent_version)
        .bind(token_digest)
        .bind(&now)
        .bind(&code_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok((device_id, user_uuid))
    }

    // --- devices ------------------------------------------------------------

    pub async fn list_devices(&self, user_id: Uuid) -> Result<Vec<Device>, AppError> {
        let rows: Vec<(String, String, String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT id, user_id, name, platform, agent_version, last_seen_at \
             FROM devices WHERE user_id = ? ORDER BY created_at",
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|(id, user_id, name, platform, agent_version, last_seen_at)| {
                Ok(Device {
                    id: parse_uuid(&id)?,
                    user_id: parse_uuid(&user_id)?,
                    name,
                    platform,
                    agent_version,
                    last_seen_at: last_seen_at.as_deref().map(parse_timestamp).transpose()?,
                })
            })
            .collect()
    }

    pub async fn find_device_by_token_digest(
        &self,
        token_digest: &[u8],
    ) -> Result<Option<Device>, AppError> {
        let row: Option<(String, String, String, String, String, Option<String>)> =
            sqlx::query_as(
                "SELECT id, user_id, name, platform, agent_version, last_seen_at \
                 FROM devices WHERE token_digest = ?",
            )
            .bind(token_digest)
            .fetch_optional(&self.pool)
            .await?;

        row.map(|(id, user_id, name, platform, agent_version, last_seen_at)| {
            Ok(Device {
                id: parse_uuid(&id)?,
                user_id: parse_uuid(&user_id)?,
                name,
                platform,
                agent_version,
                last_seen_at: last_seen_at.as_deref().map(parse_timestamp).transpose()?,
            })
        })
        .transpose()
    }

    pub async fn delete_device(&self, user_id: Uuid, device_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM devices WHERE id = ? AND user_id = ?")
            .bind(device_id.to_string())
            .bind(user_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn device_exists(&self, device_id: Uuid) -> Result<bool, AppError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT id FROM devices WHERE id = ?")
            .bind(device_id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.is_some())
    }

    pub async fn touch_last_seen(&self, device_id: Uuid) -> Result<(), AppError> {
        sqlx::query("UPDATE devices SET last_seen_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(device_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

fn parse_uuid(raw: &str) -> Result<Uuid, AppError> {
    Uuid::from_str(raw).map_err(|e| AppError::internal(format!("stored UUID is malformed: {e}")))
}

fn parse_timestamp(raw: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| AppError::internal(format!("stored timestamp is malformed: {e}")))
}
