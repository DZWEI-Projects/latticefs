//! LQL Query Explainer.
//!
//! Provides explanations for why objects matched or didn't match a query.
//! Per LFS-002 section 8.6 - Explainability.

use crate::error::Result;
use crate::model::{LinkType, Object, ObjectID, Tag, Version};
use crate::query::ast::*;
use crate::storage::MetadataStore;
use crate::storage::content::hex_to_hash;
use std::fmt;

/// Maximum traversal depth for graph queries.
const MAX_TRAVERSAL_DEPTH: usize = 10;

/// Explanation for why an object matched or didn't match a query.
#[derive(Debug, Clone)]
pub struct Explanation {
    /// The object being explained.
    pub object_id: ObjectID,
    /// Whether the object matched.
    pub matched: bool,
    /// The root reason node.
    pub reason: Reason,
}

impl fmt::Display for Explanation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Object {}: {}",
            self.object_id,
            if self.matched { "MATCHED" } else { "DID NOT MATCH" }
        )?;
        self.reason.fmt_indented(f, 0)
    }
}

/// A reason node in the explanation tree.
#[derive(Debug, Clone)]
pub enum Reason {
    /// AND expression result.
    And {
        matched: bool,
        left: Box<Reason>,
        right: Box<Reason>,
    },
    /// OR expression result.
    Or {
        matched: bool,
        left: Box<Reason>,
        right: Box<Reason>,
    },
    /// NOT expression result.
    Not { matched: bool, inner: Box<Reason> },
    /// Predicate result.
    Predicate {
        matched: bool,
        description: String,
        actual_value: Option<String>,
    },
}

impl Reason {
    fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let prefix = "  ".repeat(indent);
        let status = |matched: bool| if matched { "✓" } else { "✗" };

        match self {
            Reason::And { matched, left, right } => {
                writeln!(f, "{}{} AND", prefix, status(*matched))?;
                left.fmt_indented(f, indent + 1)?;
                right.fmt_indented(f, indent + 1)
            }
            Reason::Or { matched, left, right } => {
                writeln!(f, "{}{} OR", prefix, status(*matched))?;
                left.fmt_indented(f, indent + 1)?;
                right.fmt_indented(f, indent + 1)
            }
            Reason::Not { matched, inner } => {
                writeln!(f, "{}{} NOT", prefix, status(*matched))?;
                inner.fmt_indented(f, indent + 1)
            }
            Reason::Predicate {
                matched,
                description,
                actual_value,
            } => {
                if let Some(actual) = actual_value {
                    writeln!(f, "{}{} {} (actual: {})", prefix, status(*matched), description, actual)
                } else {
                    writeln!(f, "{}{} {}", prefix, status(*matched), description)
                }
            }
        }
    }
}

/// Query explainer.
pub struct Explainer<'a> {
    store: &'a MetadataStore,
}

impl<'a> Explainer<'a> {
    /// Create a new explainer.
    pub fn new(store: &'a MetadataStore) -> Self {
        Self { store }
    }

    /// Explain why an object matched or didn't match a query.
    pub fn explain(&self, object_id: &ObjectID, query: &Query) -> Result<Explanation> {
        let reason = self.explain_expr(object_id, &query.expr)?;
        let matched = reason_matched(&reason);

        Ok(Explanation {
            object_id: *object_id,
            matched,
            reason,
        })
    }

    /// Explain an expression for an object.
    fn explain_expr(&self, object_id: &ObjectID, expr: &Expr) -> Result<Reason> {
        match expr {
            Expr::And(left, right) => {
                let left_reason = self.explain_expr(object_id, left)?;
                let right_reason = self.explain_expr(object_id, right)?;
                let matched = reason_matched(&left_reason) && reason_matched(&right_reason);

                Ok(Reason::And {
                    matched,
                    left: Box::new(left_reason),
                    right: Box::new(right_reason),
                })
            }
            Expr::Or(left, right) => {
                let left_reason = self.explain_expr(object_id, left)?;
                let right_reason = self.explain_expr(object_id, right)?;
                let matched = reason_matched(&left_reason) || reason_matched(&right_reason);

                Ok(Reason::Or {
                    matched,
                    left: Box::new(left_reason),
                    right: Box::new(right_reason),
                })
            }
            Expr::Not(inner) => {
                let inner_reason = self.explain_expr(object_id, inner)?;
                let matched = !reason_matched(&inner_reason);

                Ok(Reason::Not {
                    matched,
                    inner: Box::new(inner_reason),
                })
            }
            Expr::Predicate(pred) => self.explain_predicate(object_id, pred),
        }
    }

