use std::net::SocketAddr;
use std::process::ExitCode;

use clap::Parser;
use tracing::error;
use tracing_subscriber::EnvFilter;

mod relay;

/// Forward incoming `WoL` magic packets to a local broadcast address
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Required. Address+port the relay binds to
    #[arg(long)]
    listen: SocketAddr,

    /// Required. Broadcast address+port packets are sent to
    #[arg(long)]
    broadcast: SocketAddr,

    /// Run detached in the background with no attached console
    #[arg(long, alias = "background", alias = "detach")]
    daemon: bool,
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("{0}")]
    Args(#[from] clap::Error),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Daemon(#[from] daemon_forge::DaemonError),

    #[error(
        "listen and broadcast must use different ports (both use {0}); \
         using the same port causes the relayed packet to loop back to the listener"
    )]
    SamePort(u16),
}

fn run() -> Result<(), AppError> {
    let args = Args::try_parse()?;

    if args.listen.port() == args.broadcast.port() {
        return Err(AppError::SamePort(args.listen.port()));
    }

    if args.daemon {
        let daemon = daemon_forge::ForgeDaemon::new()
            .name("wakeonlan-relay")
            .stdout(daemon_forge::Stdio::devnull())
            .stderr(daemon_forge::Stdio::devnull())
            .privileged_action(move || {
                relay::run(args.listen, args.broadcast).map_err(daemon_forge::DaemonError::from)
            });
        daemon_forge::ForgeDaemon::start(daemon)?;
    } else {
        relay::run(args.listen, args.broadcast)?;
    }

    Ok(())
}

fn main() -> ExitCode {
    if let Err(e) = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .try_init()
    {
        eprintln!("warning: could not initialize tracing subscriber: {e}");
    }

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}
