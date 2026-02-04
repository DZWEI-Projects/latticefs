use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::LatticeRepo;

use super::common::resolve_object_id;

#[derive(Args, Debug)]
pub struct TagsArgs {
    /// Object reference
    pub reference: String,
}

pub async fn run(repo: LatticeRepo, args: TagsArgs) -> Result<()> {
    let object_id = resolve_object_id(&repo, &args.reference)?;
    let object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;

    if object.tags.is_empty() {
        println!("No tags");
        return Ok(());
    }

    for tag in object.tags {
        println!("{}", tag.full_path());
    }

    Ok(())
}
