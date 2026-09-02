use std::net::SocketAddr;
use std::process::ExitCode;

use clap::Parser;

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

fn main() -> ExitCode {
    let args = match Args::try_parse() {
        Ok(a) => a,
        Err(e) => {
            let _ = e.print();
            let code = u8::try_from(e.exit_code()).unwrap_or(2);
            return ExitCode::from(code);
        }
    };

    let result: Result<(), Box<dyn std::error::Error + Send + Sync>> = if args.daemon {
        let daemon = daemon_forge::ForgeDaemon::new()
            .name("wakeonlan-relay")
            .stdout(daemon_forge::Stdio::devnull())
            .stderr(daemon_forge::Stdio::devnull())
            .privileged_action(move || {
                relay::run(args.listen, args.broadcast).map_err(daemon_forge::DaemonError::from)
            });
        daemon_forge::ForgeDaemon::start(daemon).map_err(to_boxed_error)
    } else {
        relay::run(args.listen, args.broadcast).map_err(to_boxed_error)
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn to_boxed_error(
    e: impl std::error::Error + Send + Sync + 'static,
) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(e)
}
