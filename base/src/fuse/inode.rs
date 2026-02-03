//! Deterministic inode mapping for FUSE.
//!
//! Inode strategy (LFS-001 / plan):
//! - inode = BLAKE3(object_id)[0..8] as u64
//! - collisions handled by linear probing in sled index
//! - reserved inodes:
//!   1: /
//!   2: /views
//!   3: /projects
//!   4: /recent

use crate::error::Result;
use crate::model::ObjectID;
use crate::storage::MetadataStore;

pub const ROOT_INODE: u64 = 1;
pub const VIEWS_INODE: u64 = 2;
pub const PROJECTS_INODE: u64 = 3;
pub const RECENT_INODE: u64 = 4;
const RESERVED_MAX: u64 = 4;

pub struct InodeMapper<'a> {
    store: &'a MetadataStore,
}

impl<'a> InodeMapper<'a> {
    pub fn new(store: &'a MetadataStore) -> Self {
        Self { store }
    }

    /// Compute or retrieve a deterministic inode for an object ID.
    pub fn inode_for_object(&self, object_id: &ObjectID) -> Result<u64> {
        let mut inode = hash_to_inode(object_id.as_bytes());
        if inode <= RESERVED_MAX {
            inode = inode.wrapping_add(RESERVED_MAX + 1);
        }

        loop {
            if let Some(existing) = self.store.load_inode_mapping(inode)? {
                if existing == object_id.as_bytes() {
                    return Ok(inode);
                }
                inode = inode.wrapping_add(1);
                continue;
            }

            self.store.store_inode_mapping(inode, object_id.as_bytes())?;
            return Ok(inode);
        }
    }

    /// Resolve an inode back to an object ID if mapped.
    pub fn object_id_for_inode(&self, inode: u64) -> Result<Option<ObjectID>> {
        if let Some(bytes) = self.store.load_inode_mapping(inode)? {
            if let Ok(uuid) = uuid::Uuid::from_slice(&bytes) {
                return Ok(Some(ObjectID::from_uuid(uuid)));
            }
        }
        Ok(None)
    }
}

/// Compute a deterministic inode for a view name.
/// Not persisted; used for directory inodes under /views.
pub fn inode_for_view_name(name: &str) -> u64 {
    let mut data = Vec::new();
    data.extend_from_slice(b"view:");
    data.extend_from_slice(name.as_bytes());
    let mut inode = hash_to_inode(&data);
    if inode <= RESERVED_MAX {
        inode = inode.wrapping_add(RESERVED_MAX + 1);
    }
    inode
}

fn hash_to_inode(bytes: &[u8]) -> u64 {
    let hash = blake3::hash(bytes);
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&hash.as_bytes()[0..8]);
    u64::from_le_bytes(arr)
}
