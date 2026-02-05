use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::LatticeRepo;
use latticefs_base::import::{ExportMode, export_ref, export_view};
use std::path::PathBuf;

use super::common::find_view_by_id;

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Object reference or view name/ID
    pub reference: String,
    /// Output path
    #[arg(long, short = 'o')]
    pub output: PathBuf,
    /// Export mode: tree or archive
    #[arg(long, default_value = "tree")]
    pub mode: String,
}

pub async fn run(repo: LatticeRepo, args: ExportArgs) -> Result<()> {
    let mode: ExportMode = args.mode.parse().map_err(|e: String| anyhow::anyhow!(e))?;

    if let Ok(uuid) = uuid::Uuid::parse_str(&args.reference) {
        if let Some(view) = find_view_by_id(&repo, &uuid)? {
            export_view(&repo, &view.id.to_string(), &args.output, mode)
                .await
                .with_context(|| format!("Failed to export view {}", view.id))?;
            println!("Exported view {} ({})", view.name, view.id);
            return Ok(());
        }
    }

    export_ref(&repo, &args.reference, &args.output, mode)
        .await
        .with_context(|| format!("Failed to export {}", args.reference))?;

    println!("Exported {}", args.reference);
    Ok(())
}
