use anyhow::Result;
use clap::Args;
use latticefs_base::LatticeRepo;
use latticefs_base::ipc::start_ipc_server;

#[derive(Args, Debug)]
pub struct IpcArgs {}

pub async fn run(repo: LatticeRepo, _args: IpcArgs) -> Result<()> {
    let mut config = repo.config.clone();
    let socket_path = config.socket_path();
    eprintln!("Starting IPC server...");
    eprintln!("Socket will be available at: {}", socket_path.display());
    eprintln!("Press Ctrl+C to stop the server\n");

    // Drop repo to release Sled lock before starting the server
    drop(repo);

    // Enable verbose output for CLI usage
    config.ipc.verbose = true;

    start_ipc_server(config).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Debug, Parser)]
    struct Cli {
        #[command(flatten)]
        args: IpcArgs,
    }

    #[test]
    fn parses_without_flags() {
        let cli = Cli::parse_from(["ipc"]);
        let _ = cli.args;
    }

    #[test]
    fn rejects_unknown_flag() {
        let err = Cli::try_parse_from(["ipc", "--verbose"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);
    }
}
