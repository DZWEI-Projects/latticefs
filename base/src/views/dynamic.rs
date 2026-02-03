//! Dynamic views for LatticeFS.
//!
//! Dynamic views are query-backed collections that update in real-time
//! as objects are added, modified, or removed.

use crate::error::Result;
use crate::model::{Object, ObjectID, State, Version};
use crate::query::{parse, Query, QueryEvaluator};
use crate::storage::MetadataStore;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for dynamic views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewConfig {
    /// Maximum number of results to return.
    pub max_results: Option<usize>,
    /// Cache TTL for view results (0 = no caching).
    pub cache_ttl_secs: u64,
    /// Whether to include archived objects.
    pub include_archived: bool,
    /// Minimum trust level for objects in this view.
    pub min_trust_level: Option<u8>,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            max_results: None,
            cache_ttl_secs: 60, // 1 minute default cache
            include_archived: false,
            min_trust_level: None,
        }
    }
}

impl ViewConfig {
    /// Create a config with no result limit.
    pub fn unlimited() -> Self {
        Self {
            max_results: None,
            ..Default::default()
        }
    }

    /// Set the maximum number of results.
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }

    /// Set the cache TTL.
    pub fn with_cache_ttl(mut self, secs: u64) -> Self {
        self.cache_ttl_secs = secs;
        self
    }

    /// Include archived objects.
    pub fn include_archived(mut self) -> Self {
        self.include_archived = true;
        self
    }

    /// Set minimum trust level.
    pub fn with_min_trust(mut self, level: u8) -> Self {
        self.min_trust_level = Some(level);
        self
    }
}

/// A dynamic view that evaluates a query on demand.
pub struct DynamicView<'a> {
    /// The parsed query.
    query: Query,
    /// The original query string.
    query_string: String,
    /// View configuration.
    config: ViewConfig,
    /// Reference to the metadata store.
    store: &'a MetadataStore,
    /// Cached results and timestamp.
    cache: Option<ViewCache>,
}

/// Cached view results.
struct ViewCache {
    results: Vec<ObjectID>,
    cached_at: std::time::Instant,
}

impl<'a> DynamicView<'a> {
    /// Create a new dynamic view from a query string.
    pub fn new(query: &str, store: &'a MetadataStore) -> Result<Self> {
        let parsed = parse(query)?;

        Ok(Self {
            query: parsed,
            query_string: query.to_string(),
            config: ViewConfig::default(),
            store,
            cache: None,
        })
    }

    /// Create with a specific configuration.
    pub fn with_config(mut self, config: ViewConfig) -> Self {
        self.config = config;
        self
    }

    /// Get the query string.
    pub fn query_string(&self) -> &str {
        &self.query_string
    }

    /// Evaluate the view and return matching objects.
    pub fn evaluate(&mut self) -> Result<Vec<ObjectID>> {
        // Check cache
        if let Some(cache) = &self.cache {
            if self.config.cache_ttl_secs > 0 {
                let ttl = Duration::from_secs(self.config.cache_ttl_secs);
                if cache.cached_at.elapsed() < ttl {
                    return Ok(cache.results.clone());
                }
            }
        }

        // Execute query
        let evaluator = QueryEvaluator::new(self.store);
        let mut results = evaluator.execute(&self.query)?;

        // Apply config filters
        results = self.apply_config_filters(results)?;

        // Update cache
        self.cache = Some(ViewCache {
            results: results.clone(),
            cached_at: std::time::Instant::now(),
        });

        Ok(results)
    }

