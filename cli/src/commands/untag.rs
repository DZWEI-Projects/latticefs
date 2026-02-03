use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::LatticeRepo;

use super::common::resolve_object_id;

#[derive(Args, Debug)]
pub struct UntagArgs {
    /// Object reference
    pub reference: String,
    /// Tag key to remove
    pub key: String,
}

pub async fn run(repo: LatticeRepo, args: UntagArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;
    let mut object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;

    let removed: Vec<_> = object
        .tags
        .iter()
        .filter(|t| t.key == args.key)
        .cloned()
        .collect();

    object.remove_tag(&args.key);
    for tag in removed {
        repo.metadata
            .remove_from_tag_index(&tag.full_path(), object_id.as_bytes())?;
    }

    repo.metadata.store_object(&object)?;
    println!("Untagged {}", object_id);
    Ok(())
}
