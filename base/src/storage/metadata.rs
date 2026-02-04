use crate::error::{LatticeError, Result};
use crate::storage::chunks::ChunkManifest;
use crate::storage::content::{hash_to_hex, Hash};
use sled::{Db, Tree};
use std::path::Path;

/// Metadata store using sled embedded database
pub struct MetadataStore {
    root: std::path::PathBuf,
    #[allow(dead_code)]
    db: Db,
    objects: Tree,
    versions: Tree,
    manifests: Tree,
    tags: Tree,
    links: Tree,
    policies: Tree,
    policies_by_id: Tree,
    views: Tree,
    snapshots: Tree,
    text: Tree,
    inodes: Tree,
    capabilities: Tree,
    revocations: Tree,
    rate_limits: Tree,
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
        let policies = db.open_tree("policies")?;
        let policies_by_id = db.open_tree("policies_by_id")?;
        let views = db.open_tree("views")?;
        let snapshots = db.open_tree("snapshots")?;
        let text = db.open_tree("text")?;
        let inodes = db.open_tree("inodes")?;
        let capabilities = db.open_tree("capabilities")?;
        let revocations = db.open_tree("revocations")?;
        let rate_limits = db.open_tree("rate_limits")?;
        let aliases = db.open_tree("aliases")?;

