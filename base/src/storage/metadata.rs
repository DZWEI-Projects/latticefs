use crate::error::{LatticeError, Result};
use crate::storage::chunks::ChunkManifest;
use crate::storage::content::{hash_to_hex, Hash};
use sled::{Db, Tree};
use std::path::Path;

/// Metadata store using sled embedded database
pub struct MetadataStore {
    #[allow(dead_code)]
    db: Db,
    objects: Tree,
    versions: Tree,
    manifests: Tree,
    tags: Tree,
    #[allow(dead_code)] // Used in Phase 3
    links: Tree,
    capabilities: Tree,
    revocations: Tree,
    aliases: Tree,
}

impl MetadataStore {
    /// Open or create a metadata store at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let db = sled::open(path.join("meta"))?;

        let objects = db.open_tree("objects")?;
        let versions = db.open_tree("versions")?;
        let manifests = db.open_tree("manifests")?;
        let tags = db.open_tree("tags")?;
        let links = db.open_tree("links")?;
        let capabilities = db.open_tree("capabilities")?;
        let revocations = db.open_tree("revocations")?;
        let aliases = db.open_tree("aliases")?;

        Ok(MetadataStore {
            db,
            objects,
            versions,
            manifests,
            tags,
            links,
            capabilities,
            revocations,
            aliases,
        })
    }

    /// Store a manifest and return its hash
    pub fn store_manifest(&self, manifest: &ChunkManifest) -> Result<Hash> {
        let data = bincode::serialize(manifest).map_err(|e| {
            LatticeError::Serialization(format!("Failed to serialize manifest: {}", e))
        })?;

        let hash = crate::storage::content::compute_hash(&data);
        let key = hash_to_hex(&hash);

        self.manifests.insert(key.as_bytes(), data)?;

        Ok(hash)
    }

    /// Load a manifest by its hash
    pub fn load_manifest(&self, hash: &Hash) -> Result<ChunkManifest> {
        let key = hash_to_hex(hash);

        let data =
            self.manifests
                .get(key.as_bytes())?
                .ok_or_else(|| LatticeError::ManifestNotFound {
                    hash: hash_to_hex(hash),
                })?;

        let manifest = bincode::deserialize(&data).map_err(|e| {
            LatticeError::Serialization(format!("Failed to deserialize manifest: {}", e))
        })?;

        Ok(manifest)
    }

    /// Store an object (implementation depends on Object type from model module)
    pub fn store_object_bytes(&self, id: &[u8], data: &[u8]) -> Result<()> {
        self.objects.insert(id, data)?;
        Ok(())
    }

    /// Load an object by ID (returns raw bytes)
    pub fn load_object_bytes(&self, id: &[u8]) -> Result<Vec<u8>> {
        let data = self
            .objects
            .get(id)?
            .ok_or_else(|| LatticeError::ObjectNotFound {
                id: hex::encode(id),
            })?;

        Ok(data.to_vec())
    }

    /// Delete an object
    pub fn delete_object(&self, id: &[u8]) -> Result<()> {
        self.objects.remove(id)?;
        Ok(())
    }

    /// Store a version (implementation depends on Version type from model module)
    pub fn store_version_bytes(&self, id: &[u8], data: &[u8]) -> Result<()> {
        self.versions.insert(id, data)?;
        Ok(())
    }

    /// Load a version by ID (returns raw bytes)
    pub fn load_version_bytes(&self, id: &[u8]) -> Result<Vec<u8>> {
        let data = self
            .versions
            .get(id)?
            .ok_or_else(|| LatticeError::VersionNotFound {
                id: hex::encode(id),
            })?;

        Ok(data.to_vec())
    }

    /// Add object to tag index
    pub fn add_to_tag_index(&self, tag: &str, object_id: &[u8]) -> Result<()> {
        let key = tag.as_bytes();

        // Get existing list or create new one
        let mut object_ids: Vec<Vec<u8>> = if let Some(data) = self.tags.get(key)? {
            bincode::deserialize(&data).map_err(|e| {
                LatticeError::Serialization(format!("Failed to deserialize tag index: {}", e))
            })?
        } else {
            Vec::new()
        };

        // Add object_id if not already present
        if !object_ids.iter().any(|id| id == object_id) {
            object_ids.push(object_id.to_vec());
        }

        // Store updated list
        let data = bincode::serialize(&object_ids).map_err(|e| {
            LatticeError::Serialization(format!("Failed to serialize tag index: {}", e))
        })?;

        self.tags.insert(key, data)?;

        Ok(())
    }

    /// Remove object from tag index
    pub fn remove_from_tag_index(&self, tag: &str, object_id: &[u8]) -> Result<()> {
        let key = tag.as_bytes();

        if let Some(data) = self.tags.get(key)? {
            let mut object_ids: Vec<Vec<u8>> = bincode::deserialize(&data).map_err(|e| {
                LatticeError::Serialization(format!("Failed to deserialize tag index: {}", e))
            })?;

            object_ids.retain(|id| id != object_id);

            if object_ids.is_empty() {
                self.tags.remove(key)?;
            } else {
                let data = bincode::serialize(&object_ids).map_err(|e| {
                    LatticeError::Serialization(format!("Failed to serialize tag index: {}", e))
                })?;
                self.tags.insert(key, data)?;
            }
        }

        Ok(())
    }

    /// Query objects by tag
    pub fn query_by_tag(&self, tag: &str) -> Result<Vec<Vec<u8>>> {
        let key = tag.as_bytes();

        if let Some(data) = self.tags.get(key)? {
            let object_ids: Vec<Vec<u8>> = bincode::deserialize(&data).map_err(|e| {
                LatticeError::Serialization(format!("Failed to deserialize tag index: {}", e))
            })?;
            Ok(object_ids)
        } else {
            Ok(Vec::new())
        }
    }

    /// Set an alias for an object (overwrites existing alias).
    pub fn set_alias(&self, alias: &str, object_id: &[u8]) -> Result<()> {
        self.aliases.insert(alias.as_bytes(), object_id)?;
        Ok(())
    }

    /// Resolve an alias to an object ID.
    pub fn resolve_alias(&self, alias: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.aliases.get(alias.as_bytes())?.map(|v| v.to_vec()))
    }

    /// Delete an alias.
    pub fn delete_alias(&self, alias: &str) -> Result<()> {
        self.aliases.remove(alias.as_bytes())?;
        Ok(())
    }

    /// Flush all pending writes to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    /// Iterate over all version IDs for a given object.
    ///
    /// This scans all versions and filters by object_id.
    /// Returns the raw bytes of each matching VersionID.
    pub fn iter_versions_for_object(&self, object_id: &[u8]) -> Result<Vec<Vec<u8>>> {
        let mut matching_versions = Vec::new();

        for item in self.versions.iter() {
            let (key, value) = item?;

            // Deserialize the version to check its object_id
            let version: crate::model::Version = bincode::deserialize(&value).map_err(|e| {
                LatticeError::Serialization(format!("Failed to deserialize version: {}", e))
            })?;

            if version.object_id.as_bytes() == object_id {
                matching_versions.push(key.to_vec());
            }
        }

        Ok(matching_versions)
    }

    /// Iterate over all versions in the store.
    ///
    /// Returns an iterator over (VersionID bytes, Version bytes) pairs.
    pub fn iter_all_versions(&self) -> impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + '_ {
        self.versions.iter().map(|result| {
            result
                .map(|(k, v)| (k.to_vec(), v.to_vec()))
                .map_err(LatticeError::from)
        })
    }

    /// Store a capability token by its CID.
    pub fn store_capability(&self, capability: &crate::crypto::Capability) -> Result<()> {
        let cid = capability.cid();
        self.capabilities
            .insert(cid.as_bytes(), capability.token.as_bytes())?;
        Ok(())
    }

    /// Load a capability token by its CID.
    pub fn load_capability(&self, cid: &str) -> Result<crate::crypto::Capability> {
        let data = self
            .capabilities
            .get(cid.as_bytes())?
            .ok_or_else(|| LatticeError::CapabilityNotFound {
                cid: cid.to_string(),
            })?;

        let token = std::str::from_utf8(&data)
            .map_err(|e| LatticeError::Serialization(format!("Invalid UTF-8 token: {}", e)))?;

        crate::crypto::Capability::parse(token)
    }

    /// Delete a capability token by its CID.
    pub fn delete_capability(&self, cid: &str) -> Result<()> {
        self.capabilities.remove(cid.as_bytes())?;
        Ok(())
    }

    /// Store a revocation entry.
    pub fn store_revocation(&self, revocation: &crate::crypto::Revocation) -> Result<()> {
        let data = serde_json::to_vec(revocation)
            .map_err(|e| LatticeError::Serialization(format!("Revocation serialize: {}", e)))?;
        self.revocations
            .insert(revocation.ucan_cid.as_bytes(), data)?;
        Ok(())
    }

    /// Load a revocation entry by CID.
    pub fn load_revocation(&self, cid: &str) -> Result<crate::crypto::Revocation> {
        let data = self
            .revocations
            .get(cid.as_bytes())?
            .ok_or_else(|| LatticeError::RevocationNotFound {
                cid: cid.to_string(),
            })?;

        let revocation: crate::crypto::Revocation = serde_json::from_slice(&data).map_err(|e| {
            LatticeError::Serialization(format!("Revocation deserialize: {}", e))
        })?;

        Ok(revocation)
    }

    /// Check if a capability CID is revoked (verifies signature).
    pub fn is_revoked(&self, cid: &str) -> Result<bool> {
        match self.revocations.get(cid.as_bytes())? {
            Some(data) => {
                let revocation: crate::crypto::Revocation =
                    serde_json::from_slice(&data).map_err(|e| {
                        LatticeError::Serialization(format!("Revocation deserialize: {}", e))
                    })?;
                revocation.verify()?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Iterate over all revocations.
    pub fn iter_revocations(&self) -> impl Iterator<Item = Result<crate::crypto::Revocation>> + '_ {
        self.revocations.iter().map(|item| {
            item.map_err(LatticeError::from).and_then(|(_k, v)| {
                serde_json::from_slice::<crate::crypto::Revocation>(&v).map_err(|e| {
                    LatticeError::Serialization(format!("Revocation deserialize: {}", e))
                })
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{Capability, Identity, Permission, PublicKey, RevocationList};
    use crate::storage::chunks::{ChunkManifest, ChunkRef};

    #[test]
    fn test_manifest_store_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let manifest = ChunkManifest {
            version: 1,
            total_size: 1000,
            chunk_size_avg: 16384,
            chunks: vec![ChunkRef {
                hash: [0u8; 32],
                offset: 0,
                length: 1000,
            }],
            merkle_root: [1u8; 32],
        };

        let hash = store.store_manifest(&manifest).unwrap();
        let loaded = store.load_manifest(&hash).unwrap();

        assert_eq!(loaded.version, manifest.version);
        assert_eq!(loaded.total_size, manifest.total_size);
        assert_eq!(loaded.chunks.len(), manifest.chunks.len());
    }

    #[test]
    fn test_tag_index() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let obj_id1 = b"object1";
        let obj_id2 = b"object2";
        let tag = "project:phoenix";

        // Add objects to tag
        store.add_to_tag_index(tag, obj_id1).unwrap();
        store.add_to_tag_index(tag, obj_id2).unwrap();

        // Query tag
        let results = store.query_by_tag(tag).unwrap();
        assert_eq!(results.len(), 2);

        // Remove one object
        store.remove_from_tag_index(tag, obj_id1).unwrap();

        let results = store.query_by_tag(tag).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_capability_store_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let issuer = Identity::generate("alice");
        let audience = PublicKey::new(Identity::generate("bob").verifying_key);
        let object_id = crate::model::ObjectID::new();

        let cap = Capability::create(
            &issuer,
            &audience,
            &object_id,
            Permission::Read,
            std::time::Duration::from_secs(3600),
        )
        .unwrap();

        store.store_capability(&cap).unwrap();
        let loaded = store.load_capability(&cap.cid()).unwrap();
        assert_eq!(loaded.token, cap.token);
    }

    #[test]
    fn test_revocation_store() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let issuer = Identity::generate("alice");
        let audience = PublicKey::new(Identity::generate("bob").verifying_key);
        let object_id = crate::model::ObjectID::new();

        let cap = Capability::create(
            &issuer,
            &audience,
            &object_id,
            Permission::Read,
            std::time::Duration::from_secs(3600),
        )
        .unwrap();

        let revocation = cap
            .revoke(&issuer, None, None, &RevocationList::default())
            .unwrap();

        store.store_revocation(&revocation).unwrap();
        assert!(store.is_revoked(&cap.cid()).unwrap());

        let loaded = store.load_revocation(&cap.cid()).unwrap();
        assert_eq!(loaded.ucan_cid, revocation.ucan_cid);
        assert!(loaded.verify().is_ok());
    }

    #[test]
    fn test_alias_store_resolve() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id = crate::model::ObjectID::new();
        let alias = "project-readme";

        store.set_alias(alias, object_id.as_bytes()).unwrap();
        let resolved = store.resolve_alias(alias).unwrap().unwrap();
        assert_eq!(resolved, object_id.as_bytes());

        store.delete_alias(alias).unwrap();
        assert!(store.resolve_alias(alias).unwrap().is_none());
    }
}
