//! View snapshots for LatticeFS.
//!
//! Snapshots capture the state of a view at a specific point in time,
//! creating an immutable set of object references that can be shared.

use crate::model::{timestamp_now, ObjectID};
use crate::views::{View, ViewID};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a view snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotID(Uuid);

impl SnapshotID {
    /// Create a new random snapshot ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SnapshotID {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SnapshotID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An immutable snapshot of a view at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewSnapshot {
    /// Unique identifier.
    pub id: SnapshotID,
    /// The view this snapshot was created from (if any).
    pub source_view: Option<ViewID>,
    /// Human-readable name.
    pub name: String,
    /// The query that was used to generate this snapshot.
    pub query: String,
    /// When the snapshot was created.
    pub created_at: i64,
    /// Who created the snapshot.
    pub created_by: [u8; 32],
    /// The object IDs captured in this snapshot.
    pub object_ids: Vec<ObjectID>,
    /// Optional description.
    pub description: Option<String>,
    /// Content hash of the snapshot for integrity verification.
    pub content_hash: [u8; 32],
}

impl ViewSnapshot {
    /// Create a new snapshot from a set of object IDs.
    pub fn new(
        name: String,
        query: String,
        object_ids: Vec<ObjectID>,
        created_by: [u8; 32],
    ) -> Self {
        let content_hash = Self::compute_hash(&object_ids);

        Self {
            id: SnapshotID::new(),
            source_view: None,
            name,
            query,
            created_at: timestamp_now(),
            created_by,
            object_ids,
            description: None,
            content_hash,
        }
    }

    /// Create a snapshot from a view.
    pub fn from_view(view: &View, object_ids: Vec<ObjectID>, created_by: [u8; 32]) -> Self {
        let content_hash = Self::compute_hash(&object_ids);

        Self {
            id: SnapshotID::new(),
            source_view: Some(view.id),
            name: format!("{} (snapshot)", view.name),
            query: view.query.clone(),
            created_at: timestamp_now(),
            created_by,
            object_ids,
            description: view.description.clone(),
            content_hash,
        }
    }

    /// Add a description.
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Get the number of objects in the snapshot.
    pub fn len(&self) -> usize {
        self.object_ids.len()
    }

    /// Check if the snapshot is empty.
    pub fn is_empty(&self) -> bool {
        self.object_ids.is_empty()
    }

    /// Verify the integrity of the snapshot.
    pub fn verify_integrity(&self) -> bool {
        let computed = Self::compute_hash(&self.object_ids);
        computed == self.content_hash
    }

    /// Check if an object is in this snapshot.
    pub fn contains(&self, object_id: &ObjectID) -> bool {
        self.object_ids.contains(object_id)
    }

    /// Iterate over object IDs in the snapshot.
    pub fn iter(&self) -> impl Iterator<Item = &ObjectID> {
        self.object_ids.iter()
    }

    /// Compute a hash of the object IDs for integrity verification.
    fn compute_hash(object_ids: &[ObjectID]) -> [u8; 32] {
        use blake3::Hasher;

        let mut hasher = Hasher::new();
        for id in object_ids {
            hasher.update(id.as_bytes());
        }
        *hasher.finalize().as_bytes()
    }

    /// Compute the difference between two snapshots.
    pub fn diff(&self, other: &ViewSnapshot) -> SnapshotDiff {
        use std::collections::HashSet;

        let self_set: HashSet<_> = self.object_ids.iter().collect();
        let other_set: HashSet<_> = other.object_ids.iter().collect();

        let added: Vec<ObjectID> = other_set.difference(&self_set).map(|id| **id).collect();
        let removed: Vec<ObjectID> = self_set.difference(&other_set).map(|id| **id).collect();
        let common: Vec<ObjectID> = self_set.intersection(&other_set).map(|id| **id).collect();

        SnapshotDiff {
            from: self.id,
            to: other.id,
            added,
            removed,
            common_count: common.len(),
        }
    }
}

/// Difference between two snapshots.
#[derive(Debug, Clone)]
pub struct SnapshotDiff {
    /// The source snapshot.
    pub from: SnapshotID,
    /// The target snapshot.
    pub to: SnapshotID,
    /// Objects added in the target.
    pub added: Vec<ObjectID>,
    /// Objects removed in the target.
    pub removed: Vec<ObjectID>,
    /// Number of objects common to both.
    pub common_count: usize,
}

