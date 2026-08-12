//! termy-agent: the target-side daemon of Termy's remote terminal (v2.0).
//!
//! No account, no pairing service: the agent's identity is a local Ed25519
//! keypair (doc 5.1) and pairing is "copy the connection code it prints"
//! (doc 5.2). Everything except `run` is a one-shot operator command.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use termy_agent::config::{self, Config};
use termy_agent::identity::DeviceIdentity;
use termy_agent::p2p::{self, EndpointProfile};
use termy_agent::serve::{self, ServeOptions};
use termy_agent::{lock, state};

#[derive(Parser)]
#[command(name = "termy-agent", version, about = "Termy remote terminal agent")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Inspect or change stored settings.
    #[command(subcommand)]
    Config(ConfigCommand),
    /// Print the connection code and serve control ends over iroh.
    Run {
        /// Bind to 127.0.0.1 only, with relays and address publishing
        /// disabled. For local development: nothing leaves this machine and
        /// nothing is announced to the discovery network.
        #[arg(long)]
        loopback: bool,
    },
    /// Print what the running agent last recorded.
    Status,
    /// Regenerate the device identity (doc 5.3). The previous connection
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
        Command::Config(command) => {
            let mut config = Config::load_or_default(&path)?;
            match command {
                ConfigCommand::Show => {
                    println!("deviceName             {}", config.device_name);
                    println!("identityKeyPath        {}", config.identity_key_path.display());
                    println!("receiveRoot            {}", config.receive_root.display());
                    println!("maxConcurrentSessions  {}", config.max_concurrent_sessions);
                    println!(
                        "shell                  {} {}",
                        config.shell.program,
                        config.shell.args.join(" ")
                    );
                    if !path.exists() {
                        println!();
                        println!("(all defaults; no config file at {})", path.display());
                    }
                    return Ok(());
                }
                ConfigCommand::SetName { name } => config.device_name = name,
                ConfigCommand::SetReceiveRoot { path } => {
                    config.receive_root = std::path::PathBuf::from(path)
                }
                ConfigCommand::SetShell { program, args } => {
                    config.shell = config::ShellConfig { program, args }
                }
            }
            config.save(&path)?;
            config.ensure_receive_root()?;
            println!("saved");
            Ok(())
        }

        Command::Run { loopback } => {
            // Doc 7.7: one agent per identity, otherwise two processes would
            // fight over the same EndpointId.
            let _guard = lock::acquire(&lock::lock_path())?;

            let config = Config::load_or_default(&path)?;
            config.ensure_receive_root()?;
            let identity = DeviceIdentity::load_or_create(&config.identity_key_path)?;

            let profile = if loopback {
                EndpointProfile::Loopback
            } else {
                EndpointProfile::Production
            };
            let endpoint = p2p::bind_endpoint(&identity, profile).await?;

            let code = if loopback {
                p2p::connection_code(p2p::loopback_addr(&endpoint))
            } else {
                // The code embeds the addresses a controller can dial, which
                // production endpoints learn from the relay handshake and
                // address watchers; give that a moment rather than printing
                // a code with no reachable address in it.
                if tokio::time::timeout(std::time::Duration::from_secs(20), endpoint.online())
                    .await
                    .is_err()
                {
                    eprintln!(
                        "warning: no relay reachable after 20s; the connection code may \
                         only contain local addresses"
                    );
                }
                p2p::connection_code(endpoint.addr())
            };

            println!("device    {} ({})", config.device_name, identity.fingerprint());
            println!();
            println!("connection code (paste it in Termy's \"添加设备\"):");
            println!();
            println!("  {code}");
            println!();
            println!("waiting for a controller; press Ctrl-C to stop");

            let mut agent_state = state::AgentState::new();
            agent_state.connection = state::ConnectionState::Connecting;
            agent_state.connection_code = Some(code);
            let _ = state::write(&state::state_path(), &agent_state);

            let options = ServeOptions {
                shell: config.shell.clone(),
                max_concurrent_sessions: config.max_concurrent_sessions,
                receive_root: config.receive_root.clone(),
            };

            tokio::select! {
                _ = serve::serve(endpoint.clone(), options) => {}
                _ = tokio::signal::ctrl_c() => {
                    println!("shutting down");
                    endpoint.close().await;
                }
            }

            agent_state.connection = state::ConnectionState::Disconnected;
            agent_state.connection_code = None;
            let _ = state::write(&state::state_path(), &agent_state);

            // Exit outright rather than unwinding: lingering teardown (a
            // stuck PTY reap, an unfinished background task) must not keep
            // a Ctrl-C'd agent alive - on Windows it did (2026-07-31 run).
            // State is written and the endpoint is closed; nothing beyond
            // this point matters.
            std::process::exit(0);
        }

        Command::Status => {
            let config = Config::load_or_default(&path)?;

            match DeviceIdentity::load_or_create(&config.identity_key_path) {
                Ok(identity) => println!("identity   {}", identity.fingerprint()),
                Err(err) => println!("identity   error: {err}"),
            }

            let state_file = state::state_path();
            match state::read(&state_file) {
                None => {
                    println!("agent      never run (no state at {})", state_file.display());
                }
                Some(state) => {
                    let running = process_alive(state.pid);
                    println!(
                        "agent      pid {} ({})",
                        state.pid,
                        if running { "running" } else { "not running" }
                    );
                    match (running, state.connection_code) {
                        (true, Some(code)) => {
                            println!("code       {code}");
                        }
                        (true, None) => {
                            println!("code       unavailable (agent started before it was recorded)");
                        }
                        (false, _) => {
                            println!("code       none (start the agent with `termy-agent run`)");
                        }
                    }
                }
            }
            Ok(())
        }

        Command::RotateIdentity { yes } => {
            let config = Config::load_or_default(&path)?;
            let identity_path = &config.identity_key_path;
            let previous = DeviceIdentity::load_or_create(identity_path)?;

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

            let rotated = DeviceIdentity::rotate(identity_path)?;
            println!(
                "identity rotated: {} -> {}",
                previous.fingerprint(),
                rotated.fingerprint()
            );
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

#[cfg(unix)]
fn process_alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

/// The stub that used to sit here always answered `false`, so on Windows
/// `status` reported a running agent as "not running" and hid its
/// connection code (2026-07-31 acceptance run).
#[cfg(windows)]
fn process_alive(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, STILL_ACTIVE};
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }
        let mut exit_code: u32 = 0;
        let ok = GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle);
        ok != 0 && exit_code == STILL_ACTIVE as u32
    }
}

#[cfg(not(any(unix, windows)))]
fn process_alive(_pid: u32) -> bool {
    false
}
