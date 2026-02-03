use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::model::Version;
use latticefs_base::LatticeRepo;

use super::common::{ensure_identity, identity_actor, parse_ref_with_version, resolve_object_id};

#[derive(Args, Debug)]
pub struct VersionsArgs {
    /// Object reference
    pub reference: String,
    /// Show parent graph
    #[arg(long)]
    pub graph: bool,
}

#[derive(Args, Debug)]
pub struct RestoreArgs {
    /// Object reference
    pub reference: String,
    /// Version to restore (uuid or vN)
    pub version: String,
}

pub async fn run(repo: LatticeRepo, args: VersionsArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;
    let object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;

    let mut versions: Vec<Version> = Vec::new();
    for vid in object.versions {
        versions.push(repo.metadata.load_version(&vid)?);
    }
    versions.sort_by_key(|v| v.created_at);

    if args.graph {
        for (idx, v) in versions.iter().enumerate() {
            let parent = v
                .parent_version
                .map(|p| p.to_string())
                .unwrap_or_else(|| "none".to_string());
            println!("v{} {} parent={} size={} state={}", idx + 1, v.id, parent, v.size_bytes, v.state);
        }
    } else {
        for (idx, v) in versions.iter().enumerate() {
            println!("v{} {} size={} state={}", idx + 1, v.id, v.size_bytes, v.state);
        }
    }

    Ok(())
}

pub async fn restore(repo: LatticeRepo, args: RestoreArgs) -> Result<()> {
    let spec = format!("{}@{}", args.reference, args.version);
    let (object_id, version_id) = parse_ref_with_version(&repo, &spec)?;
    let Some(version_id) = version_id else {
        return Err(anyhow::anyhow!("Invalid version spec"));
    };

    let actor = match ensure_identity("default", None) {
        Ok(id) => identity_actor(&id),
        Err(_) => [0u8; 32],
    };

    let mut object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    let target = repo.metadata.load_version(&version_id)?;

    let new_version = Version::new(
        object_id,
        Some(object.current_version),
        target.chunk_root,
        target.manifest_ref,
        actor,
        target.size_bytes,
        target.chunk_count,
        Some(format!("restore {}", version_id)),
    );

    object.add_version(new_version.id);
    repo.metadata.store_version(&new_version)?;
    repo.metadata.store_object(&object)?;

    println!("Restored {} to new version {}", object_id, new_version.id);
    Ok(())
}
