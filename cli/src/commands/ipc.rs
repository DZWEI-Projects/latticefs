use anyhow::Result;
use clap::Args;
use latticefs_base::ipc::start_ipc_server;
use latticefs_base::LatticeRepo;

#[derive(Args, Debug)]
pub struct IpcArgs {
}

pub async fn run(mut repo: LatticeRepo, _args: IpcArgs) -> Result<()> {
    let socket_path = latticefs_base::ipc::socket_path(&repo);
    eprintln!("Starting IPC server...");
    eprintln!("Socket will be available at: {}", socket_path.display());
    eprintln!("Press Ctrl+C to stop the server\n");
    
    // Enable verbose output for CLI usage
    repo.config.ipc.verbose = true;
    
    start_ipc_server(repo).await?;
    Ok(())
}
