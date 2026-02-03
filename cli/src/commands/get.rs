use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::crypto::Capability;
use latticefs_base::model::Tag;
use latticefs_base::LatticeRepo;
use std::path::PathBuf;

use super::common::parse_ref_with_version;

#[derive(Args, Debug)]
pub struct GetArgs {
    /// Object reference (optionally @version)
    pub reference: String,
    /// Output path
    #[arg(long, short = 'o')]
    pub output: PathBuf,
    /// UCAN capability token (optional)
    #[arg(long)]
    pub ucan: Option<String>,
}

pub async fn run(repo: LatticeRepo, args: GetArgs) -> Result<()> {
    let (object_id, version_id) = parse_ref_with_version(&repo, &args.reference)?;

    if let Some(token) = args.ucan.as_ref() {
        let cap = Capability::parse(token)?;
        cap.validate(&repo.metadata)?;
        if !cap.has_permission(&object_id, latticefs_base::Permission::Read) {
            return Err(anyhow::anyhow!("Capability does not grant read permission"));
        }
    }

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

    let out_path = if args.output.is_dir() {
        args.output.join(object_id.to_string())
    } else {
        args.output
    };

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out_path, data)?;

    println!("Wrote {}", out_path.display());
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
