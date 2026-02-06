//! Built-in views for LatticeFS.
//!
//! Provides commonly used views that are always available:
//! - Recent: Objects updated within the last 7 days
//! - Projects: Objects tagged with project:*
//! - Drafts: Objects in draft state
//! - Review: Objects pending review
//! - Approved: Objects that have been approved

use crate::error::Result;
use crate::model::ObjectID;
use crate::query::{parse, QueryEvaluator};
use crate::storage::MetadataStore;

/// Supported locales for built-in view names and descriptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    /// English locale
    English,
    /// German locale (Deutsch)
    German,
}

impl Locale {
    /// Detect the locale from the operating system, falling back to German.
    pub fn from_system() -> Self {
        sys_locale::get_locale()
            .map(|locale| {
                if locale.starts_with("en") {
                    Locale::English
                } else {
                    // Default fallback to German for all non-English locales
                    Locale::German
                }
            })
            .unwrap_or(Locale::German)
    }
}

/// Built-in view types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinView {
    /// Objects updated within the last 7 days.
    Recent,
    /// Objects tagged with project:*.
    Projects,
    /// Objects in draft state.
    Drafts,
    /// Objects pending review.
    Review,
    /// Approved objects.
    Approved,
    /// All objects (no filter).
    All,
}

impl BuiltinView {
    /// Get the LQL query for this built-in view.
    pub fn query(&self) -> &'static str {
        match self {
            BuiltinView::Recent => "updated within 7d SORT updated DESC LIMIT 100",
            BuiltinView::Projects => "tag:project SORT updated DESC",
            BuiltinView::Drafts => "state:draft SORT updated DESC",
            BuiltinView::Review => "state:review SORT updated DESC",
            BuiltinView::Approved => "state:approved SORT updated DESC",
            BuiltinView::All => "trust >= 0", // Match all objects
        }
    }

    /// Get the display name for this view.
    pub fn name(&self) -> &'static str {
        match self {
            BuiltinView::Recent => "Recent",
            BuiltinView::Projects => "Projects",
            BuiltinView::Drafts => "Drafts",
            BuiltinView::Review => "Pending Review",
            BuiltinView::Approved => "Approved",
            BuiltinView::All => "All Objects",
        }
    }

    /// Get a description for this view.
    pub fn description(&self) -> &'static str {
        match self {
            BuiltinView::Recent => "Objects updated within the last 7 days",
            BuiltinView::Projects => "Objects tagged as projects",
            BuiltinView::Drafts => "Objects in draft state",
            BuiltinView::Review => "Objects pending review",
            BuiltinView::Approved => "Approved objects",
            BuiltinView::All => "All objects in the repository",
        }
    }

    /// Get the localized display name for this view.
    pub fn name_localized(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (BuiltinView::Recent, Locale::English) => "Recent",
            (BuiltinView::Recent, Locale::German) => "Neueste",
            (BuiltinView::Projects, Locale::English) => "Projects",
            (BuiltinView::Projects, Locale::German) => "Projekte",
            (BuiltinView::Drafts, Locale::English) => "Drafts",
            (BuiltinView::Drafts, Locale::German) => "Entwürfe",
            (BuiltinView::Review, Locale::English) => "Pending Review",
            (BuiltinView::Review, Locale::German) => "Zur Prüfung",
            (BuiltinView::Approved, Locale::English) => "Approved",
            (BuiltinView::Approved, Locale::German) => "Genehmigt",
            (BuiltinView::All, Locale::English) => "All Objects",
            (BuiltinView::All, Locale::German) => "Alle Objekte",
        }
    }

    /// Get the localized description for this view.
    pub fn description_localized(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (BuiltinView::Recent, Locale::English) => "Objects updated within the last 7 days",
            (BuiltinView::Recent, Locale::German) => {
                "Objekte, die in den letzten 7 Tagen aktualisiert wurden"
            }
            (BuiltinView::Projects, Locale::English) => "Objects tagged as projects",
            (BuiltinView::Projects, Locale::German) => {
                "Objekte, die als Projekte gekennzeichnet sind"
            }
            (BuiltinView::Drafts, Locale::English) => "Objects in draft state",
            (BuiltinView::Drafts, Locale::German) => "Objekte im Entwurfsstadium",
            (BuiltinView::Review, Locale::English) => "Objects pending review",
            (BuiltinView::Review, Locale::German) => "Objekte, die auf Prüfung warten",
            (BuiltinView::Approved, Locale::English) => "Approved objects",
            (BuiltinView::Approved, Locale::German) => "Genehmigte Objekte",
            (BuiltinView::All, Locale::English) => "All objects in the repository",
            (BuiltinView::All, Locale::German) => "Alle Objekte im Repository",
        }
    }

    /// List all built-in views.
    pub fn all() -> &'static [BuiltinView] {
        &[
            BuiltinView::Recent,
            BuiltinView::Projects,
            BuiltinView::Drafts,
            BuiltinView::Review,
            BuiltinView::Approved,
            BuiltinView::All,
        ]
    }

    /// Get a built-in view by name (supports English and German names).
    pub fn by_name(name: &str) -> Option<BuiltinView> {
        match name.to_lowercase().as_str() {
            // English names
            "recent" => Some(BuiltinView::Recent),
            "projects" => Some(BuiltinView::Projects),
            "drafts" => Some(BuiltinView::Drafts),
            "review" | "pending review" => Some(BuiltinView::Review),
            "approved" => Some(BuiltinView::Approved),
            "all" | "all objects" => Some(BuiltinView::All),
            // German names
            "neueste" => Some(BuiltinView::Recent),
            "projekte" => Some(BuiltinView::Projects),
            "entwürfe" => Some(BuiltinView::Drafts),
            "zur prüfung" => Some(BuiltinView::Review),
            "genehmigt" => Some(BuiltinView::Approved),
            "alle objekte" => Some(BuiltinView::All),
            _ => None,
        }
    }
}

