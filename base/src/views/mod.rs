//! Views module for LatticeFS.
//!
//! Views provide query-backed collections of objects:
//! - Dynamic views: Live-updating based on LQL queries
//! - Snapshot views: Immutable point-in-time captures
//! - Built-in views: Pre-defined views like "Recent" and "Projects"

pub mod builtin;
pub mod dynamic;
pub mod snapshot;

pub use builtin::{BuiltinView, BuiltinViews, Locale};
pub use dynamic::{DynamicView, ViewConfig};
pub use snapshot::ViewSnapshot;

use crate::error::{LatticeError, Result};
use crate::model::timestamp_now;
use crate::storage::MetadataStore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Default maximum depth allowed when traversing nested view parent chains.
///
/// This constant limits how many levels deep a view hierarchy can be nested
/// to prevent infinite recursion and excessive query complexity. When computing
/// the effective query for a nested view, the system walks up the parent chain
/// and stops at this depth limit, returning an error if exceeded.
pub const DEFAULT_MAX_PARENT_DEPTH: u32 = 16;

/// Logical operator for combining parent view queries when computing effective queries.
///
/// When a view has a parent, the effective query combines the view's own query
/// with all ancestor queries using this operator. For example:
/// - `And`: Combines queries with logical AND (intersection)
/// - `Or`: Combines queries with logical OR (union)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewJoinOperator {
    /// Combine queries with logical AND (intersection).
    And,
    /// Combine queries with logical OR (union).
    Or,
}

impl ViewJoinOperator {
    /// Returns the SQL/LQL keyword string for this operator.
    fn as_keyword(self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
        }
    }
}

/// Configuration options for computing effective queries from nested views.
///
/// When a view has a parent, the effective query is computed by walking up
/// the parent chain and combining all queries. This struct controls how
/// that composition is performed.
///
/// # Example
///
/// ```no_run
/// use base::views::{EffectiveQueryOptions, ViewJoinOperator};
///
/// // Use AND to combine parent queries (default)
/// let options = EffectiveQueryOptions::default();
///
/// // Use OR to combine parent queries
/// let options = EffectiveQueryOptions {
///     max_parent_depth: 10,
///     join_operator: ViewJoinOperator::Or,
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct EffectiveQueryOptions {
    /// Maximum depth allowed when traversing the parent chain.
    ///
    /// If the parent chain exceeds this depth, an error is returned.
    /// This prevents infinite recursion and overly complex queries.
    pub max_parent_depth: u32,
    /// Operator used to combine queries from the view and its parents.
    ///
    /// When multiple queries are combined, they are wrapped in parentheses
    /// and joined with this operator's keyword (e.g., "AND" or "OR").
    pub join_operator: ViewJoinOperator,
}

impl Default for EffectiveQueryOptions {
    /// Creates default options with `DEFAULT_MAX_PARENT_DEPTH` and `ViewJoinOperator::And`.
    fn default() -> Self {
        Self {
            max_parent_depth: DEFAULT_MAX_PARENT_DEPTH,
            join_operator: ViewJoinOperator::And,
        }
    }
}

/// Unique identifier for a view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ViewID(Uuid);

impl ViewID {
    /// Create a new random view ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from a UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Get as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Default for ViewID {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ViewID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Legacy view struct for migration (without parent_id).
#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyView {
    id: ViewID,
    name: String,
    description: Option<String>,
    query: String,
    created_at: i64,
    modified_at: i64,
    created_by: [u8; 32],
    config: ViewConfig,
}

/// A view definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    /// Unique identifier.
    pub id: ViewID,
    /// Human-readable name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// The LQL query that defines this view.
    pub query: String,
    /// When the view was created.
    pub created_at: i64,
    /// When the view was last modified.
    pub modified_at: i64,
    /// Who created the view (actor public key).
    pub created_by: [u8; 32],
    /// View configuration options.
    pub config: ViewConfig,
    /// Optional parent view ID for nesting.
    pub parent_id: Option<ViewID>,
}

