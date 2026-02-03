//! LQL Query Evaluator.
//!
//! Executes LQL queries against the LatticeFS object graph.
//! Per LFS-002 section 8.

use crate::error::{LatticeError, Result};
use crate::model::{LinkType, Object, ObjectID, State, Tag, Version};
use crate::query::ast::*;
use crate::storage::content::hex_to_hash;
use crate::storage::MetadataStore;
use std::collections::HashSet;

/// Maximum traversal depth for graph queries.
const MAX_TRAVERSAL_DEPTH: usize = 10;

/// Query evaluator.
pub struct QueryEvaluator<'a> {
    store: &'a MetadataStore,
}

impl<'a> QueryEvaluator<'a> {
    /// Create a new query evaluator.
    pub fn new(store: &'a MetadataStore) -> Self {
        Self { store }
    }

    /// Execute a query and return matching object IDs.
    pub fn execute(&self, query: &Query) -> Result<Vec<ObjectID>> {
        // Evaluate the expression to get candidate objects
        let mut candidates = self.evaluate_expr(&query.expr)?;

        // Convert to Vec for sorting
        let mut results: Vec<ObjectID> = candidates.drain().collect();

        // Apply sorting
        if let Some(order) = &query.order {
            self.apply_sort(&mut results, order)?;
        }

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Evaluate a boolean expression.
    fn evaluate_expr(&self, expr: &Expr) -> Result<HashSet<ObjectID>> {
        match expr {
            Expr::And(left, right) => {
                let left_set = self.evaluate_expr(left)?;
                let right_set = self.evaluate_expr(right)?;
                Ok(left_set.intersection(&right_set).copied().collect())
            }
            Expr::Or(left, right) => {
                let left_set = self.evaluate_expr(left)?;
                let right_set = self.evaluate_expr(right)?;
                Ok(left_set.union(&right_set).copied().collect())
            }
            Expr::Not(inner) => {
                let inner_set = self.evaluate_expr(inner)?;
                let all_objects = self.get_all_object_ids()?;
                Ok(all_objects.difference(&inner_set).copied().collect())
            }
            Expr::Predicate(pred) => self.evaluate_predicate(pred),
        }
    }

    /// Evaluate a single predicate.
    fn evaluate_predicate(&self, pred: &Predicate) -> Result<HashSet<ObjectID>> {
        match pred {
            Predicate::Tag { path } => self.evaluate_tag_predicate(path),
            Predicate::Type { mime } => self.evaluate_type_predicate(mime),
            Predicate::State { state } => self.evaluate_state_predicate(state),
            Predicate::Trust { op, level } => self.evaluate_trust_predicate(op, level),
            Predicate::Time { field, op, value } => self.evaluate_time_predicate(field, op, value),
            Predicate::Ref { reference } => self.evaluate_ref_predicate(reference),
            Predicate::References { target } => self.evaluate_references_predicate(target),
            Predicate::Closure { root } => self.evaluate_closure_predicate(root),
        }
    }

    /// Evaluate a tag predicate.
    fn evaluate_tag_predicate(&self, path: &[String]) -> Result<HashSet<ObjectID>> {
        let tag_key = path.join(":");

        // Query the tag index
        let matching_ids = self.store.query_by_tag(&tag_key)?;

        let mut result = HashSet::new();
        for id_bytes in matching_ids {
            if id_bytes.len() == 16 {
                let id = ObjectID::from_uuid(uuid::Uuid::from_slice(&id_bytes).map_err(|e| {
                    LatticeError::Serialization(format!("Invalid ObjectID: {}", e))
                })?);
                result.insert(id);
            }
        }

        // Also check for hierarchical matches
        // e.g., tag:project should match tag:project:phoenix
        let all_objects = self.get_all_object_ids()?;
        for object_id in all_objects {
            if let Ok(object) = self.load_object(&object_id) {
                for tag in &object.tags {
                    if self.tag_matches(&tag_key, tag) {
                        result.insert(object_id);
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    /// Check if a tag matches a pattern (hierarchical).
    fn tag_matches(&self, pattern: &str, tag: &Tag) -> bool {
        let full_path = tag.full_path();

        // Exact match
        if full_path == pattern {
            return true;
        }

        // Hierarchical match: pattern "project" matches "project:phoenix"
        if full_path.starts_with(pattern) && full_path.chars().nth(pattern.len()) == Some(':') {
            return true;
        }

        false
    }

    /// Evaluate a type predicate.
    fn evaluate_type_predicate(&self, mime: &MimePattern) -> Result<HashSet<ObjectID>> {
        let mut result = HashSet::new();

        // We need to iterate over all objects and check their MIME type
        // In a real implementation, this would use an index
        for object_id in self.get_all_object_ids()? {
            if let Ok(object) = self.load_object(&object_id) {
                // Check for auto:mimetype tag
                for tag in &object.tags {
                    if tag.key == "auto:mimetype" && mime.matches(&tag.value) {
                        result.insert(object_id);
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    /// Evaluate a state predicate.
    fn evaluate_state_predicate(&self, state: &State) -> Result<HashSet<ObjectID>> {
        let mut result = HashSet::new();

        for object_id in self.get_all_object_ids()? {
            if let Ok(object) = self.load_object(&object_id) {
                if let Ok(version) = self.load_version(&object.current_version) {
                    if &version.state == state {
                        result.insert(object_id);
                    }
                }
            }
        }

        Ok(result)
    }

    /// Evaluate a trust predicate.
    fn evaluate_trust_predicate(
        &self,
        op: &CompareOp,
        level: &TrustLevel,
    ) -> Result<HashSet<ObjectID>> {
        let mut result = HashSet::new();
        let threshold = level.value();

        for object_id in self.get_all_object_ids()? {
            if let Ok(object) = self.load_object(&object_id) {
                // Get trust level from sys:trust tag
                let trust_value = object
                    .tags
                    .iter()
                    .find(|t| t.key == "sys:trust")
                    .and_then(|t| t.value.parse::<u8>().ok())
                    .unwrap_or(75); // Default to trusted

                let matches = match op {
                    CompareOp::Eq => trust_value == threshold,
                    CompareOp::Ne => trust_value != threshold,
                    CompareOp::Gt => trust_value > threshold,
                    CompareOp::Lt => trust_value < threshold,
                    CompareOp::Ge => trust_value >= threshold,
                    CompareOp::Le => trust_value <= threshold,
                };

                if matches {
                    result.insert(object_id);
                }
            }
        }

        Ok(result)
    }

    /// Evaluate a time predicate.
    fn evaluate_time_predicate(
        &self,
        field: &TimeField,
        op: &TimeOp,
        value: &TimeValue,
    ) -> Result<HashSet<ObjectID>> {
        let mut result = HashSet::new();
        let now = crate::model::timestamp_now();

        for object_id in self.get_all_object_ids()? {
            if let Ok(object) = self.load_object(&object_id) {
                let timestamp = match field {
                    TimeField::Created => object.created_at,
                    TimeField::Updated => {
                        // Get the current version's timestamp
                        self.load_version(&object.current_version)
                            .map(|v| v.created_at)
                            .unwrap_or(object.created_at)
                    }
                };

                let matches = match (op, value) {
                    (TimeOp::Within, TimeValue::Duration(d)) => {
                        let threshold = now - (d.as_secs() as i64 * 1_000_000);
                        timestamp >= threshold
                    }
                    (TimeOp::Before, TimeValue::Timestamp(t)) => timestamp < *t,
                    (TimeOp::After, TimeValue::Timestamp(t)) => timestamp > *t,
                    (TimeOp::Between, TimeValue::Range { start, end }) => {
                        let (min, max) = if start <= end {
                            (*start, *end)
                        } else {
                            (*end, *start)
                        };
                        timestamp >= min && timestamp <= max
                    }
                    _ => false,
                };

                if matches {
                    result.insert(object_id);
                }
            }
        }

        Ok(result)
    }

    /// Evaluate a ref predicate.
    fn evaluate_ref_predicate(&self, reference: &ObjectRef) -> Result<HashSet<ObjectID>> {
        let mut result = HashSet::new();

        match reference {
            ObjectRef::Id(id) => {
                if self.load_object(id).is_ok() {
                    result.insert(*id);
                }
            }
            ObjectRef::Hash(hash) => {
                // Search for objects with matching content hash
                // This scans versions since we don't have a hash index yet
                if let Ok(hash_bytes) = hex_to_hash(hash) {
                    for item in self.store.iter_all_versions() {
                        let (_key_bytes, value_bytes) = item?;
                        let version: Version = bincode::deserialize(&value_bytes).map_err(|e| {
                            LatticeError::Serialization(format!(
                                "Failed to deserialize version: {}",
                                e
                            ))
                        })?;

                        if version.chunk_root == hash_bytes || version.manifest_ref == hash_bytes {
                            result.insert(version.object_id);
                        }
                    }
                }
            }
            ObjectRef::Alias(alias) => {
                if let Some(id) = self.resolve_alias(alias)? {
                    result.insert(id);
                }
            }
            ObjectRef::Tag(path) => {
                // Return objects matching the tag
                return self.evaluate_tag_predicate(path);
            }
        }

        Ok(result)
    }

    /// Evaluate a references predicate (1-hop incoming links).
    fn evaluate_references_predicate(&self, target: &ObjectRef) -> Result<HashSet<ObjectID>> {
        let target_ids = self.resolve_object_ref(target)?;
        let mut result = HashSet::new();

        // Find all objects that link to any of the target objects
        for object_id in self.get_all_object_ids()? {
            if let Ok(object) = self.load_object(&object_id) {
                for link in &object.links {
                    if link.link_type != LinkType::References {
                        continue;
                    }
                    // Check if link target is in our target set
                    if let Ok(uuid) = uuid::Uuid::from_slice(&link.target) {
                        let link_target = ObjectID::from_uuid(uuid);
                        if target_ids.contains(&link_target) {
                            result.insert(object_id);
                            break;
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Evaluate a closure predicate (transitive closure).
    fn evaluate_closure_predicate(&self, root: &ObjectRef) -> Result<HashSet<ObjectID>> {
        let root_ids = self.resolve_object_ref(root)?;
        let mut result = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue: Vec<ObjectID> = root_ids.into_iter().collect();
        let mut depth = 0;

        while !queue.is_empty() && depth < MAX_TRAVERSAL_DEPTH {
            let mut next_queue = Vec::new();

            for object_id in queue {
                if visited.contains(&object_id) {
                    continue;
                }
                visited.insert(object_id);
                result.insert(object_id);

                // Find all objects this object links to
                if let Ok(object) = self.load_object(&object_id) {
                    for link in &object.links {
                        if !link_type_in_closure(&link.link_type) {
                            continue;
                        }
                        if let Ok(uuid) = uuid::Uuid::from_slice(&link.target) {
                            let link_target = ObjectID::from_uuid(uuid);
                            if !visited.contains(&link_target) {
                                next_queue.push(link_target);
                            }
                        }
                    }
                }
            }

            queue = next_queue;
            depth += 1;
        }

        if depth >= MAX_TRAVERSAL_DEPTH {
            return Err(LatticeError::TraversalDepthExceeded {
                max: MAX_TRAVERSAL_DEPTH,
            });
        }

        Ok(result)
    }

    /// Resolve an object reference to a set of object IDs.
    fn resolve_object_ref(&self, reference: &ObjectRef) -> Result<HashSet<ObjectID>> {
        match reference {
            ObjectRef::Id(id) => {
                let mut set = HashSet::new();
                set.insert(*id);
                Ok(set)
            }
            ObjectRef::Hash(hash) => {
                // Resolve by content hash
                if let Ok(hash_bytes) = hex_to_hash(hash) {
                    let mut set = HashSet::new();
                    for item in self.store.iter_all_versions() {
                        let (_key_bytes, value_bytes) = item?;
                        let version: Version = bincode::deserialize(&value_bytes).map_err(|e| {
                            LatticeError::Serialization(format!(
                                "Failed to deserialize version: {}",
                                e
                            ))
                        })?;

                        if version.chunk_root == hash_bytes || version.manifest_ref == hash_bytes {
                            set.insert(version.object_id);
                        }
                    }
                    Ok(set)
                } else {
                    Ok(HashSet::new())
                }
            }
            ObjectRef::Alias(alias) => {
                let mut set = HashSet::new();
                if let Some(id) = self.resolve_alias(alias)? {
                    set.insert(id);
                }
                Ok(set)
            }
            ObjectRef::Tag(path) => self.evaluate_tag_predicate(path),
        }
    }

    /// Apply sorting to results.
    fn apply_sort(&self, results: &mut [ObjectID], order: &OrderBy) -> Result<()> {
        // Load objects and their sort keys
        let mut keyed: Vec<(ObjectID, i64)> = results
            .iter()
            .filter_map(|id| {
                let key = self.get_sort_key(id, &order.field).ok()?;
                Some((*id, key))
            })
            .collect();

        // Sort by key
        match order.direction {
            SortDirection::Asc => keyed.sort_by_key(|(_, k)| *k),
            SortDirection::Desc => keyed.sort_by_key(|(_, k)| std::cmp::Reverse(*k)),
        }

        // Write back to results
        for (i, (id, _)) in keyed.into_iter().enumerate() {
            if i < results.len() {
                results[i] = id;
            }
        }

        Ok(())
    }

    /// Get the sort key for an object.
    fn get_sort_key(&self, object_id: &ObjectID, field: &SortField) -> Result<i64> {
        let object = self.load_object(object_id)?;

        match field {
            SortField::Created => Ok(object.created_at),
            SortField::Updated => {
                let version = self.load_version(&object.current_version)?;
                Ok(version.created_at)
            }
            SortField::Size => {
                let version = self.load_version(&object.current_version)?;
                Ok(version.size_bytes as i64)
            }
            SortField::Trust => {
                let trust = object
                    .tags
                    .iter()
                    .find(|t| t.key == "sys:trust")
                    .and_then(|t| t.value.parse::<i64>().ok())
                    .unwrap_or(75);
                Ok(trust)
            }
        }
    }

    /// Get all object IDs in the store.
    fn get_all_object_ids(&self) -> Result<HashSet<ObjectID>> {
        let mut result = HashSet::new();

        // Iterate through all objects in the store
        // In a real implementation, this would be more efficient
        for item in self.store.iter_all_versions() {
            let (_key_bytes, value_bytes) = item?;

            // Deserialize the version to get the object_id
            let version: Version = bincode::deserialize(&value_bytes).map_err(|e| {
                LatticeError::Serialization(format!("Failed to deserialize version: {}", e))
            })?;

            result.insert(version.object_id);
        }

        Ok(result)
    }

    /// Load an object by ID.
    fn load_object(&self, id: &ObjectID) -> Result<Object> {
        let bytes = self.store.load_object_bytes(id.as_bytes())?;
        let object: Object = bincode::deserialize(&bytes).map_err(|e| {
            LatticeError::Serialization(format!("Failed to deserialize object: {}", e))
        })?;
        Ok(object)
    }

    /// Load a version by ID.
    fn load_version(&self, id: &crate::model::VersionID) -> Result<Version> {
        let bytes = self.store.load_version_bytes(id.as_bytes())?;
        let version: Version = bincode::deserialize(&bytes).map_err(|e| {
            LatticeError::Serialization(format!("Failed to deserialize version: {}", e))
        })?;
        Ok(version)
    }

    fn resolve_alias(&self, alias: &str) -> Result<Option<ObjectID>> {
        let bytes = match self.store.resolve_alias(alias)? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        if bytes.len() != 16 {
            return Err(LatticeError::Serialization(format!(
                "Invalid alias object id length: {}",
                bytes.len()
            )));
        }

        let uuid = uuid::Uuid::from_slice(&bytes).map_err(|e| {
            LatticeError::Serialization(format!("Invalid alias object id: {}", e))
        })?;
        Ok(Some(ObjectID::from_uuid(uuid)))
    }
}

fn link_type_in_closure(link_type: &LinkType) -> bool {
    matches!(
        link_type,
        LinkType::DerivedFrom | LinkType::References | LinkType::BelongsTo | LinkType::Replaces
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ObjectType, Tag};
    use tempfile::tempdir;

    fn test_actor() -> [u8; 32] {
        [0u8; 32]
    }

    fn create_test_object(
        store: &MetadataStore,
        tags: Vec<(&str, &str)>,
    ) -> Result<ObjectID> {
        let object_id = ObjectID::new();
        let version_id = crate::model::VersionID::new();
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
            None,
        );

        let mut object = Object::new(ObjectType::Blob, version_id, test_actor());
        object.id = object_id;

        for (key, value) in &tags {
            object.add_tag(Tag::new(key.to_string(), value.to_string(), test_actor()));
        }

        // Store object and version
        let object_bytes = bincode::serialize(&object).unwrap();
        let version_bytes = bincode::serialize(&version).unwrap();

        store.store_object_bytes(object_id.as_bytes(), &object_bytes)?;
        store.store_version_bytes(version_id.as_bytes(), &version_bytes)?;

        // Add to tag index
        for (key, value) in &tags {
            let tag_key = format!("{}:{}", key, value);
            store.add_to_tag_index(&tag_key, object_id.as_bytes())?;
        }

        Ok(object_id)
    }

    #[test]
    fn test_simple_tag_query() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        // Create some test objects
        let obj1 = create_test_object(&store, vec![("project", "phoenix")]).unwrap();
        let _obj2 = create_test_object(&store, vec![("project", "apollo")]).unwrap();

        let evaluator = QueryEvaluator::new(&store);
        let query = crate::query::parser::parse("tag:project:phoenix").unwrap();

        let results = evaluator.execute(&query).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results.contains(&obj1));
    }

    #[test]
    fn test_and_query() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let obj1 = create_test_object(
            &store,
            vec![("project", "phoenix"), ("priority", "high")],
        )
        .unwrap();
        let _obj2 = create_test_object(&store, vec![("project", "phoenix")]).unwrap();

        let evaluator = QueryEvaluator::new(&store);
        let query =
            crate::query::parser::parse("tag:project:phoenix AND tag:priority:high").unwrap();

        let results = evaluator.execute(&query).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results.contains(&obj1));
    }

    #[test]
    fn test_or_query() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let obj1 = create_test_object(&store, vec![("project", "phoenix")]).unwrap();
        let obj2 = create_test_object(&store, vec![("project", "apollo")]).unwrap();

        let evaluator = QueryEvaluator::new(&store);
        let query =
            crate::query::parser::parse("tag:project:phoenix OR tag:project:apollo").unwrap();

        let results = evaluator.execute(&query).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.contains(&obj1));
        assert!(results.contains(&obj2));
    }

    #[test]
    fn test_not_query() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let _obj1 = create_test_object(&store, vec![("project", "phoenix")]).unwrap();
        let obj2 = create_test_object(&store, vec![("project", "apollo")]).unwrap();

        let evaluator = QueryEvaluator::new(&store);
        let query = crate::query::parser::parse("NOT tag:project:phoenix").unwrap();

        let results = evaluator.execute(&query).unwrap();

        // obj2 should be in results, obj1 should not
        assert!(results.contains(&obj2));
    }

    #[test]
    fn test_limit() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        // Create multiple objects
        for i in 0..5 {
            create_test_object(&store, vec![("batch", &format!("{}", i))]).unwrap();
        }

        let evaluator = QueryEvaluator::new(&store);
        let query = crate::query::parser::parse("tag:batch LIMIT 2").unwrap();

        let results = evaluator.execute(&query).unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_ref_alias_query() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let obj1 = create_test_object(&store, vec![("project", "phoenix")]).unwrap();
        store.set_alias("project-readme", obj1.as_bytes()).unwrap();

        let evaluator = QueryEvaluator::new(&store);
        let query = crate::query::parser::parse("ref:\"project-readme\"").unwrap();

        let results = evaluator.execute(&query).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results.contains(&obj1));
    }
}