/// Manager for built-in views.
pub struct BuiltinViews<'a> {
    store: &'a MetadataStore,
}

impl<'a> BuiltinViews<'a> {
    /// Create a new built-in views manager.
    pub fn new(store: &'a MetadataStore) -> Self {
        Self { store }
    }

    /// Evaluate a built-in view and return matching objects.
    pub fn evaluate(&self, view: BuiltinView) -> Result<Vec<ObjectID>> {
        let query = parse(view.query())?;
        let evaluator = QueryEvaluator::new(self.store);
        evaluator.execute(&query)
    }

    /// Get the count for a built-in view (without fetching all results).
    pub fn count(&self, view: BuiltinView) -> Result<usize> {
        // For now, just evaluate and count
        // In a production system, this would use a more efficient count query
        let results = self.evaluate(view)?;
        Ok(results.len())
    }

    /// Get summary info for all built-in views.
    pub fn summary(&self) -> Result<Vec<ViewSummary>> {
        let mut summaries = Vec::new();

        for view in BuiltinView::all() {
            let count = self.count(*view)?;
            summaries.push(ViewSummary {
                view: *view,
                name: view.name().to_string(),
                description: view.description().to_string(),
                count,
            });
        }

        Ok(summaries)
    }
}

/// Summary information for a view.
#[derive(Debug, Clone)]
pub struct ViewSummary {
    /// The built-in view type.
    pub view: BuiltinView,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Number of objects in the view.
    pub count: usize,
}

impl std::fmt::Display for ViewSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}) - {}", self.name, self.count, self.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Object, ObjectType, State, Tag, Version};
    use tempfile::tempdir;

    fn test_actor() -> [u8; 32] {
        [0u8; 32]
    }

    fn create_test_object(
        store: &MetadataStore,
        tags: Vec<(&str, &str)>,
        state: State,
    ) -> Result<ObjectID> {
        let object_id = ObjectID::new();
        let version_id = crate::model::VersionID::new();
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
        version.state = state;

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
        for (key, value) in tags {
            let tag_key = format!("{}:{}", key, value);
            store.add_to_tag_index(&tag_key, object_id.as_bytes())?;
        }

        Ok(object_id)
    }

    #[test]
    fn test_builtin_view_queries() {
        // Ensure all built-in queries are valid
        for view in BuiltinView::all() {
            let result = parse(view.query());
            assert!(
                result.is_ok(),
                "Failed to parse query for {:?}: {:?}",
                view,
                result.err()
            );
        }
    }

    #[test]
    fn test_builtin_view_by_name() {
        assert_eq!(BuiltinView::by_name("recent"), Some(BuiltinView::Recent));
        assert_eq!(BuiltinView::by_name("RECENT"), Some(BuiltinView::Recent));
        assert_eq!(
            BuiltinView::by_name("projects"),
            Some(BuiltinView::Projects)
        );
        assert_eq!(BuiltinView::by_name("nonexistent"), None);

        // Test German names
        assert_eq!(BuiltinView::by_name("neueste"), Some(BuiltinView::Recent));
        assert_eq!(
            BuiltinView::by_name("projekte"),
            Some(BuiltinView::Projects)
        );
        assert_eq!(BuiltinView::by_name("entwürfe"), Some(BuiltinView::Drafts));
    }

    #[test]
    fn test_localization() {
        // Test English
        assert_eq!(
            BuiltinView::Recent.name_localized(Locale::English),
            "Recent"
        );
        assert_eq!(
            BuiltinView::Projects.name_localized(Locale::English),
            "Projects"
        );
        assert!(BuiltinView::Recent
            .description_localized(Locale::English)
            .contains("7 days"));

        // Test German
        assert_eq!(
            BuiltinView::Recent.name_localized(Locale::German),
            "Neueste"
        );
        assert_eq!(
            BuiltinView::Projects.name_localized(Locale::German),
            "Projekte"
        );
        assert!(BuiltinView::Recent
            .description_localized(Locale::German)
            .contains("7 Tagen"));

        // Test system locale detection doesn't panic
        let _system_locale = Locale::from_system();
    }

    #[test]
    fn test_builtin_projects_view() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        // Create project objects
        let proj1 = create_test_object(&store, vec![("project", "phoenix")], State::Draft).unwrap();
        let proj2 = create_test_object(&store, vec![("project", "apollo")], State::Draft).unwrap();
        let _other = create_test_object(&store, vec![("category", "misc")], State::Draft).unwrap();

        let builtin = BuiltinViews::new(&store);
        let results = builtin.evaluate(BuiltinView::Projects).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.contains(&proj1));
        assert!(results.contains(&proj2));
    }

    #[test]
    fn test_builtin_drafts_view() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let draft = create_test_object(&store, vec![("type", "doc")], State::Draft).unwrap();
        let _approved = create_test_object(&store, vec![("type", "doc")], State::Approved).unwrap();

        let builtin = BuiltinViews::new(&store);
        let results = builtin.evaluate(BuiltinView::Drafts).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results.contains(&draft));
    }

    #[test]
    fn test_builtin_view_summary() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        create_test_object(&store, vec![("project", "test")], State::Draft).unwrap();

        let builtin = BuiltinViews::new(&store);
        let summaries = builtin.summary().unwrap();

        assert_eq!(summaries.len(), BuiltinView::all().len());

        for summary in &summaries {
            assert!(!summary.name.is_empty());
            assert!(!summary.description.is_empty());
        }
    }
}
