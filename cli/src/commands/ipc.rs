use anyhow::Result;
use clap::Args;
use latticefs_base::ipc::start_ipc_server;
use latticefs_base::LatticeRepo;

#[derive(Args, Debug)]
pub struct IpcArgs {
}

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
