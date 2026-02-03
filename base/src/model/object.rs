use crate::model::link::Link;
use crate::model::policy::PolicyID;
use crate::model::state::State;
use crate::model::tag::{timestamp_now, ActorID, Tag, Timestamp};
use crate::storage::content::Hash;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Object identifier (UUID v7 - time-ordered)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectID(pub Uuid);

impl ObjectID {
    pub fn new() -> Self {
        ObjectID(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        ObjectID(uuid)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

impl Default for ObjectID {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Version identifier (UUID v7 - time-ordered)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionID(pub Uuid);

impl VersionID {
    pub fn new() -> Self {
        VersionID(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        VersionID(uuid)
    }

    pub fn from_bytes(bytes: &[u8]) -> crate::error::Result<Self> {
        let uuid = Uuid::from_slice(bytes).map_err(|e| {
            crate::error::LatticeError::Serialization(format!("Invalid VersionID bytes: {}", e))
        })?;
        Ok(VersionID(uuid))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

impl Default for VersionID {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for VersionID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Key identifier (stub for Phase 1, full implementation in Phase 2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyID(pub [u8; 32]);

impl KeyID {
    pub fn from_hash(key: &[u8; 32]) -> Self {
        KeyID(*key)
    }
}

/// Object type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    /// Binary data (files) - MVP only type
    Blob,
    /// Collection of objects (future)
    Tree,
    /// Symbolic commit point (future)
    Commit,
}

impl std::fmt::Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Blob => write!(f, "blob"),
            ObjectType::Tree => write!(f, "tree"),
            ObjectType::Commit => write!(f, "commit"),
        }
    }
}

/// Metadata partitioning for privacy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataPartition {
    /// Visible only to owner
    Private,
    /// Visible to capability holders
    Shared,
    /// Searchable by anyone
    Public,
}

/// Object - immutable identity with mutable versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub id: ObjectID,
    pub created_at: Timestamp,
    pub created_by: ActorID,
    pub object_type: ObjectType,
    pub current_version: VersionID,
    pub versions: Vec<VersionID>,
    pub tags: Vec<Tag>,
    pub links: Vec<Link>,
    pub policy_refs: Vec<PolicyID>,
    pub metadata_partition: MetadataPartition,
}

impl Object {
    /// Create a new object
    pub fn new(object_type: ObjectType, initial_version: VersionID, created_by: ActorID) -> Self {
        Object {
            id: ObjectID::new(),
            created_at: timestamp_now(),
            created_by,
            object_type,
            current_version: initial_version,
            versions: vec![initial_version],
            tags: Vec::new(),
            links: Vec::new(),
            policy_refs: Vec::new(),
            metadata_partition: MetadataPartition::Shared,
        }
    }

    /// Add a new version to this object
    pub fn add_version(&mut self, version_id: VersionID) {
        self.versions.push(version_id);
        self.current_version = version_id;
    }

    /// Add a tag to this object
    pub fn add_tag(&mut self, tag: Tag) {
        // Don't add duplicate tags
        if !self
            .tags
            .iter()
            .any(|t| t.key == tag.key && t.value == tag.value)
        {
            self.tags.push(tag);
        }
    }

    /// Remove a tag from this object
    pub fn remove_tag(&mut self, key: &str) {
        self.tags.retain(|t| t.key != key);
    }

    /// Add a link to this object
    pub fn add_link(&mut self, link: Link) {
        self.links.push(link);
    }

    /// Add a policy to this object
    pub fn add_policy(&mut self, policy_id: PolicyID) {
        if !self.policy_refs.contains(&policy_id) {
            self.policy_refs.push(policy_id);
        }
    }
}

/// Version - snapshot of content at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub id: VersionID,
    pub object_id: ObjectID,
    pub parent_version: Option<VersionID>,
    pub chunk_root: Hash,
    pub manifest_ref: Hash,
    pub created_at: Timestamp,
    pub created_by: ActorID,
    pub state: State,
    pub encrypted: bool,
    pub encryption_key_ref: Option<KeyID>,
    pub size_bytes: u64,
    pub chunk_count: u32,
    pub commit_message: Option<String>,
}

impl Version {
    /// Create a new version
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        object_id: ObjectID,
        parent_version: Option<VersionID>,
        chunk_root: Hash,
        manifest_ref: Hash,
        created_by: ActorID,
        size_bytes: u64,
        chunk_count: u32,
        commit_message: Option<String>,
    ) -> Self {
        Version {
            id: VersionID::new(),
            object_id,
            parent_version,
            chunk_root,
            manifest_ref,
            created_at: timestamp_now(),
            created_by,
            state: State::Draft,
            encrypted: false,
            encryption_key_ref: None,
            size_bytes,
            chunk_count,
            commit_message,
        }
    }

    /// Transition this version to a new state
    pub fn transition_state(&mut self, new_state: State) -> Result<(), String> {
        if !self.state.can_transition_to(&new_state) {
            return Err(format!(
                "Invalid state transition: {} -> {}",
                self.state, new_state
            ));
        }

        self.state = new_state;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_actor() -> ActorID {
        [0u8; 32]
    }

    #[test]
    fn test_object_creation() {
        let version_id = VersionID::new();
        let object = Object::new(ObjectType::Blob, version_id, test_actor());

        assert_eq!(object.object_type, ObjectType::Blob);
        assert_eq!(object.current_version, version_id);
        assert_eq!(object.versions.len(), 1);
        assert_eq!(object.versions[0], version_id);
    }

    #[test]
    fn test_object_add_version() {
        let version1 = VersionID::new();
        let mut object = Object::new(ObjectType::Blob, version1, test_actor());

        let version2 = VersionID::new();
        object.add_version(version2);

        assert_eq!(object.current_version, version2);
        assert_eq!(object.versions.len(), 2);
        assert_eq!(object.versions[0], version1);
        assert_eq!(object.versions[1], version2);
    }

    #[test]
    fn test_object_add_tag() {
        let version_id = VersionID::new();
        let mut object = Object::new(ObjectType::Blob, version_id, test_actor());

        let tag = Tag::new("project".to_string(), "phoenix".to_string(), test_actor());
        object.add_tag(tag.clone());

        assert_eq!(object.tags.len(), 1);
        assert_eq!(object.tags[0].key, "project");

        // Adding duplicate should not increase count
        object.add_tag(tag);
        assert_eq!(object.tags.len(), 1);
    }

    #[test]
    fn test_object_remove_tag() {
        let version_id = VersionID::new();
        let mut object = Object::new(ObjectType::Blob, version_id, test_actor());

        let tag = Tag::new("project".to_string(), "phoenix".to_string(), test_actor());
        object.add_tag(tag);

        assert_eq!(object.tags.len(), 1);

        object.remove_tag("project");
        assert_eq!(object.tags.len(), 0);
    }

    #[test]
    fn test_version_creation() {
        let object_id = ObjectID::new();
        let chunk_root = [1u8; 32];
        let manifest_ref = [2u8; 32];

        let version = Version::new(
            object_id,
            None,
            chunk_root,
            manifest_ref,
            test_actor(),
            1000,
            5,
            Some("Initial version".to_string()),
        );

        assert_eq!(version.object_id, object_id);
        assert_eq!(version.parent_version, None);
        assert_eq!(version.chunk_root, chunk_root);
        assert_eq!(version.state, State::Draft);
        assert_eq!(version.size_bytes, 1000);
        assert_eq!(version.chunk_count, 5);
    }

    #[test]
    fn test_version_state_transition() {
        let object_id = ObjectID::new();
        let chunk_root = [1u8; 32];
        let manifest_ref = [2u8; 32];

        let mut version = Version::new(
            object_id,
            None,
            chunk_root,
            manifest_ref,
            test_actor(),
            1000,
            5,
            None,
        );

        // Valid transition
        assert!(version.transition_state(State::Review).is_ok());
        assert_eq!(version.state, State::Review);

        // Invalid transition
        assert!(version.transition_state(State::Draft).is_ok()); // Can go back to Draft
        assert!(version.transition_state(State::Approved).is_err()); // Cannot skip Review
    }

    #[test]
    fn test_object_id_roundtrip() {
        let id = ObjectID::new();
        let bytes = id.to_bytes();
        let uuid = Uuid::from_slice(&bytes).unwrap();
        let id2 = ObjectID::from_uuid(uuid);

        assert_eq!(id, id2);
    }

    #[test]
    fn test_version_id_roundtrip() {
        let id = VersionID::new();
        let bytes = id.to_bytes();
        let uuid = Uuid::from_slice(&bytes).unwrap();
        let id2 = VersionID::from_uuid(uuid);

        assert_eq!(id, id2);
    }
}
