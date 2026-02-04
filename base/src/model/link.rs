use crate::model::tag::{timestamp_now, ActorID, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Link identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinkID(pub Uuid);

impl LinkID {
    pub fn new() -> Self {
        LinkID(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        LinkID(uuid)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Default for LinkID {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for LinkID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of relationship between objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkType {
    /// A is derived from B (e.g., PDF from DOCX)
    DerivedFrom,
    /// A references B (citation, dependency)
    References,
    /// A belongs to collection B
    BelongsTo,
    /// A replaces B (successor)
    Replaces,
    /// General relationship
    Related,
}

impl LinkType {
    /// Check if this link type is transitive
    pub fn is_transitive(&self) -> bool {
        matches!(self, LinkType::BelongsTo | LinkType::Replaces)
    }

    /// Check if this link type is bidirectional
    pub fn is_bidirectional(&self) -> bool {
        matches!(self, LinkType::Related)
    }
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkType::DerivedFrom => write!(f, "derived-from"),
            LinkType::References => write!(f, "references"),
            LinkType::BelongsTo => write!(f, "belongs-to"),
            LinkType::Replaces => write!(f, "replaces"),
            LinkType::Related => write!(f, "related"),
        }
    }
}

impl std::str::FromStr for LinkType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "derived-from" | "derivedfrom" => Ok(LinkType::DerivedFrom),
            "references" => Ok(LinkType::References),
            "belongs-to" | "belongsto" => Ok(LinkType::BelongsTo),
            "replaces" => Ok(LinkType::Replaces),
            "related" => Ok(LinkType::Related),
            _ => Err(format!("Invalid link type: {}", s)),
        }
    }
}

/// Placeholder for ObjectID (uses raw bytes to avoid circular dependency)
pub type ObjectIDBytes = Vec<u8>;

/// Graph link between objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub id: LinkID,
    pub source: ObjectIDBytes,
    pub target: ObjectIDBytes,
    pub link_type: LinkType,
    pub created_at: Timestamp,
    pub created_by: ActorID,
    pub metadata: Option<HashMap<String, String>>,
}

impl Link {
    /// Create a new link
    pub fn new(
        source: ObjectIDBytes,
        target: ObjectIDBytes,
        link_type: LinkType,
        created_by: ActorID,
    ) -> Self {
        Link {
            id: LinkID::new(),
            source,
            target,
            link_type,
            created_at: timestamp_now(),
            created_by,
            metadata: None,
        }
    }

    /// Create a link with metadata
    pub fn with_metadata(
        source: ObjectIDBytes,
        target: ObjectIDBytes,
        link_type: LinkType,
        created_by: ActorID,
        metadata: HashMap<String, String>,
    ) -> Self {
        Link {
            id: LinkID::new(),
            source,
            target,
            link_type,
            created_at: timestamp_now(),
            created_by,
            metadata: Some(metadata),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_type_transitivity() {
        assert!(LinkType::BelongsTo.is_transitive());
        assert!(LinkType::Replaces.is_transitive());
        assert!(!LinkType::DerivedFrom.is_transitive());
        assert!(!LinkType::References.is_transitive());
        assert!(!LinkType::Related.is_transitive());
    }

    #[test]
    fn test_link_type_bidirectional() {
        assert!(LinkType::Related.is_bidirectional());
        assert!(!LinkType::DerivedFrom.is_bidirectional());
        assert!(!LinkType::References.is_bidirectional());
        assert!(!LinkType::BelongsTo.is_bidirectional());
        assert!(!LinkType::Replaces.is_bidirectional());
    }

    #[test]
    fn test_link_type_from_str() {
        assert_eq!(
            "derived-from".parse::<LinkType>().unwrap(),
            LinkType::DerivedFrom
        );
        assert_eq!(
            "references".parse::<LinkType>().unwrap(),
            LinkType::References
        );
        assert_eq!(
            "belongs-to".parse::<LinkType>().unwrap(),
            LinkType::BelongsTo
        );
        assert_eq!("replaces".parse::<LinkType>().unwrap(), LinkType::Replaces);
        assert_eq!("related".parse::<LinkType>().unwrap(), LinkType::Related);

        assert!("invalid".parse::<LinkType>().is_err());
    }

    #[test]
    fn test_link_type_display() {
        assert_eq!(LinkType::DerivedFrom.to_string(), "derived-from");
        assert_eq!(LinkType::References.to_string(), "references");
        assert_eq!(LinkType::BelongsTo.to_string(), "belongs-to");
        assert_eq!(LinkType::Replaces.to_string(), "replaces");
        assert_eq!(LinkType::Related.to_string(), "related");
    }

    #[test]
    fn test_link_creation() {
        let source = vec![1u8; 16];
        let target = vec![2u8; 16];
        let actor = [0u8; 32];

        let link = Link::new(source.clone(), target.clone(), LinkType::References, actor);

        assert_eq!(link.source, source);
        assert_eq!(link.target, target);
        assert_eq!(link.link_type, LinkType::References);
        assert!(link.metadata.is_none());
    }

    #[test]
    fn test_link_with_metadata() {
        let source = vec![1u8; 16];
        let target = vec![2u8; 16];
        let actor = [0u8; 32];

        let mut metadata = HashMap::new();
        metadata.insert("note".to_string(), "test link".to_string());

        let link = Link::with_metadata(
            source.clone(),
            target.clone(),
            LinkType::Related,
            actor,
            metadata.clone(),
        );

        assert_eq!(link.metadata, Some(metadata));
    }
}