impl SnapshotDiff {
    /// Check if there are no changes.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }

    /// Get the total number of changes.
    pub fn change_count(&self) -> usize {
        self.added.len() + self.removed.len()
    }
}

impl std::fmt::Display for SnapshotDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Snapshot diff: {} -> {}", self.from, self.to)?;
        writeln!(f, "  Added: {}", self.added.len())?;
        writeln!(f, "  Removed: {}", self.removed.len())?;
        writeln!(f, "  Common: {}", self.common_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_actor() -> [u8; 32] {
        [0u8; 32]
    }

    fn test_object_ids(count: usize) -> Vec<ObjectID> {
        (0..count).map(|_| ObjectID::new()).collect()
    }

    #[test]
    fn test_snapshot_creation() {
        let objects = test_object_ids(5);
        let snapshot = ViewSnapshot::new(
            "Test Snapshot".to_string(),
            "tag:project".to_string(),
            objects.clone(),
            test_actor(),
        );

        assert_eq!(snapshot.name, "Test Snapshot");
        assert_eq!(snapshot.len(), 5);
        assert!(!snapshot.is_empty());
        assert!(snapshot.verify_integrity());
    }

    #[test]
    fn test_snapshot_contains() {
        let objects = test_object_ids(3);
        let other = ObjectID::new();

        let snapshot = ViewSnapshot::new(
            "Test".to_string(),
            "tag:test".to_string(),
            objects.clone(),
            test_actor(),
        );

        assert!(snapshot.contains(&objects[0]));
        assert!(snapshot.contains(&objects[1]));
        assert!(snapshot.contains(&objects[2]));
        assert!(!snapshot.contains(&other));
    }

    #[test]
    fn test_snapshot_integrity() {
        let objects = test_object_ids(5);
        let mut snapshot = ViewSnapshot::new(
            "Test".to_string(),
            "tag:test".to_string(),
            objects,
            test_actor(),
        );

        assert!(snapshot.verify_integrity());

        // Tamper with the data
        snapshot.object_ids.push(ObjectID::new());
        assert!(!snapshot.verify_integrity());
    }

    #[test]
    fn test_snapshot_diff() {
        let common = test_object_ids(3);
        let only_in_a = test_object_ids(2);
        let only_in_b = test_object_ids(2);

        let mut objects_a = common.clone();
        objects_a.extend(only_in_a.clone());

        let mut objects_b = common.clone();
        objects_b.extend(only_in_b.clone());

        let snapshot_a = ViewSnapshot::new(
            "A".to_string(),
            "tag:a".to_string(),
            objects_a,
            test_actor(),
        );
        let snapshot_b = ViewSnapshot::new(
            "B".to_string(),
            "tag:b".to_string(),
            objects_b,
            test_actor(),
        );

        let diff = snapshot_a.diff(&snapshot_b);

        assert_eq!(diff.common_count, 3);
        assert_eq!(diff.added.len(), 2); // only_in_b
        assert_eq!(diff.removed.len(), 2); // only_in_a
        assert!(!diff.is_empty());
        assert_eq!(diff.change_count(), 4);
    }

    #[test]
    fn test_snapshot_diff_no_changes() {
        let objects = test_object_ids(5);

        let snapshot_a = ViewSnapshot::new(
            "A".to_string(),
            "tag:a".to_string(),
            objects.clone(),
            test_actor(),
        );
        let snapshot_b =
            ViewSnapshot::new("B".to_string(), "tag:b".to_string(), objects, test_actor());

        let diff = snapshot_a.diff(&snapshot_b);

        assert!(diff.is_empty());
        assert_eq!(diff.common_count, 5);
    }

    #[test]
    fn test_snapshot_iter() {
        let objects = test_object_ids(3);
        let snapshot = ViewSnapshot::new(
            "Test".to_string(),
            "tag:test".to_string(),
            objects.clone(),
            test_actor(),
        );

        let collected: Vec<&ObjectID> = snapshot.iter().collect();
        assert_eq!(collected.len(), 3);
    }

    #[test]
    fn test_snapshot_from_view() {
        let view = crate::views::View::new(
            "My View".to_string(),
            "tag:project".to_string(),
            test_actor(),
        );

        let objects = test_object_ids(5);
        let snapshot = ViewSnapshot::from_view(&view, objects, test_actor());

        assert_eq!(snapshot.source_view, Some(view.id));
        assert!(snapshot.name.contains("snapshot"));
    }
}
