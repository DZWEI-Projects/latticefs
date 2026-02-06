//! LQL Abstract Syntax Tree types.
//!
//! Defines the AST for Lattice Query Language (LQL) per LFS-002.
//! The AST represents parsed queries for evaluation against the object graph.

use crate::model::{ObjectID, State};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Top-level query structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    /// Filter expression.
    pub expr: Expr,
    /// Optional ordering.
    pub order: Option<OrderBy>,
    /// Optional result limit.
    pub limit: Option<usize>,
}

impl Query {
    /// Create a new query with just an expression.
    pub fn new(expr: Expr) -> Self {
        Self {
            expr,
            order: None,
            limit: None,
        }
    }

    /// Add ordering to the query.
    pub fn with_order(mut self, order: OrderBy) -> Self {
        self.order = Some(order);
        self
    }

    /// Add a result limit.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Boolean expression (filter predicate).
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Logical AND.
    And(Box<Expr>, Box<Expr>),
    /// Logical OR.
    Or(Box<Expr>, Box<Expr>),
    /// Logical NOT.
    Not(Box<Expr>),
    /// Leaf predicate.
    Predicate(Predicate),
}

impl Expr {
    /// Create an AND expression.
    pub fn and(left: Expr, right: Expr) -> Self {
        Expr::And(Box::new(left), Box::new(right))
    }

    /// Create an OR expression.
    pub fn or(left: Expr, right: Expr) -> Self {
        Expr::Or(Box::new(left), Box::new(right))
    }

    /// Create a NOT expression.
    pub fn not(expr: Expr) -> Self {
        Expr::Not(Box::new(expr))
    }

    /// Create a predicate expression.
    pub fn predicate(pred: Predicate) -> Self {
        Expr::Predicate(pred)
    }
}

/// Leaf predicate types.
#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    /// Tag predicate: `tag:project:phoenix`
    Tag { path: Vec<String> },

    /// Type predicate: `type:application/pdf` or `type:image/*`
    Type { mime: MimePattern },

    /// State predicate: `state:approved`
    State { state: State },

    /// Trust predicate: `trust >= trusted`
    Trust { op: CompareOp, level: TrustLevel },

    /// Time predicate: `updated within 7d`
    Time {
        field: TimeField,
        op: TimeOp,
        value: TimeValue,
    },

    /// Reference predicate: `ref:01934e3a...`
    Ref { reference: ObjectRef },

    /// References traversal: `references(<ref>)`
    References { target: ObjectRef },

    /// Closure traversal: `closure(<ref>)`
    Closure { root: ObjectRef },
}

impl Predicate {
    /// Create a tag predicate.
    pub fn tag(path: Vec<String>) -> Self {
        Predicate::Tag { path }
    }

    /// Create a type predicate.
    pub fn mime_type(major: &str, minor: Option<&str>) -> Self {
        Predicate::Type {
            mime: MimePattern {
                major: major.to_string(),
                minor: minor.map(|s| s.to_string()),
            },
        }
    }

    /// Create a state predicate.
    pub fn state(state: State) -> Self {
        Predicate::State { state }
    }
}

/// MIME type pattern with optional wildcard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MimePattern {
    /// Major type (e.g., "image", "application").
    pub major: String,
    /// Minor type, or None for wildcard (*).
    pub minor: Option<String>,
}

impl MimePattern {
    /// Create a new MIME pattern.
    pub fn new(major: &str, minor: Option<&str>) -> Self {
        Self {
            major: major.to_string(),
            minor: minor.map(|s| s.to_string()),
        }
    }

    /// Check if a MIME type matches this pattern.
    pub fn matches(&self, mime_type: &str) -> bool {
        let parts: Vec<&str> = mime_type.split('/').collect();
        if parts.len() != 2 {
            return false;
        }

        if self.major != "*" && self.major != parts[0] {
            return false;
        }

        match &self.minor {
            Some(minor) => minor == "*" || minor == parts[1],
            None => true, // Wildcard
        }
    }
}

impl std::fmt::Display for MimePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.minor {
            Some(minor) => write!(f, "{}/{}", self.major, minor),
            None => write!(f, "{}/*", self.major),
        }
    }
}

/// Comparison operators for trust/numeric predicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
    Eq, // =
    Ne, // !=
    Gt, // >
    Lt, // <
    Ge, // >=
    Le, // <=
}

impl std::fmt::Display for CompareOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompareOp::Eq => write!(f, "="),
            CompareOp::Ne => write!(f, "!="),
            CompareOp::Gt => write!(f, ">"),
            CompareOp::Lt => write!(f, "<"),
            CompareOp::Ge => write!(f, ">="),
            CompareOp::Le => write!(f, "<="),
        }
    }
}

/// Trust level for predicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Untrusted (0).
    Untrusted,
    /// Quarantined (25).
    Quarantined,
    /// Trusted (75).
    Trusted,
    /// Approved (100).
    Approved,
    /// Numeric value.
    Numeric(u8),
}

impl TrustLevel {
    /// Get the numeric value of the trust level.
    pub fn value(&self) -> u8 {
        match self {
            TrustLevel::Untrusted => 0,
            TrustLevel::Quarantined => 25,
            TrustLevel::Trusted => 75,
            TrustLevel::Approved => 100,
            TrustLevel::Numeric(v) => *v,
        }
    }
}

