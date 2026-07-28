//! termy-relay: single-node cloud relay for the Termy remote MVP.
//!
//! Subcommands other than `serve` are the operator surface from doc 6.1. MVP
//! deliberately has no registration endpoint, so `useradd` is the only way an
//! account comes into existence.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use termy_relay::config::Config;
use termy_relay::crypto;
use termy_relay::db::Db;

#[derive(Parser)]
#[command(name = "termy-relay", version, about = "Termy remote MVP relay")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the HTTPS API and WSS gateway.
    Serve,
    /// Create an account. The password is read from stdin, never from argv.
    Useradd { login: String },
    /// Change an account password.
    Passwd { login: String },
    /// List the devices bound to an account.
    DeviceList { login: String },
    /// Probe a running relay over HTTP. Used as the container health check,
    /// where `--version` would only prove the binary loads.
    Healthcheck {
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: String,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "termy_relay=info,tower_http=info".into()),
        )
        .init();

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Runs before the config is read: a health probe must work without the
    // secrets the server itself needs.
    if let Command::Healthcheck { addr } = &cli.command {
        return healthcheck(addr).await;
    }

    let config = Config::from_env()?;
    let db = Db::connect(&config.database_path).await?;

    match cli.command {
        Command::Serve => serve(config, db).await,
        Command::Useradd { login } => useradd(&db, &login).await,
        Command::Passwd { login } => passwd(&db, &login).await,
        Command::DeviceList { login } => device_list(&db, &login).await,
        Command::Healthcheck { .. } => unreachable!("handled before the config is loaded"),
    }
}

/// Minimal GET /health. Deliberately not a full HTTP client: the slim runtime
/// image has no curl, and pulling one in for a liveness probe is not worth it.
async fn healthcheck(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut stream = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::net::TcpStream::connect(addr),
    )
    .await??;

    stream
        .write_all(
            format!("GET /health HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n").as_bytes(),
        )
        .await?;

    let mut response = Vec::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        stream.read_to_end(&mut response),
    )
    .await??;

    let text = String::from_utf8_lossy(&response);
    if text.starts_with("HTTP/1.1 200") {
        Ok(())
    } else {
        Err(format!(
            "relay is not healthy: {}",
            text.lines().next().unwrap_or("no response")
        )
        .into())
    }
}

async fn serve(config: Config, db: Db) -> Result<(), Box<dyn std::error::Error>> {
    let bind = config.bind;
    let state = termy_relay::api::AppState::new(db, config);
    let app = termy_relay::api::router(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(%bind, "termy-relay listening");

    // ConnectInfo makes the peer address available to the rate limiter when no
    // reverse proxy is in front (doc 6.5 keys several limits on source IP).
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}

async fn useradd(db: &Db, login: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_login(login)?;

    if db.find_user_by_login(login).await?.is_some() {
        return Err(format!("login {login} already exists").into());
    }

    let password = read_new_password()?;
    let digest = crypto::hash_password(&password)?;
    let id = db.create_user(login, &digest).await?;

    println!("created user {login} ({id})");
    Ok(())
}

async fn passwd(db: &Db, login: &str) -> Result<(), Box<dyn std::error::Error>> {
    if db.find_user_by_login(login).await?.is_none() {
        return Err(format!("no such user: {login}").into());
    }

    let password = read_new_password()?;
    let digest = crypto::hash_password(&password)?;
    if !db.set_password_digest(login, &digest).await? {
        return Err("password was not updated".into());
    }

    println!("password updated for {login}");
    Ok(())
}

async fn device_list(db: &Db, login: &str) -> Result<(), Box<dyn std::error::Error>> {
    let user = db
        .find_user_by_login(login)
        .await?
        .ok_or_else(|| format!("no such user: {login}"))?;

    let devices = db.list_devices(user.id).await?;
    if devices.is_empty() {
        println!("{login} has no devices");
        return Ok(());
    }

    println!("{:<38} {:<24} {:<14} LAST SEEN", "ID", "NAME", "PLATFORM");
    for device in devices {
        let last_seen = device
            .last_seen_at
            .map(|t| t.to_rfc3339())
            .unwrap_or_else(|| "never".into());
        println!(
            "{:<38} {:<24} {:<14} {}",
            device.id, device.name, device.platform, last_seen
        );
    }
    Ok(())
}

/// Doc 6.2 bounds the login at 1..254 characters. The character restriction is
/// ours: logins end up in log lines and CLI output, so control characters and
/// whitespace are refused rather than escaped everywhere downstream.
fn validate_login(login: &str) -> Result<(), String> {
    if login.is_empty() || login.chars().count() > 254 {
        return Err("login must be 1..254 characters".into());
    }
    if login.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("login must not contain whitespace or control characters".into());
    }
    Ok(())
}

/// Doc 6.2 bounds the password at 8..1024. Never echoed, never in argv.
///
/// Two modes, because `rpassword::prompt_password` opens /dev/tty and fails with
/// ENXIO where there is none - which is exactly where provisioning runs
/// (`docker exec` without -t, Ansible, a systemd unit). Interactively it prompts
/// twice with echo off; non-interactively it takes the first line of stdin, so
/// `echo 'pw' | termy-relay useradd alice` works. Confirmation is skipped in the
/// piped case since there is nobody to re-prompt.
fn read_new_password() -> Result<String, Box<dyn std::error::Error>> {
    use std::io::{BufRead, IsTerminal};

    let password = if std::io::stdin().is_terminal() {
        let first = rpassword::prompt_password("New password: ")?;
        let second = rpassword::prompt_password("Repeat password: ")?;
        if first != second {
            return Err("passwords do not match".into());
        }
        first
    } else {
        let mut buf = String::new();
        std::io::stdin().lock().read_line(&mut buf)?;
        buf.trim_end_matches(['\r', '\n']).to_string()
    };

    if password.len() < 8 || password.len() > 1024 {
        return Err("password must be 8..1024 bytes".into());
    }
    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::validate_login;

    #[test]
    fn login_rules() {
        assert!(validate_login("alice").is_ok());
        assert!(validate_login("alice@example.com").is_ok());
        assert!(validate_login("").is_err());
        assert!(validate_login("has space").is_err());
        assert!(validate_login("has\ttab").is_err());
        assert!(validate_login("has\nnewline").is_err());
        assert!(validate_login(&"x".repeat(255)).is_err());
        assert!(validate_login(&"x".repeat(254)).is_ok());
    }
}
