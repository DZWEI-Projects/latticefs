use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::import::{export_object, export_ref, export_view, ExportMode};
use latticefs_base::LatticeRepo;
use std::path::PathBuf;

use super::common::{find_view_by_id, parse_ref_with_version};

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
    let mode: ExportMode = args
        .mode
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    // Check if reference contains a version specifier (e.g., @v1 or @<version-id>)
    if args.reference.contains('@') {
        let (object_id, version_id) = parse_ref_with_version(&repo, &args.reference)
            .with_context(|| format!("Failed to parse reference: {}", args.reference))?;
        export_object(&repo, &object_id, version_id, &args.output, mode)
            .await
            .with_context(|| format!("Failed to export {}", args.reference))?;
        println!("Exported {}", args.reference);
        return Ok(());
    }

    // Check if it's a view ID (UUID without version specifier)
    if let Ok(uuid) = uuid::Uuid::parse_str(&args.reference) {
        if let Some(view) = find_view_by_id(&repo, &uuid)? {
            // Export using the resolved view name. The export_view function requires
            // the view name to create appropriate directory structures and labels.
            export_view(&repo, &view.name, &args.output, mode)
                .await
                .with_context(|| format!("Failed to export view {}", view.name))?;
            println!("Exported view {} ({})", view.name, view.id);
            return Ok(());
        }
    }

    // Try exporting as object reference or view name
    export_ref(&repo, &args.reference, &args.output, mode)
        .await
        .with_context(|| format!("Failed to export {}", args.reference))?;

    println!("Exported {}", args.reference);
    Ok(())
}
