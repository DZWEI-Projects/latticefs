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

#[cfg(test)]
mod tests {
    use super::*;
    use latticefs_base::model::{Object, ObjectID, ObjectType, Version, VersionID};
    use tempfile::TempDir;

    fn test_actor() -> [u8; 32] {
        [0u8; 32]
    }

    // Helper to create a test object with multiple versions
    async fn create_test_object_with_versions(
        repo: &LatticeRepo,
        num_versions: usize,
    ) -> Result<(ObjectID, Vec<VersionID>)> {
        let object_id = ObjectID::new();
        let mut version_ids = Vec::new();

        for i in 0..num_versions {
            let data = format!("Version {}", i + 1);
            let manifest = repo.store_object_data(data.as_bytes()).await?;
            let manifest_ref = repo.metadata.store_manifest(&manifest)?;

            let parent_version = if i == 0 {
                None
            } else {
                Some(version_ids[i - 1])
            };

            let version = Version::new(
                object_id,
                parent_version,
                manifest.merkle_root,
                manifest_ref,
                test_actor(),
                data.len() as u64,
                manifest.chunks.len() as u32,
                None,
            );

            version_ids.push(version.id);
            repo.metadata.store_version(&version)?;
        }

        // Create object with first version as current
        let mut object = Object::new(ObjectType::Blob, version_ids[0], test_actor());
        object.id = object_id;
        object.versions = version_ids.clone();
        if num_versions > 1 {
            // Set current to last version
            object.current_version = version_ids[num_versions - 1];
        }
        repo.metadata.store_object(&object)?;

        Ok((object_id, version_ids))
    }

    #[tokio::test]
    async fn test_checkout_success() {
        let temp = TempDir::new().unwrap();
        let repo = LatticeRepo::open_at(temp.path()).unwrap();

        let (object_id, _version_ids) = create_test_object_with_versions(&repo, 3).await.unwrap();

        // Checkout to version 1 (first version)
        let args = CheckoutArgs {
            reference: format!("{}@v1", object_id),
        };

        let result = run(repo, args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_checkout_updates_current_version() {
        let temp = TempDir::new().unwrap();
        let repo = LatticeRepo::open_at(temp.path()).unwrap();

        let (object_id, version_ids) = create_test_object_with_versions(&repo, 3).await.unwrap();

        // Current version should be v3 (last)
        let object_before = repo.metadata.load_object(&object_id).unwrap();
        assert_eq!(object_before.current_version, version_ids[2]);

        // Checkout to v1
        let args = CheckoutArgs {
            reference: format!("{}@v1", object_id),
        };

        run(repo, args).await.unwrap();

        // Re-open repo to verify current version is now v1
        let repo = LatticeRepo::open_at(temp.path()).unwrap();
        let object_after = repo.metadata.load_object(&object_id).unwrap();
        assert_eq!(object_after.current_version, version_ids[0]);
    }

    #[tokio::test]
    async fn test_checkout_by_version_uuid() {
        let temp = TempDir::new().unwrap();
        let repo = LatticeRepo::open_at(temp.path()).unwrap();

        let (object_id, version_ids) = create_test_object_with_versions(&repo, 2).await.unwrap();

        // Checkout using explicit version UUID
        let args = CheckoutArgs {
            reference: format!("{}@{}", object_id, version_ids[0]),
        };

        let result = run(repo, args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_checkout_no_version_specified() {
        let temp = TempDir::new().unwrap();
        let repo = LatticeRepo::open_at(temp.path()).unwrap();

        let (object_id, _version_ids) = create_test_object_with_versions(&repo, 2).await.unwrap();

        // Try to checkout without specifying version
        let args = CheckoutArgs {
            reference: object_id.to_string(),
        };

        let result = run(repo, args).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("checkout requires a version spec")
        );
    }

    #[tokio::test]
    async fn test_checkout_object_not_found() {
        let temp = TempDir::new().unwrap();
        let repo = LatticeRepo::open_at(temp.path()).unwrap();

        let fake_id = ObjectID::new();
        let fake_version = VersionID::new();
        let args = CheckoutArgs {
            reference: format!("{}@{}", fake_id, fake_version),
        };

        let result = run(repo, args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Object not found"));
    }

    #[tokio::test]
    async fn test_checkout_version_not_found() {
        let temp = TempDir::new().unwrap();
        let repo = LatticeRepo::open_at(temp.path()).unwrap();

        let (object_id, _version_ids) = create_test_object_with_versions(&repo, 2).await.unwrap();

        let fake_version = VersionID::new();
        let args = CheckoutArgs {
            reference: format!("{}@{}", object_id, fake_version),
        };

        let result = run(repo, args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_checkout_version_belongs_to_different_object() {
        let temp = TempDir::new().unwrap();
        let repo = LatticeRepo::open_at(temp.path()).unwrap();

        // Create two objects
        let (object1_id, _version1_ids) = create_test_object_with_versions(&repo, 1).await.unwrap();
        let (_object2_id, version2_ids) = create_test_object_with_versions(&repo, 1).await.unwrap();

        // Try to checkout object1 to a version from object2
        let args = CheckoutArgs {
            reference: format!("{}@{}", object1_id, version2_ids[0]),
        };

        let result = run(repo, args).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Version does not belong to object")
        );
    }

    #[tokio::test]
    async fn test_checkout_invalid_version_alias() {
        let temp = TempDir::new().unwrap();
        let repo = LatticeRepo::open_at(temp.path()).unwrap();

        let (object_id, _version_ids) = create_test_object_with_versions(&repo, 2).await.unwrap();

        // Try to checkout to a version that doesn't exist (v10)
        let args = CheckoutArgs {
            reference: format!("{}@v10", object_id),
        };

        let result = run(repo, args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_checkout_between_versions() {
        let temp = TempDir::new().unwrap();
        let repo = LatticeRepo::open_at(temp.path()).unwrap();

        let (object_id, version_ids) = create_test_object_with_versions(&repo, 3).await.unwrap();

        // Checkout to v2
        let args = CheckoutArgs {
            reference: format!("{}@v2", object_id),
        };
        run(repo, args).await.unwrap();

        // Re-open repo to verify
        let repo = LatticeRepo::open_at(temp.path()).unwrap();
        let object = repo.metadata.load_object(&object_id).unwrap();
        assert_eq!(object.current_version, version_ids[1]);
    }
}
