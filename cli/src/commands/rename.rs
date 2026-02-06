use anyhow::{Context, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use clap::Args;
use latticefs_base::LatticeRepo;
use latticefs_base::model::Tag;

use super::common::{ensure_identity, identity_actor, resolve_object_id};

#[derive(Args, Debug)]
pub struct RenameArgs {
    /// Object reference
    pub reference: String,
    /// New filename to store as auto:filename_b64
    pub name: String,
}

pub async fn run(repo: LatticeRepo, args: RenameArgs) -> Result<()> {
    let name = args.name.trim();
    if name.is_empty() {
        anyhow::bail!("Filename cannot be empty");
    }

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

    let removed: Vec<_> = object
        .tags
        .iter()
        .filter(|t| t.key == "auto:filename_b64")
        .cloned()
        .collect();

    if !removed.is_empty() {
        object.remove_tag("auto:filename_b64");
        for tag in removed {
            repo.metadata
                .remove_from_tag_index(&tag.full_path(), object_id.as_bytes())?;
        }
    }

    let encoded = encode_filename_tag(name);
    let tag = Tag::new("auto:filename_b64".to_string(), encoded, actor);
    let full = tag.full_path();
    object.add_tag(tag);
    repo.metadata
        .add_to_tag_index(&full, object_id.as_bytes())?;
    repo.metadata.store_object(&object)?;

    println!("Renamed {} to {}", object_id, name);
    Ok(())
}

fn encode_filename_tag(name: &str) -> String {
    URL_SAFE_NO_PAD.encode(name.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::encode_filename_tag;

    #[test]
    fn encode_filename_tag_uses_base64url() {
        let encoded = encode_filename_tag("Report Final (v1).txt");
        assert_eq!(encoded, "UmVwb3J0IEZpbmFsICh2MSkudHh0");
    }
}
