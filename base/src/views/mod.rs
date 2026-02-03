//! Views module for LatticeFS.
//!
//! Views provide query-backed collections of objects:
//! - Dynamic views: Live-updating based on LQL queries
//! - Snapshot views: Immutable point-in-time captures
//! - Built-in views: Pre-defined views like "Recent" and "Projects"

pub mod builtin;
pub mod dynamic;
pub mod snapshot;

pub use builtin::{BuiltinView, BuiltinViews};
pub use dynamic::{DynamicView, ViewConfig};
pub use snapshot::ViewSnapshot;

use crate::error::{LatticeError, Result};
use crate::model::timestamp_now;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
        self.views.get(id).ok_or_else(|| LatticeError::ViewNotFound {
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

        let view = View::new("Recent".to_string(), "updated within 7d".to_string(), test_actor());
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
    fn test_view_delete() {
        let mut store = ViewStore::new();

        let view = View::new("ToDelete".to_string(), "tag:temp".to_string(), test_actor());
        let id = view.id;

        store.store(view).unwrap();
        assert!(store.exists("ToDelete"));

        store.delete(&id).unwrap();
        assert!(!store.exists("ToDelete"));
    }
}