impl From<LegacyView> for View {
    fn from(legacy: LegacyView) -> Self {
        Self {
            id: legacy.id,
            name: legacy.name,
            description: legacy.description,
            query: legacy.query,
            created_at: legacy.created_at,
            modified_at: legacy.modified_at,
            created_by: legacy.created_by,
            config: legacy.config,
            parent_id: None,
        }
    }
}

impl View {
    /// Create a new view.
    pub fn new(name: String, query: String, created_by: [u8; 32]) -> Self {
        let now = timestamp_now();
        Self {
            id: ViewID::new(),
            name,
            description: None,
            query,
            created_at: now,
            modified_at: now,
            created_by,
            config: ViewConfig::default(),
            parent_id: None,
        }
    }

    /// Add a description to the view.
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set the view configuration.
    pub fn with_config(mut self, config: ViewConfig) -> Self {
        self.config = config;
        self
    }

    /// Update the query.
    pub fn update_query(&mut self, query: String) {
        self.query = query;
        self.modified_at = timestamp_now();
    }

    /// Set the parent view ID for nesting.
    pub fn with_parent(mut self, parent_id: ViewID) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}

/// View store for persisting view definitions.
pub struct ViewStore {
    views: std::collections::HashMap<ViewID, View>,
    views_by_name: std::collections::HashMap<String, ViewID>,
}

impl ViewStore {
    /// Create a new empty view store.
    pub fn new() -> Self {
        Self {
            views: std::collections::HashMap::new(),
            views_by_name: std::collections::HashMap::new(),
        }
    }

    /// Store a view.
    pub fn store(&mut self, view: View) -> Result<()> {
        let id = view.id;
        let name = view.name.clone();

        // Check for name conflicts
        if let Some(existing_id) = self.views_by_name.get(&name) {
            if *existing_id != id {
                return Err(LatticeError::InvalidViewQuery(format!(
                    "View with name '{}' already exists",
                    name
                )));
            }
        }

        self.views_by_name.insert(name, id);
        self.views.insert(id, view);
        Ok(())
    }

    /// Get a view by ID.
    pub fn get(&self, id: &ViewID) -> Result<&View> {
        self.views
            .get(id)
            .ok_or_else(|| LatticeError::ViewNotFound {
                name: id.to_string(),
            })
    }

    /// Get a view by name.
    pub fn get_by_name(&self, name: &str) -> Result<&View> {
        let id = self
            .views_by_name
            .get(name)
            .ok_or_else(|| LatticeError::ViewNotFound {
                name: name.to_string(),
            })?;
        self.get(id)
    }

    /// Delete a view.
    pub fn delete(&mut self, id: &ViewID) -> Result<()> {
        if let Some(view) = self.views.remove(id) {
            self.views_by_name.remove(&view.name);
        }
        Ok(())
    }

    /// List all views.
    pub fn list(&self) -> Vec<&View> {
        self.views.values().collect()
    }

    /// Check if a view exists by name.
    pub fn exists(&self, name: &str) -> bool {
        self.views_by_name.contains_key(name)
    }
}

impl Default for ViewStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the effective LQL query for a view, combining parent queries with AND.
///
/// This walks up the parent chain and combines all queries with logical AND.
/// Returns an error if the parent chain exceeds the default depth limit or contains a cycle.
pub fn effective_query(store: &MetadataStore, view: &View) -> Result<String> {
    effective_query_with_options(store, view, EffectiveQueryOptions::default())
}