    /// Apply configuration filters to results.
    fn apply_config_filters(&self, mut results: Vec<ObjectID>) -> Result<Vec<ObjectID>> {
        // Filter out archived objects unless explicitly included
        if !self.config.include_archived {
            results.retain(|id| {
                self.load_object(id)
                    .and_then(|object| self.load_version(&object.current_version))
                    .map(|version| version.state != State::Archived)
                    .unwrap_or(false)
            });
        }

        // Enforce minimum trust level if configured
        if let Some(min_trust) = self.config.min_trust_level {
            results.retain(|id| {
                self.load_object(id)
                    .map(|object| {
                        object
                            .tags
                            .iter()
                            .find(|t| t.key == "sys:trust")
                            .and_then(|t| t.value.parse::<u8>().ok())
                            .unwrap_or(75)
                    })
                    .map(|trust| trust >= min_trust)
                    .unwrap_or(false)
            });
        }

        // Apply max_results if set (query limit might already have been applied)
        if let Some(max) = self.config.max_results {
            results.truncate(max);
        }

        Ok(results)
    }

    /// Force a cache refresh on next evaluate.
    pub fn invalidate_cache(&mut self) {
        self.cache = None;
    }

    /// Check if the cache is valid.
    pub fn is_cached(&self) -> bool {
        if let Some(cache) = &self.cache {
            if self.config.cache_ttl_secs > 0 {
                let ttl = Duration::from_secs(self.config.cache_ttl_secs);
                return cache.cached_at.elapsed() < ttl;
            }
        }
        false
    }

    /// Get the number of cached results (if any).
    pub fn cached_count(&self) -> Option<usize> {
        self.cache.as_ref().map(|c| c.results.len())
    }

    fn load_object(&self, id: &ObjectID) -> Result<Object> {
        let bytes = self.store.load_object_bytes(id.as_bytes())?;
        let object: Object = bincode::deserialize(&bytes).map_err(|e| {
            crate::error::LatticeError::Serialization(format!(
                "Failed to deserialize object: {}",
                e
            ))
        })?;
        Ok(object)
    }

    fn load_version(&self, id: &crate::model::VersionID) -> Result<Version> {
        let bytes = self.store.load_version_bytes(id.as_bytes())?;
        let version: Version = bincode::deserialize(&bytes).map_err(|e| {
            crate::error::LatticeError::Serialization(format!(
                "Failed to deserialize version: {}",
                e
            ))
        })?;
        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Object, ObjectType, Tag, Version};
    use tempfile::tempdir;

    fn test_actor() -> [u8; 32] {
        [0u8; 32]
    }

    fn create_test_object(store: &MetadataStore, tags: Vec<(&str, &str)>) -> Result<ObjectID> {
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
    fn test_dynamic_view_evaluate() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let obj1 = create_test_object(&store, vec![("project", "phoenix")]).unwrap();
        let _obj2 = create_test_object(&store, vec![("project", "apollo")]).unwrap();

        let mut view = DynamicView::new("tag:project:phoenix", &store).unwrap();
        let results = view.evaluate().unwrap();

        assert_eq!(results.len(), 1);
        assert!(results.contains(&obj1));
    }

    #[test]
    fn test_dynamic_view_caching() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        create_test_object(&store, vec![("project", "phoenix")]).unwrap();

        let mut view = DynamicView::new("tag:project:phoenix", &store)
            .unwrap()
            .with_config(ViewConfig::default().with_cache_ttl(300));

        // First evaluation
        let results1 = view.evaluate().unwrap();
        assert!(view.is_cached());
        assert_eq!(view.cached_count(), Some(1));

        // Second evaluation should use cache
        let results2 = view.evaluate().unwrap();
        assert_eq!(results1, results2);
    }

    #[test]
    fn test_dynamic_view_invalidate() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        create_test_object(&store, vec![("project", "phoenix")]).unwrap();

        let mut view = DynamicView::new("tag:project:phoenix", &store).unwrap();

        view.evaluate().unwrap();
        assert!(view.is_cached());

        view.invalidate_cache();
        assert!(!view.is_cached());
    }

    #[test]
    fn test_view_config() {
        let config = ViewConfig::default()
            .with_max_results(10)
            .with_cache_ttl(120)
            .include_archived()
            .with_min_trust(50);

        assert_eq!(config.max_results, Some(10));
        assert_eq!(config.cache_ttl_secs, 120);
        assert!(config.include_archived);
        assert_eq!(config.min_trust_level, Some(50));
    }
}
