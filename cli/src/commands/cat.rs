use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::model::Tag;
use latticefs_base::LatticeRepo;
use std::io::{self, Write};

use super::common::parse_ref_with_version;

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

fn trust_level(tags: &[Tag]) -> u8 {
    tags.iter()
        .find(|t| t.key == "sys:trust")
        .and_then(|t| t.value.parse::<u8>().ok())
        .unwrap_or(75)
}

fn has_executable_tag(tags: &[Tag]) -> bool {
    tags.iter()
        .any(|t| t.key == "auto:executable" && t.value == "true")
}

fn is_quarantined_executable(tags: &[Tag]) -> bool {
    has_executable_tag(tags) && trust_level(tags) < 90
}
