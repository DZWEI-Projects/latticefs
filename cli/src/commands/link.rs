use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::model::{Link, LinkType};
use latticefs_base::LatticeRepo;
use latticefs_base::Permission;

use super::common::{ensure_identity, identity_actor, resolve_object_id};

#[derive(Args, Debug)]
pub struct LinkArgs {
    /// Source object reference
    pub source: String,
    /// Link type (derived-from, references, belongs-to, replaces, related)
    pub link_type: String,
    /// Target object reference
    pub target: String,
}

pub async fn run(repo: LatticeRepo, args: LinkArgs) -> Result<()> {
    let source_id = resolve_object_id(&repo, &args.source)?;
    let target_id = resolve_object_id(&repo, &args.target)?;
    let link_type: LinkType = args
        .link_type
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    let actor = match ensure_identity("default", None) {
        Ok(id) => identity_actor(&id),
        Err(_) => [0u8; 32],
    };

    let mut source = repo
        .metadata
        .load_object(&source_id)
        .with_context(|| format!("Object not found: {}", source_id))?;
    let mut target = repo
        .metadata
        .load_object(&target_id)
        .with_context(|| format!("Object not found: {}", target_id))?;
    repo.authorize_object_permission(&source, Permission::Write, false)?;
    if link_type.is_bidirectional() {
        repo.authorize_object_permission(&target, Permission::Write, false)?;
    }
    repo.enforce_rate_limit(1)?;

    let link = Link::new(
        source_id.as_bytes().to_vec(),
        target_id.as_bytes().to_vec(),
        link_type,
        actor,
    );

    source.add_link(link.clone());
    repo.metadata.store_link(&link)?;

    if link_type.is_bidirectional() {
        let reverse = Link::new(
            target_id.as_bytes().to_vec(),
            source_id.as_bytes().to_vec(),
            link_type,
            actor,
        );
        target.add_link(reverse.clone());
        repo.metadata.store_link(&reverse)?;
        repo.metadata.store_object(&target)?;
    }

    repo.metadata.store_object(&source)?;
    println!("Linked {} -> {} ({})", source_id, target_id, link_type);
    Ok(())
}
