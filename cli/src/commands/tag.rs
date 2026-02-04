use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::model::Tag;
use latticefs_base::LatticeRepo;

use super::common::{ensure_identity, identity_actor, resolve_object_id};

#[derive(Args, Debug)]
pub struct TagArgs {
    /// Object reference
    pub reference: String,
    /// Tags to add (key:value)
    pub tags: Vec<String>,
}

pub async fn run(repo: LatticeRepo, args: TagArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;
    let actor = match ensure_identity("default", None) {
        Ok(id) => identity_actor(&id),
        Err(_) => [0u8; 32],
    };

    let mut object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;

    for tag_str in &args.tags {
        let tag = Tag::parse(tag_str, actor)?;
        let full = tag.full_path();
        object.add_tag(tag);
        repo.metadata.add_to_tag_index(&full, object_id.as_bytes())?;
    }

    repo.metadata.store_object(&object)?;
    println!("Tagged {}", object_id);
    Ok(())
}