impl std::fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustLevel::Untrusted => write!(f, "untrusted"),
            TrustLevel::Quarantined => write!(f, "quarantined"),
            TrustLevel::Trusted => write!(f, "trusted"),
            TrustLevel::Approved => write!(f, "approved"),
            TrustLevel::Numeric(v) => write!(f, "{}", v),
        }
    }
}

/// Time field for predicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeField {
    Updated,
    Created,
}

impl std::fmt::Display for TimeField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeField::Updated => write!(f, "updated"),
            TimeField::Created => write!(f, "created"),
        }
    }
}

/// Time comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeOp {
    /// Within the last duration.
    Within,
    /// Before a timestamp.
    Before,
    /// After a timestamp.
    After,
    /// Between two timestamps (inclusive).
    Between,
}

impl std::fmt::Display for TimeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeOp::Within => write!(f, "within"),
            TimeOp::Before => write!(f, "before"),
            TimeOp::After => write!(f, "after"),
            TimeOp::Between => write!(f, "between"),
        }
    }
}

/// Time value for predicates.
#[derive(Debug, Clone, PartialEq)]
pub enum TimeValue {
    /// Duration (e.g., "7d").
    Duration(Duration),
    /// Absolute timestamp (Unix microseconds).
    Timestamp(i64),
    /// Inclusive range of timestamps (Unix microseconds).
    Range { start: i64, end: i64 },
}

/// Object reference for traversal predicates.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectRef {
    /// Object ID (UUID).
    Id(ObjectID),
    /// Content hash.
    Hash(String),
    /// Tag-based reference.
    Tag(Vec<String>),
    /// Alias reference.
    Alias(String),
}

impl std::fmt::Display for ObjectRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectRef::Id(id) => write!(f, "{}", id),
            ObjectRef::Hash(h) => write!(f, "{}", h),
            ObjectRef::Tag(path) => write!(f, "tag:{}", path.join(":")),
            ObjectRef::Alias(alias) => write!(f, "\"{}\"", alias),
        }
    }
}

/// Ordering specification.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderBy {
    /// Field to sort by.
    pub field: SortField,
    /// Sort direction.
    pub direction: SortDirection,
}

impl OrderBy {
    pub fn new(field: SortField, direction: SortDirection) -> Self {
        Self { field, direction }
    }
}

/// Fields that can be sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortField {
    Updated,
    Created,
    Size,
    Trust,
}

impl std::fmt::Display for SortField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortField::Updated => write!(f, "updated"),
            SortField::Created => write!(f, "created"),
            SortField::Size => write!(f, "size"),
            SortField::Trust => write!(f, "trust"),
        }
    }
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Desc
    }
}

impl std::fmt::Display for SortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortDirection::Asc => write!(f, "ASC"),
            SortDirection::Desc => write!(f, "DESC"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_pattern_exact_match() {
        let pattern = MimePattern::new("application", Some("pdf"));
        assert!(pattern.matches("application/pdf"));
        assert!(!pattern.matches("application/json"));
        assert!(!pattern.matches("image/pdf"));
    }

    #[test]
    fn test_mime_pattern_wildcard() {
        let pattern = MimePattern::new("image", None);
        assert!(pattern.matches("image/png"));
        assert!(pattern.matches("image/jpeg"));
        assert!(pattern.matches("image/gif"));
        assert!(!pattern.matches("application/pdf"));
    }

    #[test]
    fn test_mime_pattern_star_minor() {
        let pattern = MimePattern::new("text", Some("*"));
        assert!(pattern.matches("text/plain"));
        assert!(pattern.matches("text/html"));
        assert!(!pattern.matches("image/png"));
    }

    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::Approved > TrustLevel::Trusted);
        assert!(TrustLevel::Trusted > TrustLevel::Quarantined);
        assert!(TrustLevel::Quarantined > TrustLevel::Untrusted);
    }

    #[test]
    fn test_trust_level_values() {
        assert_eq!(TrustLevel::Untrusted.value(), 0);
        assert_eq!(TrustLevel::Quarantined.value(), 25);
        assert_eq!(TrustLevel::Trusted.value(), 75);
        assert_eq!(TrustLevel::Approved.value(), 100);
        assert_eq!(TrustLevel::Numeric(50).value(), 50);
    }

    #[test]
    fn test_query_builder() {
        let query = Query::new(Expr::predicate(Predicate::tag(vec![
            "project".to_string(),
            "phoenix".to_string(),
        ])))
        .with_order(OrderBy::new(SortField::Updated, SortDirection::Desc))
        .with_limit(10);

        assert!(query.order.is_some());
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_expr_combinators() {
        let tag = Expr::predicate(Predicate::tag(vec!["project".to_string()]));
        let state = Expr::predicate(Predicate::state(State::Approved));

        let combined = Expr::and(tag.clone(), Expr::not(state));

        match combined {
            Expr::And(left, right) => {
                assert!(matches!(*left, Expr::Predicate(Predicate::Tag { .. })));
                assert!(matches!(*right, Expr::Not(_)));
            }
            _ => panic!("Expected And expression"),
        }
    }
}
