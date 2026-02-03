//! Version DAG (Directed Acyclic Graph) operations.
//!
//! Provides traversal, ancestry checking, and cycle detection for the version history.
//! Per LFS-004, versions form a DAG where each version can have a parent,
//! enabling branching and merging workflows.

use crate::error::{LatticeError, Result};
use crate::model::object::{ObjectID, Version, VersionID};
use crate::storage::MetadataStore;
use std::collections::{HashMap, HashSet, VecDeque};

/// Version DAG operations for traversing and validating version history.
pub struct VersionDAG<'a> {
    store: &'a MetadataStore,
}

impl<'a> VersionDAG<'a> {
    /// Create a new VersionDAG backed by a MetadataStore.
    pub fn new(store: &'a MetadataStore) -> Self {
        Self { store }
    }

    /// Load a version by ID from the store.
    fn load_version(&self, version_id: VersionID) -> Result<Version> {
        let bytes = self.store.load_version_bytes(version_id.as_bytes())?;
        let version: Version = bincode::deserialize(&bytes).map_err(|e| {
            LatticeError::Serialization(format!("Failed to deserialize version: {}", e))
        })?;
        Ok(version)
    }

    /// Check if `ancestor` is an ancestor of `descendant`.
    ///
    /// Walks up the parent chain from `descendant` to see if `ancestor` is found.
    /// Returns `true` if `ancestor` is found in the ancestry chain.
    pub fn is_ancestor(&self, ancestor: VersionID, descendant: VersionID) -> Result<bool> {
        if ancestor == descendant {
            return Ok(false); // A version is not its own ancestor
        }

        let mut current = descendant;
        let mut visited = HashSet::new();

        loop {
            if visited.contains(&current) {
                // Cycle detected (shouldn't happen in a valid DAG)
                return Err(LatticeError::CyclicVersion);
            }
            visited.insert(current);

            let version = self.load_version(current)?;

            match version.parent_version {
                Some(parent_id) => {
                    if parent_id == ancestor {
                        return Ok(true);
                    }
                    current = parent_id;
                }
                None => {
                    // Reached the root version, ancestor not found
                    return Ok(false);
                }
            }
        }
    }

    /// Get all ancestors of a version in order from immediate parent to root.
    ///
    /// Returns an empty vector if the version has no parent (root version).
    pub fn ancestors(&self, version_id: VersionID) -> Result<Vec<VersionID>> {
        let mut ancestors = Vec::new();
        let mut current = version_id;
        let mut visited = HashSet::new();

        loop {
            if visited.contains(&current) {
                return Err(LatticeError::CyclicVersion);
            }
            visited.insert(current);

            let version = self.load_version(current)?;

            match version.parent_version {
                Some(parent_id) => {
                    ancestors.push(parent_id);
                    current = parent_id;
                }
                None => {
                    break;
                }
            }
        }

        Ok(ancestors)
    }

    /// Get all descendants of a version.
    ///
    /// This requires scanning all versions to find those that have the given version
    /// in their ancestry chain. Note: This is O(n) where n is the number of versions.
    pub fn descendants(&self, version_id: VersionID, object_id: ObjectID) -> Result<Vec<VersionID>> {
        let mut descendants = Vec::new();

        // Get all version IDs for this object
        let all_version_ids = self.store.iter_versions_for_object(object_id.as_bytes())?;

        for vid_bytes in all_version_ids {
            let vid = VersionID::from_bytes(&vid_bytes)?;

            // Skip the version itself
            if vid == version_id {
                continue;
            }

            // Check if version_id is an ancestor of vid
            if self.is_ancestor(version_id, vid)? {
                descendants.push(vid);
            }
        }

        Ok(descendants)
    }

