use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::LatticeRepo;
use latticefs_base::model::Tag;

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
    repo.authorize_object_permission(&object, latticefs_base::Permission::Write, false)?;
    repo.enforce_rate_limit(1)?;

    for tag_str in &args.tags {
        let tag = Tag::parse(tag_str, actor)?;
        let full = tag.full_path();
        object.add_tag(tag);
        repo.metadata
            .add_to_tag_index(&full, object_id.as_bytes())?;
    }

    repo.metadata.store_object(&object)?;
    println!("Tagged {}", object_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Debug, Parser)]
    struct Cli {
        #[command(flatten)]
        args: TagArgs,
    }

    #[test]
    fn parses_reference_and_tags() {
        let cli = Cli::parse_from(["tag", "obj-ref", "kind:image", "project:lattice"]);
        assert_eq!(cli.args.reference, "obj-ref");
        assert_eq!(
            cli.args.tags,
            vec!["kind:image".to_string(), "project:lattice".to_string()]
        );
    }

    #[test]
    fn tag_requires_reference() {
        let err = Cli::try_parse_from(["tag"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }
}
