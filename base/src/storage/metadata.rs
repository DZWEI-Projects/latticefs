use crate::error::{LatticeError, Result};
use crate::storage::chunks::ChunkManifest;
use crate::storage::content::{hash_to_hex, Hash};
use sled::{Db, Tree};
use std::path::Path;

/// Metadata store using sled embedded database
pub struct MetadataStore {
    db: Db,
    objects: Tree,
    versions: Tree,
    manifests: Tree,
    tags: Tree,
    links: Tree,
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

        Ok(MetadataStore {
            db,
            objects,
            versions,
            manifests,
            tags,
            links,
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

    /// Flush all pending writes to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
