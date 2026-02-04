use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::import::{export_ref, ExportMode};
use latticefs_base::LatticeRepo;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Object reference or view name
    pub reference: String,
    /// Output path
    #[arg(long, short = 'o')]
    pub output: PathBuf,
    /// Export mode: tree or archive
    #[arg(long, default_value = "tree")]
    pub mode: String,
}

pub async fn run(repo: LatticeRepo, args: ExportArgs) -> Result<()> {
    let mode: ExportMode = args
        .mode
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    export_ref(&repo, &args.reference, &args.output, mode)
        .await
        .with_context(|| format!("Failed to export {}", args.reference))?;

    println!("Exported {}", args.reference);
    Ok(())
}