/// Compute the effective LQL query for a view with explicit composition options.
pub fn effective_query_with_options(
    store: &MetadataStore,
    view: &View,
    options: EffectiveQueryOptions,
) -> Result<String> {
    let mut queries = vec![view.query.clone()];
    let mut current = view.parent_id;
    let mut depth = 0u32;
    let mut visited = std::collections::HashSet::new();
    visited.insert(view.id);

    while let Some(pid) = current {
        if depth >= options.max_parent_depth {
            return Err(LatticeError::InvalidViewQuery(format!(
                "Parent chain depth exceeds maximum of {} levels",
                options.max_parent_depth
            )));
        }
        if visited.contains(&pid) {
            return Err(LatticeError::InvalidViewQuery(
                "Circular reference detected in parent chain".to_string(),
            ));
        }
        visited.insert(pid);

        let parent = store.load_view_by_id(&pid)?;
        queries.push(parent.query.clone());
        current = parent.parent_id;
        depth += 1;
    }

    queries.reverse();
    let joiner = format!(" {} ", options.join_operator.as_keyword());
    // Wrap each query in parens and join with the selected operator.
    Ok(queries
        .iter()
        .map(|q| format!("({})", q))
        .collect::<Vec<_>>()
        .join(&joiner))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_actor() -> [u8; 32] {
        [0u8; 32]
    }

    #[test]
    fn test_view_creation() {
        let view = View::new(
            "My Projects".to_string(),
            "tag:project".to_string(),
            test_actor(),
        )
        .with_description("All project-tagged objects".to_string());

        assert_eq!(view.name, "My Projects");
        assert_eq!(view.query, "tag:project");
        assert!(view.description.is_some());
    }

    #[test]
    fn test_view_store() {
        let mut store = ViewStore::new();

        let view = View::new(
            "Recent".to_string(),
            "updated within 7d".to_string(),
            test_actor(),
        );
        let id = view.id;

        store.store(view).unwrap();

        assert!(store.exists("Recent"));
        let retrieved = store.get(&id).unwrap();
        assert_eq!(retrieved.name, "Recent");

        let by_name = store.get_by_name("Recent").unwrap();
        assert_eq!(by_name.id, id);
    }

    #[test]
    fn test_view_store_name_conflict() {
        let mut store = ViewStore::new();

        let view1 = View::new("Test".to_string(), "tag:a".to_string(), test_actor());
        let view2 = View::new("Test".to_string(), "tag:b".to_string(), test_actor());

        store.store(view1).unwrap();
        let result = store.store(view2);

        assert!(result.is_err());
    }

    #[test]
    fn test_effective_query_or_join() {
        use crate::storage::MetadataStore;

        let temp_dir = tempfile::tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let parent = View::new(
            "Parent".to_string(),
            "tag:project:phoenix".to_string(),
            test_actor(),
        );
        let parent_id = parent.id;
        store.store_view(&parent).unwrap();

        let child = View::new(
            "Child".to_string(),
            "tag:kind:doc".to_string(),
            test_actor(),
        )
        .with_parent(parent_id);
        store.store_view(&child).unwrap();

        let effective = effective_query_with_options(
            &store,
            &child,
            EffectiveQueryOptions {
                max_parent_depth: 16,
                join_operator: ViewJoinOperator::Or,
            },
        )
        .unwrap();

        assert!(effective.contains(" OR "));
        assert!(!effective.contains(" AND "));
    }

    #[test]
    fn test_effective_query_with_custom_depth_limit() {
        use crate::storage::MetadataStore;

        let temp_dir = tempfile::tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let parent = View::new("Parent".to_string(), "tag:parent".to_string(), test_actor());
        let parent_id = parent.id;
        store.store_view(&parent).unwrap();

        let child = View::new("Child".to_string(), "tag:child".to_string(), test_actor())
            .with_parent(parent_id);
        store.store_view(&child).unwrap();

        let err = effective_query_with_options(
            &store,
            &child,
            EffectiveQueryOptions {
                max_parent_depth: 0,
                join_operator: ViewJoinOperator::And,
            },
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("Parent chain depth exceeds maximum of 0 levels"));
    }

    #[test]
    fn test_view_delete() {
        let mut store = ViewStore::new();

        let view = View::new("ToDelete".to_string(), "tag:temp".to_string(), test_actor());
        let id = view.id;

        store.store(view).unwrap();
        assert!(store.exists("ToDelete"));

        store.delete(&id).unwrap();
        assert!(!store.exists("ToDelete"));
    }

    #[test]
    fn test_view_with_parent() {
        let view = View::new("Child".to_string(), "tag:child".to_string(), test_actor());
        let parent_id = ViewID::new();
        let nested_view = view.with_parent(parent_id);

        assert_eq!(nested_view.parent_id, Some(parent_id));
    }
}
