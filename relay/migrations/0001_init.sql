-- Initial schema, per doc 11.1.
--
-- Only users, pairing codes and devices are persisted. OnlineConnection,
-- TerminalSession and TransferRoute live in single-node memory and are
-- deliberately absent here.
--
-- Storage conventions:
--   * UUIDs are stored as lowercase hyphenated TEXT so the database stays
--     readable with the sqlite3 CLI during ops;
--   * timestamps are UTC RFC 3339 TEXT for the same reason;
--   * secret digests are 32-byte BLOBs (HMAC-SHA-256 with a server pepper).
--     No plaintext secret is ever stored.

CREATE TABLE users (
    id              TEXT PRIMARY KEY NOT NULL,
    login           TEXT NOT NULL UNIQUE,
    password_digest TEXT NOT NULL,
    created_at      TEXT NOT NULL
) STRICT;

CREATE TABLE pairing_codes (
    id          TEXT PRIMARY KEY NOT NULL,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_digest BLOB NOT NULL UNIQUE,
    created_at  TEXT NOT NULL,
    -- Doc 6.3: a code has no expiry. It is consumed exactly once, or revoked.
    consumed_at TEXT,
    revoked_at  TEXT
) STRICT;

CREATE INDEX idx_pairing_codes_user ON pairing_codes(user_id);

CREATE TABLE devices (
    id              TEXT PRIMARY KEY NOT NULL,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    platform        TEXT NOT NULL CHECK (platform IN ('windows-x64', 'ubuntu-x64')),
    agent_version   TEXT NOT NULL,
    token_digest    BLOB NOT NULL UNIQUE,
    created_at      TEXT NOT NULL,
    -- Doc 11.1: updated in memory on every heartbeat, flushed at most once a
    -- minute and on disconnect, so a 20 s heartbeat does not thrash SQLite.
    last_seen_at    TEXT,
    -- Which code bound this device. Kept for support questions; the code row is
    -- retained after consumption so this reference stays valid.
    pairing_code_id TEXT REFERENCES pairing_codes(id)
) STRICT;

CREATE INDEX idx_devices_user ON devices(user_id);