    /// Explain a predicate for an object.
    fn explain_predicate(&self, object_id: &ObjectID, pred: &Predicate) -> Result<Reason> {
        let object = self.load_object(object_id)?;

        match pred {
            Predicate::Tag { path } => {
                let tag_pattern = path.join(":");
                let matching_tags: Vec<String> = object
                    .tags
                    .iter()
                    .filter(|t| self.tag_matches(&tag_pattern, t))
                    .map(|t| t.full_path())
                    .collect();

                let matched = !matching_tags.is_empty();
                let actual = if matched {
                    Some(matching_tags.join(", "))
                } else {
                    let all_tags: Vec<String> = object.tags.iter().map(|t| t.full_path()).collect();
                    if all_tags.is_empty() {
                        Some("no tags".to_string())
                    } else {
                        Some(format!("tags: {}", all_tags.join(", ")))
                    }
                };

                Ok(Reason::Predicate {
                    matched,
                    description: format!("tag:{}", tag_pattern),
                    actual_value: actual,
                })
            }

            Predicate::Type { mime } => {
                let actual_mime = object
                    .tags
                    .iter()
                    .find(|t| t.key == "auto:mimetype")
                    .map(|t| t.value.clone());

                let matched = actual_mime
                    .as_ref()
                    .map(|m| mime.matches(m))
                    .unwrap_or(false);

                Ok(Reason::Predicate {
                    matched,
                    description: format!("type:{}", mime),
                    actual_value: actual_mime.or_else(|| Some("no mimetype".to_string())),
                })
            }

            Predicate::State { state } => {
                let version = self.load_version(&object.current_version)?;
                let actual_state = &version.state;
                let matched = actual_state == state;

                Ok(Reason::Predicate {
                    matched,
                    description: format!("state:{:?}", state),
                    actual_value: Some(format!("{:?}", actual_state)),
                })
            }

            Predicate::Trust { op, level } => {
                let trust_value = object
                    .tags
                    .iter()
                    .find(|t| t.key == "sys:trust")
                    .and_then(|t| t.value.parse::<u8>().ok())
                    .unwrap_or(75);

                let threshold = level.value();
                let matched = match op {
                    CompareOp::Eq => trust_value == threshold,
                    CompareOp::Ne => trust_value != threshold,
                    CompareOp::Gt => trust_value > threshold,
                    CompareOp::Lt => trust_value < threshold,
                    CompareOp::Ge => trust_value >= threshold,
                    CompareOp::Le => trust_value <= threshold,
                };

                Ok(Reason::Predicate {
                    matched,
                    description: format!("trust {} {}", op, level),
                    actual_value: Some(format!("{}", trust_value)),
                })
            }

            Predicate::Time { field, op, value } => {
                let now = crate::model::timestamp_now();
                let timestamp = match field {
                    TimeField::Created => object.created_at,
                    TimeField::Updated => {
                        self.load_version(&object.current_version)
                            .map(|v| v.created_at)
                            .unwrap_or(object.created_at)
                    }
                };

                let matched = match (op, value) {
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

                let age_secs = (now - timestamp) / 1_000_000;
                let age_desc = if age_secs < 60 {
                    format!("{}s ago", age_secs)
                } else if age_secs < 3600 {
                    format!("{}m ago", age_secs / 60)
                } else if age_secs < 86400 {
                    format!("{}h ago", age_secs / 3600)
                } else {
                    format!("{}d ago", age_secs / 86400)
                };

                Ok(Reason::Predicate {
                    matched,
                    description: format!("{} {} {:?}", field, op, value),
                    actual_value: Some(age_desc),
                })
            }

            Predicate::Ref { reference } => {
                let matched = match reference {
                    ObjectRef::Id(id) => id == object_id,
                    ObjectRef::Alias(alias) => {
                        self.resolve_alias(alias)?
                            .map(|resolved| &resolved == object_id)
                            .unwrap_or(false)
                    }
                    _ => false, // Hash resolution requires content lookup
                };

                Ok(Reason::Predicate {
                    matched,
                    description: format!("ref:{}", reference),
                    actual_value: Some(format!("{}", object_id)),
                })
            }

            Predicate::References { target } => {
                let target_ids = self.resolve_object_ref(target)?;
                let refs_target = object.links.iter().any(|link| {
                    if link.link_type != LinkType::References {
                        return false;
                    }
                    uuid::Uuid::from_slice(&link.target)
                        .ok()
                        .map(|uuid| target_ids.contains(&ObjectID::from_uuid(uuid)))
                        .unwrap_or(false)
                });

                let link_targets: Vec<String> = object
                    .links
                    .iter()
                    .filter(|l| l.link_type == LinkType::References)
                    .filter_map(|l| {
                        uuid::Uuid::from_slice(&l.target)
                            .ok()
                            .map(|u| u.to_string()[..8].to_string())
                    })
                    .collect();

                Ok(Reason::Predicate {
                    matched: refs_target,
                    description: format!("references({})", target),
                    actual_value: if link_targets.is_empty() {
                        Some("no outgoing links".to_string())
                    } else {
                        Some(format!("links to: {}", link_targets.join(", ")))
                    },
                })
            }

            Predicate::Closure { root } => {
                let closure_ids = self.compute_closure(root)?;
                let matched = closure_ids.contains(object_id);

                Ok(Reason::Predicate {
                    matched,
                    description: format!("closure({})", root),
                    actual_value: Some(format!("closure size: {}", closure_ids.len())),
                })
            }
        }
    }

    /// Check if a tag matches a pattern.
    fn tag_matches(&self, pattern: &str, tag: &Tag) -> bool {
        let full_path = tag.full_path();

        // Exact match
        if full_path == pattern {
            return true;
        }

        // Hierarchical match
        if full_path.starts_with(pattern) && full_path.chars().nth(pattern.len()) == Some(':') {
            return true;
        }

        false
    }

    /// Resolve an object reference to a set of IDs.
    fn resolve_object_ref(
        &self,
        reference: &ObjectRef,
    ) -> Result<std::collections::HashSet<ObjectID>> {
        use std::collections::HashSet;

        match reference {
            ObjectRef::Id(id) => {
                let mut set = HashSet::new();
                set.insert(*id);
                Ok(set)
            }
            ObjectRef::Hash(hash) => {
                if let Ok(hash_bytes) = hex_to_hash(hash) {
                    let mut set = HashSet::new();
                    for item in self.store.iter_all_versions() {
                        let (_key_bytes, value_bytes) = item?;
                        let version: Version = bincode::deserialize(&value_bytes).map_err(|e| {
                            crate::error::LatticeError::Serialization(format!(
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
            ObjectRef::Tag(path) => {
                let tag_key = path.join(":");
                let matching_ids = self.store.query_by_tag(&tag_key)?;

                let mut result = HashSet::new();
                for id_bytes in matching_ids {
                    if id_bytes.len() == 16 {
                        if let Ok(uuid) = uuid::Uuid::from_slice(&id_bytes) {
                            result.insert(ObjectID::from_uuid(uuid));
                        }
                    }
                }
                Ok(result)
            }
            ObjectRef::Alias(alias) => {
                let mut set = HashSet::new();
                if let Some(id) = self.resolve_alias(alias)? {
                    set.insert(id);
                }
                Ok(set)
            }
        }
    }

    fn compute_closure(&self, root: &ObjectRef) -> Result<std::collections::HashSet<ObjectID>> {
        use std::collections::HashSet;

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
            return Err(crate::error::LatticeError::TraversalDepthExceeded {
                max: MAX_TRAVERSAL_DEPTH,
            });
        }

        Ok(result)
    }

    /// Load an object by ID.
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

    /// Load a version by ID.
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

    fn resolve_alias(&self, alias: &str) -> Result<Option<ObjectID>> {
        let bytes = match self.store.resolve_alias(alias)? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        if bytes.len() != 16 {
            return Err(crate::error::LatticeError::Serialization(format!(
                "Invalid alias object id length: {}",
                bytes.len()
            )));
        }

        let uuid = uuid::Uuid::from_slice(&bytes).map_err(|e| {
            crate::error::LatticeError::Serialization(format!("Invalid alias object id: {}", e))
        })?;
        Ok(Some(ObjectID::from_uuid(uuid)))
    }
}

/// Check if a reason indicates a match.
fn reason_matched(reason: &Reason) -> bool {
    match reason {
        Reason::And { matched, .. } => *matched,
        Reason::Or { matched, .. } => *matched,
        Reason::Not { matched, .. } => *matched,
        Reason::Predicate { matched, .. } => *matched,
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

        for (key, value) in tags {
            object.add_tag(Tag::new(key.to_string(), value.to_string(), test_actor()));
        }

        let object_bytes = bincode::serialize(&object).unwrap();
        let version_bytes = bincode::serialize(&version).unwrap();

        store.store_object_bytes(object_id.as_bytes(), &object_bytes)?;
        store.store_version_bytes(version_id.as_bytes(), &version_bytes)?;

        Ok(object_id)
    }

    #[test]
    fn test_explain_matching_tag() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id = create_test_object(&store, vec![("project", "phoenix")]).unwrap();

        let explainer = Explainer::new(&store);
        let query = crate::query::parser::parse("tag:project:phoenix").unwrap();

        let explanation = explainer.explain(&object_id, &query).unwrap();

        assert!(explanation.matched);
        println!("{}", explanation);
    }

    #[test]
    fn test_explain_non_matching_tag() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id = create_test_object(&store, vec![("project", "apollo")]).unwrap();

        let explainer = Explainer::new(&store);
        let query = crate::query::parser::parse("tag:project:phoenix").unwrap();

        let explanation = explainer.explain(&object_id, &query).unwrap();

        assert!(!explanation.matched);
        println!("{}", explanation);
    }

    #[test]
    fn test_explain_and_expression() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id =
            create_test_object(&store, vec![("project", "phoenix"), ("priority", "high")]).unwrap();

        let explainer = Explainer::new(&store);
        let query =
            crate::query::parser::parse("tag:project:phoenix AND tag:priority:high").unwrap();

        let explanation = explainer.explain(&object_id, &query).unwrap();

        assert!(explanation.matched);
        println!("{}", explanation);
    }

    #[test]
    fn test_explain_partial_and() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id = create_test_object(&store, vec![("project", "phoenix")]).unwrap();

        let explainer = Explainer::new(&store);
        let query =
            crate::query::parser::parse("tag:project:phoenix AND tag:priority:high").unwrap();

        let explanation = explainer.explain(&object_id, &query).unwrap();

        assert!(!explanation.matched);
        println!("{}", explanation);
    }

    #[test]
    fn test_explain_or_expression() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id = create_test_object(&store, vec![("project", "apollo")]).unwrap();

        let explainer = Explainer::new(&store);
        let query =
            crate::query::parser::parse("tag:project:phoenix OR tag:project:apollo").unwrap();

        let explanation = explainer.explain(&object_id, &query).unwrap();

        assert!(explanation.matched);
        println!("{}", explanation);
    }

    #[test]
    fn test_explain_not_expression() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id = create_test_object(&store, vec![("project", "apollo")]).unwrap();

        let explainer = Explainer::new(&store);
        let query = crate::query::parser::parse("NOT tag:project:phoenix").unwrap();

        let explanation = explainer.explain(&object_id, &query).unwrap();

        assert!(explanation.matched);
        println!("{}", explanation);
    }

    #[test]
    fn test_explain_display() {
        let temp_dir = tempdir().unwrap();
        let store = MetadataStore::open(temp_dir.path()).unwrap();

        let object_id =
            create_test_object(&store, vec![("project", "phoenix"), ("status", "active")]).unwrap();

        let explainer = Explainer::new(&store);
        let query =
            crate::query::parser::parse("tag:project:phoenix AND NOT tag:deleted").unwrap();

        let explanation = explainer.explain(&object_id, &query).unwrap();

        let output = format!("{}", explanation);
        assert!(output.contains("MATCHED"));
        assert!(output.contains("AND"));
        assert!(output.contains("NOT"));
    }
}
