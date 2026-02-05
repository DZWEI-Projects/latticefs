//! Dynamic view resolution and nested query composition.

use crate::error::{LatticeError, Result};
use crate::query::{parse, Expr, Query};
use crate::storage::MetadataStore;
use crate::views::{View, ViewID};
use std::collections::{HashSet, VecDeque};

/// Maximum allowed parent-chain depth for nested views.
pub const MAX_VIEW_NESTING_DEPTH: usize = 32;

/// Resolve a dynamic view from a reference (UUID, `a/b/c` path, or bare name).
pub fn resolve_dynamic_view_reference(store: &MetadataStore, reference: &str) -> Result<View> {
    if let Ok(id) = reference.parse::<ViewID>() {
        return store.load_view_by_id(&id);
    }

    if reference.contains('/') {
        return resolve_view_path(store, reference);
    }

    resolve_unique_name(store, reference)
}

/// Resolve a dynamic view using a slash-separated path from the dynamic view root.
pub fn resolve_view_path(store: &MetadataStore, path: &str) -> Result<View> {
    let segments: Vec<&str> = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.is_empty() {
        return Err(LatticeError::InvalidViewQuery(
            "View path cannot be empty".to_string(),
        ));
    }

    let mut parent_id: Option<ViewID> = None;
    let mut current: Option<View> = None;

    for segment in segments {
        let children = store.list_children(parent_id)?;
        let matches: Vec<View> = children
            .into_iter()
            .filter(|candidate| candidate.name == segment)
            .collect();

        current = match matches.as_slice() {
            [] => {
                return Err(LatticeError::ViewNotFound {
                    name: path.to_string(),
                });
            }
            [single] => Some(single.clone()),
            _ => {
                return Err(LatticeError::InvalidViewQuery(format!(
                    "Ambiguous view path segment '{}'",
                    segment
                )));
            }
        };

        parent_id = current.as_ref().map(|view| view.id);
    }

    current.ok_or_else(|| LatticeError::ViewNotFound {
        name: path.to_string(),
    })
}

/// Build the slash-separated path for a dynamic view.
pub fn view_full_path(store: &MetadataStore, id: ViewID) -> Result<String> {
    let chain = view_chain_by_id(store, id)?;
    Ok(chain
        .into_iter()
        .map(|view| view.name)
        .collect::<Vec<_>>()
        .join("/"))
}

/// Resolve and compose the effective query for a dynamic view ID.
pub fn resolve_effective_query_by_id(store: &MetadataStore, id: ViewID) -> Result<Query> {
    let view = store.load_view_by_id(&id)?;
    resolve_effective_query(store, &view)
}

/// Resolve and compose the effective query for a dynamic view.
pub fn resolve_effective_query(store: &MetadataStore, view: &View) -> Result<Query> {
    let chain = view_chain(store, view)?;
    compose_effective_query(&chain)
}

/// Validate that assigning `parent_id` to a view will not create cycles.
pub fn validate_parent_assignment(
    store: &MetadataStore,
    view_id: Option<ViewID>,
    parent_id: Option<ViewID>,
) -> Result<()> {
    let Some(mut current_id) = parent_id else {
        return Ok(());
    };

    if let Some(view_id) = view_id {
        if view_id == current_id {
            return Err(LatticeError::InvalidViewQuery(
                "A view cannot be its own parent".to_string(),
            ));
        }
    }

    let mut visited = HashSet::new();
    for _ in 0..=MAX_VIEW_NESTING_DEPTH {
        if !visited.insert(current_id) {
            return Err(LatticeError::InvalidViewQuery(
                "Cyclic parent relationship detected".to_string(),
            ));
        }

        let parent = store.load_view_by_id(&current_id)?;
        match parent.parent_id {
            Some(next_id) => {
                if let Some(view_id) = view_id {
                    if next_id == view_id {
                        return Err(LatticeError::InvalidViewQuery(
                            "Parent assignment would create a cycle".to_string(),
                        ));
                    }
                }
                current_id = next_id;
            }
            None => return Ok(()),
        }
    }

    Err(LatticeError::InvalidViewQuery(format!(
        "Nested view depth exceeded (max {})",
        MAX_VIEW_NESTING_DEPTH
    )))
}

/// Collect all descendants for a view using breadth-first traversal.
pub fn collect_descendants(store: &MetadataStore, root_id: ViewID) -> Result<Vec<View>> {
    let mut descendants = Vec::new();
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    queue.push_back((root_id, 0usize));
    visited.insert(root_id);

    while let Some((parent_id, depth)) = queue.pop_front() {
        if depth >= MAX_VIEW_NESTING_DEPTH {
            return Err(LatticeError::InvalidViewQuery(format!(
                "Nested view depth exceeded (max {})",
                MAX_VIEW_NESTING_DEPTH
            )));
        }

        for child in store.list_children(Some(parent_id))? {
            if !visited.insert(child.id) {
                return Err(LatticeError::InvalidViewQuery(
                    "Cyclic parent relationship detected".to_string(),
                ));
            }
            queue.push_back((child.id, depth + 1));
            descendants.push(child);
        }
    }

    Ok(descendants)
}

