use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::LatticeRepo;
use std::io::{self, Write};

use super::common::parse_ref_with_version;
use latticefs_base::is_quarantined_executable;

#[derive(Args, Debug)]
pub struct CatArgs {
    /// Object reference (optionally @version)
    pub reference: String,
}

pub async fn run(repo: LatticeRepo, args: CatArgs) -> Result<()> {
    let (object_id, version_id) = parse_ref_with_version(&repo, &args.reference)?;

    let object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    let version = match version_id {
        Some(v) => repo.metadata.load_version(&v)?,
        None => repo.metadata.load_version(&object.current_version)?,
    };

    if is_quarantined_executable(&object.tags) {
        return Err(anyhow::anyhow!("Object is quarantined and executable"));
    }

    let manifest = repo.metadata.load_manifest(&version.manifest_ref)?;
    let data = repo.chunks.retrieve_object(&manifest).await?;

    let mut stdout = io::stdout();
    stdout.write_all(&data)?;
    Ok(())
}