    /// Verify that adding a new version with the given parent would not create a cycle.
    ///
    /// This should be called before creating a new version to ensure DAG integrity.
    /// Per LFS-004, the DAG must be acyclic: no version can be its own ancestor.
    pub fn verify_acyclic(&self, new_version_id: VersionID, parent_id: Option<VersionID>) -> Result<()> {
        // If no parent, it's a root version - always acyclic
        let Some(parent) = parent_id else {
            return Ok(());
        };

        // The new version would create a cycle if the parent is already a descendant
        // of any version that would become the new version's descendant.
        // Since we're adding a new version (not yet stored), we just need to check
        // that the new version ID doesn't appear in the ancestry chain of the parent.

        // In practice, since the new version doesn't exist yet, it can't be in
        // the parent's ancestry. The only way to create a cycle would be if
        // someone tried to set the parent to itself.
        if new_version_id == parent {
            return Err(LatticeError::CyclicVersion);
        }

        // Additionally, verify the parent exists and we can traverse its chain
        // (this also validates the DAG is intact)
        let _ = self.ancestors(parent)?;

        Ok(())
    }

    /// Perform a topological sort of the given versions.
    ///
    /// Returns versions ordered such that each version appears after all its ancestors.
    /// Uses Kahn's algorithm for topological sorting.
    pub fn topological_sort(&self, versions: &[VersionID]) -> Result<Vec<VersionID>> {
        if versions.is_empty() {
            return Ok(Vec::new());
        }

        // Build adjacency list and in-degree count
        let version_set: HashSet<_> = versions.iter().copied().collect();
        let mut in_degree: HashMap<VersionID, usize> = HashMap::new();
        let mut children: HashMap<VersionID, Vec<VersionID>> = HashMap::new();

        // Initialize all versions with in-degree 0
        for &vid in versions {
            in_degree.insert(vid, 0);
            children.insert(vid, Vec::new());
        }

        // Build the graph
        for &vid in versions {
            let version = self.load_version(vid)?;
            if let Some(parent) = version.parent_version {
                if version_set.contains(&parent) {
                    // Increment in-degree of child (vid has an edge from parent)
                    *in_degree.entry(vid).or_insert(0) += 1;
                    // Add vid to parent's children
                    children.entry(parent).or_default().push(vid);
                }
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<VersionID> = VecDeque::new();
        let mut result: Vec<VersionID> = Vec::new();

        // Start with all vertices that have no incoming edges
        for (&vid, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(vid);
            }
        }

        while let Some(vid) = queue.pop_front() {
            result.push(vid);

            if let Some(child_list) = children.get(&vid) {
                for &child in child_list {
                    if let Some(degree) = in_degree.get_mut(&child) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(child);
                        }
                    }
                }
            }
        }

        // Check if all versions are included (cycle detection)
        if result.len() != versions.len() {
            return Err(LatticeError::CyclicVersion);
        }

