use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::LatticeRepo;
use latticefs_base::Permission;

use super::common::parse_ref_with_version;

#[derive(Args, Debug)]
pub struct CheckoutArgs {
    /// Object reference with version (ref@version)
    pub reference: String,
}

pub async fn run(repo: LatticeRepo, args: CheckoutArgs) -> Result<()> {
    let (object_id, version_id) = parse_ref_with_version(&repo, &args.reference)?;
    let Some(version_id) = version_id else {
        return Err(anyhow::anyhow!(
            "checkout requires a version spec (ref@version)"
        ));
    };

    let mut object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    repo.authorize_object_permission(&object, Permission::Write, false)?;
    repo.enforce_rate_limit(1)?;

    // Ensure version belongs to object
    let version = repo.metadata.load_version(&version_id)?;
    if version.object_id != object_id {
        return Err(anyhow::anyhow!("Version does not belong to object"));
    }

    object.current_version = version_id;
    repo.metadata.store_object(&object)?;

    println!("Checked out {} to {}", object_id, version_id);
    Ok(())
}
