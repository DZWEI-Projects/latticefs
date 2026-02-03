use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::model::Version;
use latticefs_base::LatticeRepo;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

use super::common::{ensure_identity, identity_actor, resolve_object_id};

#[derive(Args, Debug)]
pub struct ReviseArgs {
    /// Object reference
    pub reference: String,
    /// Path to new content for this object
    pub file: Option<PathBuf>,
    /// Read content from stdin instead of a file
    #[arg(long)]
    pub stdin: bool,
    /// Optional message for the new version
    #[arg(long, short = 'm')]
    pub message: Option<String>,
}

pub async fn run(repo: LatticeRepo, args: ReviseArgs) -> Result<()> {
    if args.stdin && args.file.is_some() {
        return Err(anyhow::anyhow!("Use either a file path or --stdin, not both"));
    }
    if !args.stdin && args.file.is_none() {
        return Err(anyhow::anyhow!("revise requires a file path or --stdin"));
    }

    let object_id = resolve_object_id(&repo, &args.reference)?;
    let mut object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;

    let actor = match ensure_identity("default", None) {
        Ok(id) => identity_actor(&id),
        Err(_) => [0u8; 32],
    };

    let data = if args.stdin {
        let mut buf = Vec::new();
        let mut stdin = tokio::io::stdin();
        stdin.read_to_end(&mut buf).await?;
        buf
    } else {
        let path = args.file.as_ref().expect("file required");
        tokio::fs::read(path)
            .await
            .with_context(|| format!("Failed to read {}", path.display()))?
    };
    let manifest = repo.chunks.store_object(&data).await?;
    let manifest_hash = repo.metadata.store_manifest(&manifest)?;

    let version = Version::new(
        object_id,
        Some(object.current_version),
        manifest.merkle_root,
        manifest_hash,
        actor,
        data.len() as u64,
        manifest.chunks.len() as u32,
        args.message.clone(),
    );

    object.add_version(version.id);
    repo.metadata.store_version(&version)?;
    repo.metadata.store_object(&object)?;

    println!("Revised {} to new version {}", object_id, version.id);
    Ok(())
}