fn resolve_unique_name(store: &MetadataStore, name: &str) -> Result<View> {
    let matches = store.find_views_by_name(name)?;
    match matches.as_slice() {
        [] => Err(LatticeError::ViewNotFound {
            name: name.to_string(),
        }),
        [single] => Ok(single.clone()),
        _ => Err(LatticeError::InvalidViewQuery(format!(
            "Ambiguous view reference '{}'; use UUID or path",
            name
        ))),
    }
}

fn view_chain_by_id(store: &MetadataStore, id: ViewID) -> Result<Vec<View>> {
    let view = store.load_view_by_id(&id)?;
    view_chain(store, &view)
}

fn view_chain(store: &MetadataStore, leaf: &View) -> Result<Vec<View>> {
    let mut chain = Vec::new();
    let mut current = leaf.clone();
    let mut visited = HashSet::new();

    for _ in 0..=MAX_VIEW_NESTING_DEPTH {
        if !visited.insert(current.id) {
            return Err(LatticeError::InvalidViewQuery(
                "Cyclic parent relationship detected".to_string(),
            ));
        }

        chain.push(current.clone());

        if let Some(parent_id) = current.parent_id {
            current = store.load_view_by_id(&parent_id)?;
        } else {
            chain.reverse();
            return Ok(chain);
        }
    }

    Err(LatticeError::InvalidViewQuery(format!(
        "Nested view depth exceeded (max {})",
        MAX_VIEW_NESTING_DEPTH
    )))
}

fn compose_effective_query(chain: &[View]) -> Result<Query> {
    let mut effective: Option<Query> = None;

    for view in chain {
        let query = parse(&view.query)?;
        effective = Some(match effective {
            None => query,
            Some(parent_query) => Query {
                expr: Expr::and(parent_query.expr, query.expr),
                order: query.order.or(parent_query.order),
                limit: query.limit.or(parent_query.limit),
            },
        });
    }

    effective.ok_or_else(|| LatticeError::InvalidViewQuery("Empty view chain".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::View;
    use tempfile::tempdir;

    fn test_actor() -> [u8; 32] {
        [0u8; 32]
    }

    #[test]
    fn test_resolve_effective_query_child_overrides_parent_sort_and_limit() {
        let temp = tempdir().unwrap();
        let store = MetadataStore::open(temp.path()).unwrap();
        let actor = test_actor();

        let parent = View::new(
            "Media".to_string(),
            "tag:auto:mimetype:image/* SORT updated DESC LIMIT 100".to_string(),
            actor,
        );
        store.store_view(&parent).unwrap();

        let child = View::new(
            "Recent".to_string(),
            "updated within 7d SORT created ASC LIMIT 5".to_string(),
            actor,
        )
        .with_parent(parent.id);
        store.store_view(&child).unwrap();

        let query = resolve_effective_query(&store, &child).unwrap();
        assert!(matches!(query.expr, Expr::And(_, _)));
        assert_eq!(query.limit, Some(5));
        let order = query.order.expect("expected order");
        assert_eq!(order.field, crate::query::SortField::Created);
        assert_eq!(order.direction, crate::query::SortDirection::Asc);
    }

    #[test]
    fn test_resolve_dynamic_view_reference_errors_on_ambiguous_name() {
        let temp = tempdir().unwrap();
        let store = MetadataStore::open(temp.path()).unwrap();
        let actor = test_actor();

        let p1 = View::new("P1".to_string(), "tag:p1".to_string(), actor);
        store.store_view(&p1).unwrap();
        let p2 = View::new("P2".to_string(), "tag:p2".to_string(), actor);
        store.store_view(&p2).unwrap();

        let child1 = View::new("Same".to_string(), "tag:a".to_string(), actor).with_parent(p1.id);
        let child2 = View::new("Same".to_string(), "tag:b".to_string(), actor).with_parent(p2.id);
        store.store_view(&child1).unwrap();
        store.store_view(&child2).unwrap();

        let err = resolve_dynamic_view_reference(&store, "Same").unwrap_err();
        assert!(matches!(err, LatticeError::InvalidViewQuery(_)));
    }

    #[test]
    fn test_resolve_view_path() {
        let temp = tempdir().unwrap();
        let store = MetadataStore::open(temp.path()).unwrap();
        let actor = test_actor();

        let parent = View::new("Media".to_string(), "tag:media".to_string(), actor);
        store.store_view(&parent).unwrap();
        let child =
            View::new("Images".to_string(), "tag:image".to_string(), actor).with_parent(parent.id);
        store.store_view(&child).unwrap();

        let resolved = resolve_view_path(&store, "Media/Images").unwrap();
        assert_eq!(resolved.id, child.id);
    }

    #[test]
    fn test_validate_parent_assignment_detects_cycle() {
        let temp = tempdir().unwrap();
        let store = MetadataStore::open(temp.path()).unwrap();
        let actor = test_actor();

        let parent = View::new("Parent".to_string(), "tag:p".to_string(), actor);
        store.store_view(&parent).unwrap();
        let child =
            View::new("Child".to_string(), "tag:c".to_string(), actor).with_parent(parent.id);
        store.store_view(&child).unwrap();

        let err = validate_parent_assignment(&store, Some(parent.id), Some(child.id)).unwrap_err();
        assert!(matches!(err, LatticeError::InvalidViewQuery(_)));
    }
}