        Ok(result)
    }

    /// Find the common ancestor of two versions.
    ///
    /// Returns the most recent common ancestor, or None if the versions
    /// don't share a common ancestor (i.e., they're from different root chains).
    pub fn common_ancestor(
        &self,
        version_a: VersionID,
        version_b: VersionID,
    ) -> Result<Option<VersionID>> {
        if version_a == version_b {
            return Ok(Some(version_a));
        }

        // Get all ancestors of version_a (including itself)
        let mut ancestors_a: HashSet<VersionID> = HashSet::new();
        ancestors_a.insert(version_a);
        for ancestor in self.ancestors(version_a)? {
            ancestors_a.insert(ancestor);
        }

        // Walk up from version_b and find first common ancestor
        if ancestors_a.contains(&version_b) {
            return Ok(Some(version_b));
        }

        for ancestor in self.ancestors(version_b)? {
            if ancestors_a.contains(&ancestor) {
                return Ok(Some(ancestor));
            }
        }

        Ok(None)
    }

    /// Get the depth of a version in the DAG (distance from root).
    ///
    /// Root versions have depth 0.
    pub fn depth(&self, version_id: VersionID) -> Result<usize> {
        Ok(self.ancestors(version_id)?.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::tag::ActorID;
    use crate::storage::content::Hash;
    use tempfile::tempdir;

    fn test_actor() -> ActorID {
        [0u8; 32]
    }

    fn test_hash() -> Hash {
        [1u8; 32]
    }

    fn create_test_version(
        object_id: ObjectID,
        parent: Option<VersionID>,
    ) -> Version {
        Version::new(
            object_id,
            parent,
            test_hash(),
            test_hash(),
            test_actor(),
            1000,
            5,
            None,
        )
    }

    #[test]
    fn test_is_ancestor_linear_chain() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id = ObjectID::new();

        // Create linear chain: v1 <- v2 <- v3
        let v1 = create_test_version(object_id, None);
        let v2 = create_test_version(object_id, Some(v1.id));
        let v3 = create_test_version(object_id, Some(v2.id));

        // Store versions
        let v1_bytes = bincode::serialize(&v1).unwrap();
        let v2_bytes = bincode::serialize(&v2).unwrap();
        let v3_bytes = bincode::serialize(&v3).unwrap();

        store.store_version_bytes(v1.id.as_bytes(), &v1_bytes).unwrap();
        store.store_version_bytes(v2.id.as_bytes(), &v2_bytes).unwrap();
        store.store_version_bytes(v3.id.as_bytes(), &v3_bytes).unwrap();

        let dag = VersionDAG::new(&store);

        // v1 is ancestor of v2
        assert!(dag.is_ancestor(v1.id, v2.id).unwrap());
        // v1 is ancestor of v3
        assert!(dag.is_ancestor(v1.id, v3.id).unwrap());
        // v2 is ancestor of v3
        assert!(dag.is_ancestor(v2.id, v3.id).unwrap());

        // v3 is NOT ancestor of v1
        assert!(!dag.is_ancestor(v3.id, v1.id).unwrap());
        // v3 is NOT ancestor of v2
        assert!(!dag.is_ancestor(v3.id, v2.id).unwrap());

        // v1 is NOT ancestor of itself
        assert!(!dag.is_ancestor(v1.id, v1.id).unwrap());
    }

    #[test]
    fn test_ancestors() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id = ObjectID::new();

        // Create linear chain: v1 <- v2 <- v3
        let v1 = create_test_version(object_id, None);
        let v2 = create_test_version(object_id, Some(v1.id));
        let v3 = create_test_version(object_id, Some(v2.id));

        // Store versions
        let v1_bytes = bincode::serialize(&v1).unwrap();
        let v2_bytes = bincode::serialize(&v2).unwrap();
        let v3_bytes = bincode::serialize(&v3).unwrap();

        store.store_version_bytes(v1.id.as_bytes(), &v1_bytes).unwrap();
        store.store_version_bytes(v2.id.as_bytes(), &v2_bytes).unwrap();
        store.store_version_bytes(v3.id.as_bytes(), &v3_bytes).unwrap();

        let dag = VersionDAG::new(&store);

        // v1 has no ancestors
        let ancestors_v1 = dag.ancestors(v1.id).unwrap();
        assert!(ancestors_v1.is_empty());

        // v2 has v1 as ancestor
        let ancestors_v2 = dag.ancestors(v2.id).unwrap();
        assert_eq!(ancestors_v2, vec![v1.id]);

        // v3 has v2 and v1 as ancestors (in order)
        let ancestors_v3 = dag.ancestors(v3.id).unwrap();
        assert_eq!(ancestors_v3, vec![v2.id, v1.id]);
    }

    #[test]
    fn test_verify_acyclic_valid() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id = ObjectID::new();

        // Create version v1
        let v1 = create_test_version(object_id, None);
        let v1_bytes = bincode::serialize(&v1).unwrap();
        store.store_version_bytes(v1.id.as_bytes(), &v1_bytes).unwrap();

        let dag = VersionDAG::new(&store);

        // New version with v1 as parent should be valid
        let new_version_id = VersionID::new();
        assert!(dag.verify_acyclic(new_version_id, Some(v1.id)).is_ok());

        // New version with no parent should be valid
        assert!(dag.verify_acyclic(new_version_id, None).is_ok());
    }

    #[test]
    fn test_verify_acyclic_self_parent() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let dag = VersionDAG::new(&store);

        // Trying to set a version as its own parent should fail
        let version_id = VersionID::new();
        let result = dag.verify_acyclic(version_id, Some(version_id));
        assert!(matches!(result, Err(LatticeError::CyclicVersion)));
    }

    #[test]
    fn test_topological_sort_linear() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id = ObjectID::new();

        // Create linear chain: v1 <- v2 <- v3
        let v1 = create_test_version(object_id, None);
        let v2 = create_test_version(object_id, Some(v1.id));
        let v3 = create_test_version(object_id, Some(v2.id));

        // Store versions
        let v1_bytes = bincode::serialize(&v1).unwrap();
        let v2_bytes = bincode::serialize(&v2).unwrap();
        let v3_bytes = bincode::serialize(&v3).unwrap();

        store.store_version_bytes(v1.id.as_bytes(), &v1_bytes).unwrap();
        store.store_version_bytes(v2.id.as_bytes(), &v2_bytes).unwrap();
        store.store_version_bytes(v3.id.as_bytes(), &v3_bytes).unwrap();

        let dag = VersionDAG::new(&store);

        // Topological sort should give [v1, v2, v3]
        let versions = vec![v3.id, v1.id, v2.id]; // Unsorted input
        let sorted = dag.topological_sort(&versions).unwrap();

        assert_eq!(sorted.len(), 3);
        // v1 must come before v2, v2 must come before v3
        let pos_v1 = sorted.iter().position(|&v| v == v1.id).unwrap();
        let pos_v2 = sorted.iter().position(|&v| v == v2.id).unwrap();
        let pos_v3 = sorted.iter().position(|&v| v == v3.id).unwrap();

        assert!(pos_v1 < pos_v2);
        assert!(pos_v2 < pos_v3);
    }

    #[test]
    fn test_common_ancestor() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id = ObjectID::new();

        // Create branching structure:
        //     v1 <- v2 <- v3
        //           ^
        //           \--- v4
        let v1 = create_test_version(object_id, None);
        let v2 = create_test_version(object_id, Some(v1.id));
        let v3 = create_test_version(object_id, Some(v2.id));
        let v4 = create_test_version(object_id, Some(v2.id));

        // Store versions
        let v1_bytes = bincode::serialize(&v1).unwrap();
        let v2_bytes = bincode::serialize(&v2).unwrap();
        let v3_bytes = bincode::serialize(&v3).unwrap();
        let v4_bytes = bincode::serialize(&v4).unwrap();

        store.store_version_bytes(v1.id.as_bytes(), &v1_bytes).unwrap();
        store.store_version_bytes(v2.id.as_bytes(), &v2_bytes).unwrap();
        store.store_version_bytes(v3.id.as_bytes(), &v3_bytes).unwrap();
        store.store_version_bytes(v4.id.as_bytes(), &v4_bytes).unwrap();

        let dag = VersionDAG::new(&store);

        // Common ancestor of v3 and v4 should be v2
        let common = dag.common_ancestor(v3.id, v4.id).unwrap();
        assert_eq!(common, Some(v2.id));

        // Common ancestor of v2 and v3 should be v2
        let common = dag.common_ancestor(v2.id, v3.id).unwrap();
        assert_eq!(common, Some(v2.id));

        // Common ancestor of v1 and v4 should be v1
        let common = dag.common_ancestor(v1.id, v4.id).unwrap();
        assert_eq!(common, Some(v1.id));
    }

    #[test]
    fn test_depth() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id = ObjectID::new();

        // Create linear chain: v1 <- v2 <- v3
        let v1 = create_test_version(object_id, None);
        let v2 = create_test_version(object_id, Some(v1.id));
        let v3 = create_test_version(object_id, Some(v2.id));

        // Store versions
        let v1_bytes = bincode::serialize(&v1).unwrap();
        let v2_bytes = bincode::serialize(&v2).unwrap();
        let v3_bytes = bincode::serialize(&v3).unwrap();

        store.store_version_bytes(v1.id.as_bytes(), &v1_bytes).unwrap();
        store.store_version_bytes(v2.id.as_bytes(), &v2_bytes).unwrap();
        store.store_version_bytes(v3.id.as_bytes(), &v3_bytes).unwrap();

        let dag = VersionDAG::new(&store);

        assert_eq!(dag.depth(v1.id).unwrap(), 0);
        assert_eq!(dag.depth(v2.id).unwrap(), 1);
        assert_eq!(dag.depth(v3.id).unwrap(), 2);
    }

    #[test]
    fn test_topological_sort_empty() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let dag = VersionDAG::new(&store);

        let sorted = dag.topological_sort(&[]).unwrap();
        assert!(sorted.is_empty());
    }
}