        Ok(MetadataStore {
            root: path.to_path_buf(),
            db,
            objects,
            versions,
            manifests,
            tags,
            links,
            policies,
            policies_by_id,
            views,
            snapshots,
            text,
            inodes,
            capabilities,
            revocations,
            rate_limits,
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

    /// Store a typed object using bincode.
    pub fn store_object(&self, object: &crate::model::Object) -> Result<()> {
        let bytes = bincode::serialize(object).map_err(|e| {
            LatticeError::Serialization(format!("Failed to serialize object: {}", e))
        })?;
        self.store_object_bytes(object.id.as_bytes(), &bytes)
    }

    /// Load a typed object.
    pub fn load_object(&self, id: &crate::model::ObjectID) -> Result<crate::model::Object> {
        let bytes = self.load_object_bytes(id.as_bytes())?;
        let object = bincode::deserialize(&bytes).map_err(|e| {
            LatticeError::Serialization(format!("Failed to deserialize object: {}", e))
        })?;
        Ok(object)
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

    /// Store a typed version using bincode.
    pub fn store_version(&self, version: &crate::model::Version) -> Result<()> {
        let bytes = bincode::serialize(version).map_err(|e| {
            LatticeError::Serialization(format!("Failed to serialize version: {}", e))
        })?;
        self.store_version_bytes(version.id.as_bytes(), &bytes)
    }

    /// Load a typed version.
    pub fn load_version(&self, id: &crate::model::VersionID) -> Result<crate::model::Version> {
        let bytes = self.load_version_bytes(id.as_bytes())?;
        let version = bincode::deserialize(&bytes).map_err(|e| {
            LatticeError::Serialization(format!("Failed to deserialize version: {}", e))
        })?;
        Ok(version)
    }

    /// Iterate all objects as typed values.
    pub fn iter_objects(&self) -> impl Iterator<Item = Result<crate::model::Object>> + '_ {
        self.objects.iter().map(|item| {
            item.map_err(LatticeError::from).and_then(|(_k, v)| {
                bincode::deserialize::<crate::model::Object>(&v).map_err(|e| {
                    LatticeError::Serialization(format!("Failed to deserialize object: {}", e))
                })
            })
        })
    }

    /// Iterate all object IDs (raw bytes).
    pub fn iter_object_ids(&self) -> Result<Vec<Vec<u8>>> {
        let mut ids = Vec::new();
        for item in self.objects.iter() {
            let (k, _v) = item?;
            ids.push(k.to_vec());
        }
        Ok(ids)
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

    /// Store a link record.
    pub fn store_link(&self, link: &crate::model::Link) -> Result<()> {
        let bytes = bincode::serialize(link).map_err(|e| {
            LatticeError::Serialization(format!("Failed to serialize link: {}", e))
        })?;
        self.links.insert(link.id.as_bytes(), bytes)?;
        Ok(())
    }

    /// Load a link record by ID.
    pub fn load_link(&self, id: &crate::model::LinkID) -> Result<crate::model::Link> {
        let data = self
            .links
            .get(id.as_bytes())?
            .ok_or_else(|| LatticeError::ObjectNotFound {
                id: format!("link:{}", id),
            })?;
        let link = bincode::deserialize(&data).map_err(|e| {
            LatticeError::Serialization(format!("Failed to deserialize link: {}", e))
        })?;
        Ok(link)
    }

    /// Store a policy by name.
    pub fn store_policy(&self, policy: &crate::model::Policy) -> Result<()> {
        let bytes = bincode::serialize(policy).map_err(|e| {
            LatticeError::Serialization(format!("Failed to serialize policy: {}", e))
        })?;
        self.policies.insert(policy.name.as_bytes(), bytes)?;
        self.policies_by_id
            .insert(policy.id.as_bytes(), policy.name.as_bytes())?;
        Ok(())
    }

    /// Load a policy by name.
    pub fn load_policy(&self, name: &str) -> Result<crate::model::Policy> {
        let data = self
            .policies
            .get(name.as_bytes())?
            .ok_or_else(|| LatticeError::ObjectNotFound {
                id: format!("policy:{}", name),
            })?;
        let policy = bincode::deserialize(&data).map_err(|e| {
            LatticeError::Serialization(format!("Failed to deserialize policy: {}", e))
        })?;
        Ok(policy)
    }

    /// Load a policy by ID.
    pub fn load_policy_by_id(&self, id: &crate::model::PolicyID) -> Result<crate::model::Policy> {
        if let Some(name) = self.policies_by_id.get(id.as_bytes())? {
            let name = std::str::from_utf8(&name).map_err(|e| {
                LatticeError::Serialization(format!("Invalid policy name bytes: {}", e))
            })?;
            return self.load_policy(name);
        }

        // Fallback: scan policies for matching ID (compat for older stores).
        for item in self.policies.iter() {
            let (_k, v) = item?;
            let policy: crate::model::Policy = bincode::deserialize(&v).map_err(|e| {
                LatticeError::Serialization(format!("Failed to deserialize policy: {}", e))
            })?;
            if policy.id == *id {
                return Ok(policy);
            }
        }

        Err(LatticeError::ObjectNotFound {
            id: format!("policy:{}", id),
        })
    }

    /// Delete a policy by name.
    pub fn delete_policy(&self, name: &str) -> Result<()> {
        if let Ok(policy) = self.load_policy(name) {
            let _ = self.policies_by_id.remove(policy.id.as_bytes());
        }
        self.policies.remove(name.as_bytes())?;
        Ok(())
    }

    /// List all policies.
    pub fn list_policies(&self) -> Result<Vec<crate::model::Policy>> {
        let mut policies = Vec::new();
        for item in self.policies.iter() {
            let (_k, v) = item?;
            let policy = bincode::deserialize(&v).map_err(|e| {
                LatticeError::Serialization(format!("Failed to deserialize policy: {}", e))
            })?;
            policies.push(policy);
        }
        Ok(policies)
    }

    /// Store a view definition by name.
    pub fn store_view(&self, view: &crate::views::View) -> Result<()> {
        let bytes = bincode::serialize(view).map_err(|e| {
            LatticeError::Serialization(format!("Failed to serialize view: {}", e))
        })?;
        self.views.insert(view.name.as_bytes(), bytes)?;
        Ok(())
    }

    /// Load a view by name.
    pub fn load_view(&self, name: &str) -> Result<crate::views::View> {
        let data = self
            .views
            .get(name.as_bytes())?
            .ok_or_else(|| LatticeError::ViewNotFound {
                name: name.to_string(),
            })?;
        let view = bincode::deserialize(&data).map_err(|e| {
            LatticeError::Serialization(format!("Failed to deserialize view: {}", e))
        })?;
        Ok(view)
    }

    /// Delete a view by name.
    pub fn delete_view(&self, name: &str) -> Result<()> {
        self.views.remove(name.as_bytes())?;
        Ok(())
    }

    /// List all views.
    pub fn list_views(&self) -> Result<Vec<crate::views::View>> {
        let mut views = Vec::new();
        for item in self.views.iter() {
            let (_k, v) = item?;
            let view = bincode::deserialize(&v).map_err(|e| {
                LatticeError::Serialization(format!("Failed to deserialize view: {}", e))
            })?;
            views.push(view);
        }
        Ok(views)
    }

    /// Store a view snapshot.
    pub fn store_snapshot(&self, snapshot: &crate::views::ViewSnapshot) -> Result<()> {
        let bytes = bincode::serialize(snapshot).map_err(|e| {
            LatticeError::Serialization(format!("Failed to serialize snapshot: {}", e))
        })?;
        self.snapshots.insert(snapshot.id.to_string().as_bytes(), bytes)?;
        Ok(())
    }

    /// Load a snapshot by ID string.
    pub fn load_snapshot(&self, id: &str) -> Result<crate::views::ViewSnapshot> {
        let data = self
            .snapshots
            .get(id.as_bytes())?
            .ok_or_else(|| LatticeError::ObjectNotFound {
                id: format!("snapshot:{}", id),
            })?;
        let snapshot = bincode::deserialize(&data).map_err(|e| {
            LatticeError::Serialization(format!("Failed to deserialize snapshot: {}", e))
        })?;
        Ok(snapshot)
    }

    /// List all snapshots.
    pub fn list_snapshots(&self) -> Result<Vec<crate::views::ViewSnapshot>> {
        let mut snapshots = Vec::new();
        for item in self.snapshots.iter() {
            let (_k, v) = item?;
            let snapshot = bincode::deserialize(&v).map_err(|e| {
                LatticeError::Serialization(format!("Failed to deserialize snapshot: {}", e))
            })?;
            snapshots.push(snapshot);
        }
        Ok(snapshots)
    }

    /// Store extracted text content for an object.
    pub fn store_text(&self, object_id: &crate::model::ObjectID, text: &str) -> Result<()> {
        self.text.insert(object_id.as_bytes(), text.as_bytes())?;
        Ok(())
    }

    /// Load extracted text for an object.
    pub fn load_text(&self, object_id: &crate::model::ObjectID) -> Result<Option<String>> {
        Ok(self
            .text
            .get(object_id.as_bytes())?
            .map(|v| String::from_utf8_lossy(&v).to_string()))
    }

    /// Store an inode mapping (u64 -> object id bytes).
    pub fn store_inode_mapping(&self, inode: u64, object_id: &[u8]) -> Result<()> {
        let key = inode.to_be_bytes();
        self.inodes.insert(key, object_id)?;
        Ok(())
    }

    /// Load an inode mapping (u64 -> object id bytes).
    pub fn load_inode_mapping(&self, inode: u64) -> Result<Option<Vec<u8>>> {
        let key = inode.to_be_bytes();
        Ok(self.inodes.get(key)?.map(|v| v.to_vec()))
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

    /// List all stored capabilities.
    pub fn list_capabilities(&self) -> Result<Vec<(String, String)>> {
        let mut caps = Vec::new();
        for item in self.capabilities.iter() {
            let (k, v) = item?;
            let cid = String::from_utf8_lossy(&k).to_string();
            let token = String::from_utf8_lossy(&v).to_string();
            caps.push((cid, token));
        }
        Ok(caps)
    }

    /// Store a revocation entry.
    pub fn store_revocation(&self, revocation: &crate::crypto::Revocation) -> Result<()> {
        let data = serde_json::to_vec(revocation)
            .map_err(|e| LatticeError::Serialization(format!("Revocation serialize: {}", e)))?;
        self.revocations
            .insert(revocation.ucan_cid.as_bytes(), data)?;
        append_revocation_log(&self.root, revocation)?;
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

    /// Load the current rate limit state (if any).
    pub fn load_rate_limit_state(&self, key: &str) -> Result<Option<crate::policy::RateLimitState>> {
        if let Some(data) = self.rate_limits.get(key.as_bytes())? {
            let state: crate::policy::RateLimitState = bincode::deserialize(&data).map_err(|e| {
                LatticeError::Serialization(format!("Rate limit deserialize: {}", e))
            })?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    /// Store the rate limit state.
    pub fn store_rate_limit_state(
        &self,
        key: &str,
        state: &crate::policy::RateLimitState,
    ) -> Result<()> {
        let data = bincode::serialize(state).map_err(|e| {
            LatticeError::Serialization(format!("Rate limit serialize: {}", e))
        })?;
        self.rate_limits.insert(key.as_bytes(), data)?;
        Ok(())
    }

    /// Atomically check and consume rate limit tokens using compare-and-swap.
    ///
    /// This prevents race conditions when multiple concurrent requests try to
    /// consume tokens simultaneously. Uses optimistic locking with retry.
    pub fn atomic_rate_limit_consume<F>(
        &self,
        key: &str,
        check_fn: F,
    ) -> Result<()>
    where
        F: Fn(Option<crate::policy::RateLimitState>) -> Result<crate::policy::RateLimitState>,
    {
        const MAX_RETRIES: usize = 10;
        let key_bytes = key.as_bytes();

        for attempt in 0..MAX_RETRIES {
            // Load current state
            let current = self.rate_limits.get(key_bytes)?;
            let current_state: Option<crate::policy::RateLimitState> = match &current {
                Some(data) => Some(bincode::deserialize(data).map_err(|e| {
                    LatticeError::Serialization(format!("Rate limit deserialize: {}", e))
                })?),
                None => None,
            };

            // Apply the check function (may return RateLimited error)
            let new_state = check_fn(current_state)?;

            // Serialize new state
            let new_data = bincode::serialize(&new_state).map_err(|e| {
                LatticeError::Serialization(format!("Rate limit serialize: {}", e))
            })?;

            // Attempt atomic compare-and-swap
            let cas_result = self
                .rate_limits
                .compare_and_swap(key_bytes, current, Some(new_data))?;

            match cas_result {
                Ok(()) => return Ok(()),
                Err(_) => {
                    // CAS failed - another request updated the state concurrently
                    // Add small backoff on retries to reduce contention
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_micros(
                            100 * (1 << attempt.min(5)),
                        ));
                    }
                    continue;
                }
            }
        }

        // If we exhausted retries, the system is under extreme contention
        // Return a rate limit error to shed load
        Err(LatticeError::RateLimited {
            retry_after_secs: 1,
        })
    }
}

fn append_revocation_log(
    root: &std::path::Path,
    revocation: &crate::crypto::Revocation,
) -> Result<()> {
    let path = root.join("logs").join("revocations.jsonl");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let line = serde_json::to_string(revocation)
        .map_err(|e| LatticeError::Serialization(format!("Revocation serialize: {}", e)))?;
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", line)?;
    Ok(())
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
