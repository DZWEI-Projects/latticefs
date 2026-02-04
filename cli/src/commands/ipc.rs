use anyhow::Result;
use clap::Args;
use latticefs_base::ipc::start_ipc_server;
use latticefs_base::LatticeRepo;

#[derive(Args, Debug)]
pub struct IpcArgs {
}

pub async fn run(repo: LatticeRepo, _args: IpcArgs) -> Result<()> {
    start_ipc_server(repo).await?;
    Ok(())
}
