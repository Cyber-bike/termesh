//! termy-agent: outbound agent for the Termy remote MVP.
//!
//! Everything except `run` is a one-shot operator command. The device token is
//! never accepted as an argument (doc 7.3).

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use termy_agent::client;
use termy_agent::config::{self, Config};
use termy_agent::identity::{self, DeviceIdentity};
use termy_agent::{lock, state};

#[derive(Parser)]
#[command(name = "termy-agent", version, about = "Termy remote MVP agent")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Consume a pairing code and bind this machine to the account.
    Bind {
        #[arg(long)]
        code: String,
        /// Relay base URL, e.g. https://relay.example.com
        #[arg(long)]
        relay: String,
        /// Defaults to the system hostname.
        #[arg(long)]
        name: Option<String>,
    },
    /// Inspect or change stored settings.
    #[command(subcommand)]
    Config(ConfigCommand),
    /// Connect to the relay and serve terminal and file requests.
    Run,
    /// Print what the running agent last recorded.
    Status,
    /// Regenerate the device identity (v2.0 doc 5.3). The previous connection
    /// code stops working immediately; every control end must re-pair with
    /// the new code.
    RotateIdentity {
        /// Skip the interactive confirmation prompt.
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum ConfigCommand {
    Show,
    SetName { name: String },
    SetReceiveRoot { path: String },
    SetShell { program: String, args: Vec<String> },
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "termy_agent=info".into()),
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
    let path = config::config_path();

    match cli.command {
        Command::Bind { code, relay, name } => {
            let device_name = name.unwrap_or_else(default_device_name);
            let config = client::bind(&relay, &code, &device_name).await?;
            config.save(&path)?;
            config.ensure_receive_root()?;

            println!("bound as device {} ({device_name})", config.device_id);
            println!("config written to {}", path.display());
            println!("receive root: {}", config.receive_root.display());
            println!("next: start the agent with `termy-agent run`");
            Ok(())
        }

        Command::Config(command) => {
            let mut config = Config::load(&path)?;
            match command {
                ConfigCommand::Show => {
                    // Never print the token.
                    println!("deviceId     {}", config.device_id);
                    println!("deviceName   {}", config.device_name);
                    println!("relayUrl     {}", config.relay_url);
                    println!("receiveRoot  {}", config.receive_root.display());
                    println!(
                        "shell        {} {}",
                        config.shell.program,
                        config.shell.args.join(" ")
                    );
                    return Ok(());
                }
                ConfigCommand::SetName { name } => config.device_name = name,
                ConfigCommand::SetReceiveRoot { path } => {
                    config.receive_root = std::path::PathBuf::from(path)
                }
                ConfigCommand::SetShell { program, args } => {
                    config.shell = termy_agent::config::ShellConfig { program, args }
                }
            }
            config.save(&path)?;
            config.ensure_receive_root()?;
            println!("saved");
            Ok(())
        }

        Command::Run => {
            // Doc 7.6: refuse to start a second instance, otherwise the two take
            // turns evicting each other from the relay.
            let _guard = lock::acquire(&lock::lock_path())?;

            let config = Config::load(&path)?;
            config.ensure_receive_root()?;

            tracing::info!(
                device = %config.device_name,
                receive_root = %config.receive_root.display(),
                "starting termy-agent"
            );
            client::run(config).await?;
            Ok(())
        }

        Command::Status => {
            let state_file = state::state_path();
            match state::read(&state_file) {
                None => {
                    println!(
                        "no state file at {}; the agent has not run yet",
                        state_file.display()
                    );
                }
                Some(state) => {
                    let running = process_alive(state.pid);
                    println!(
                        "pid           {} ({})",
                        state.pid,
                        if running { "running" } else { "not running" }
                    );
                    println!("connection    {:?}", state.connection);
                    println!(
                        "last connect  {}",
                        state.last_connected_at.unwrap_or_else(|| "never".into())
                    );
                    println!(
                        "last drop     {}",
                        state.last_disconnected_at.unwrap_or_else(|| "never".into())
                    );
                    println!(
                        "session       {}",
                        if state.session_active {
                            "active"
                        } else {
                            "idle"
                        }
                    );
                    if state.needs_rebind {
                        println!();
                        println!(
                            "the relay rejected this device token; run `termy-agent bind` again"
                        );
                    }
                }
            }

            match DeviceIdentity::load_or_create(&identity::identity_path()) {
                Ok(identity) => println!("identity       {}", identity.fingerprint()),
                Err(err) => println!("identity       error: {err}"),
            }
            Ok(())
        }

        Command::RotateIdentity { yes } => {
            let path = identity::identity_path();
            let previous = DeviceIdentity::load_or_create(&path)?;

            if !yes {
                println!(
                    "this invalidates the current connection code (identity {}); \
                     every control end must re-pair with the new code.",
                    previous.fingerprint()
                );
                if !confirm("continue? [y/N] ")? {
                    println!("aborted");
                    return Ok(());
                }
            }

            let rotated = DeviceIdentity::rotate(&path)?;
            println!("identity rotated: {} -> {}", previous.fingerprint(), rotated.fingerprint());
            println!("start the agent to print the new connection code: `termy-agent run`");
            Ok(())
        }
    }
}

/// Reads a single line from stdin and treats `y`/`yes` (case-insensitive) as
/// confirmation. Anything else, including EOF, is a decline - rotation must
/// never proceed on an ambiguous answer.
fn confirm(prompt: &str) -> std::io::Result<bool> {
    use std::io::Write;

    print!("{prompt}");
    std::io::stdout().flush()?;

    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    Ok(matches!(line.trim().to_lowercase().as_str(), "y" | "yes"))
}

fn default_device_name() -> String {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .unwrap_or_else(|| "termy-agent".into())
}

#[cfg(unix)]
fn process_alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

#[cfg(not(unix))]
fn process_alive(_pid: u32) -> bool {
    false
}
