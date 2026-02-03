use crate::error::{LatticeError, Result};
use serde::{Deserialize, Serialize};

/// Timestamp in microseconds since Unix epoch
pub type Timestamp = i64;

/// Actor ID (Ed25519 public key - stub for Phase 1)
pub type ActorID = [u8; 32];

/// Tag with hierarchical namespace support
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub key: String,
    pub value: String,
    pub created_at: Timestamp,
    pub created_by: ActorID,
}

impl Tag {
    /// Create a new tag
    pub fn new(key: String, value: String, created_by: ActorID) -> Self {
        Tag {
            key,
            value,
            created_at: timestamp_now(),
            created_by,
        }
    }

    /// Parse a tag from "key:value" format
    pub fn parse(input: &str, created_by: ActorID) -> Result<Self> {
        let parts: Vec<&str> = input.splitn(2, ':').collect();

        if parts.len() != 2 {
            return Err(LatticeError::Serialization(format!(
                "Invalid tag format: expected 'key:value', got '{}'",
                input
            )));
        }

        Ok(Tag::new(
            parts[0].to_string(),
            parts[1].to_string(),
            created_by,
        ))
    }

    /// Get the namespace (first part before ':')
    pub fn namespace(&self) -> &str {
        self.key.split(':').next().unwrap_or(&self.key)
    }

    /// Get full tag path (key:value)
    pub fn full_path(&self) -> String {
        format!("{}:{}", self.key, self.value)
    }

    /// Check if this tag matches a pattern
    /// Pattern matching rules:
    /// - "project" matches any tag with key starting with "project"
    /// - "project:phoenix" matches tag with key "project:phoenix" or key starting with "project:"
    /// - Full path matching: compares against key:value
    pub fn matches(&self, pattern: &str) -> bool {
        // Get full tag path
        let full_path = self.full_path();

        // Check if pattern matches the full path (hierarchical prefix match)
        if full_path.starts_with(pattern) {
            // Make sure it's a proper hierarchical boundary
            if full_path.len() == pattern.len() {
                return true; // Exact match
            }
            // Check if next char is ':' (proper hierarchy boundary)
            if full_path.as_bytes().get(pattern.len()) == Some(&b':') {
                return true;
            }
        }

        // Also check if pattern matches just the key
        if self.key.starts_with(pattern) {
            if self.key.len() == pattern.len() {
                return true; // Exact key match
            }
            // Check if next char is ':' (proper hierarchy boundary)
            if self.key.as_bytes().get(pattern.len()) == Some(&b':') {
                return true;
            }
        }

        false
    }

    /// Check if tag is in a reserved namespace
    pub fn is_system(&self) -> bool {
        self.key.starts_with("sys:")
    }

    pub fn is_auto_generated(&self) -> bool {
        self.key.starts_with("auto:")
    }

    pub fn is_user_defined(&self) -> bool {
        self.key.starts_with("user:")
    }
}

/// Get current timestamp in microseconds
pub fn timestamp_now() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_actor() -> ActorID {
        [0u8; 32]
    }

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("project".to_string(), "phoenix".to_string(), test_actor());

        assert_eq!(tag.key, "project");
        assert_eq!(tag.value, "phoenix");
        assert_eq!(tag.namespace(), "project");
    }

    #[test]
    fn test_tag_parse() {
        let tag = Tag::parse("project:phoenix", test_actor()).unwrap();

        assert_eq!(tag.key, "project");
        assert_eq!(tag.value, "phoenix");
    }

    #[test]
    fn test_tag_parse_nested() {
        let tag = Tag::parse("project:phoenix:deliverables", test_actor()).unwrap();

        assert_eq!(tag.key, "project");
        assert_eq!(tag.value, "phoenix:deliverables");
    }

    #[test]
    fn test_tag_parse_invalid() {
        let result = Tag::parse("invalid", test_actor());
        assert!(result.is_err());
    }

    #[test]
    fn test_tag_full_path() {
        let tag = Tag::new("project".to_string(), "phoenix".to_string(), test_actor());
        assert_eq!(tag.full_path(), "project:phoenix");
    }

    #[test]
    fn test_tag_matches() {
        let tag = Tag::new(
            "project:phoenix".to_string(),
            "active".to_string(),
            test_actor(),
        );

        // Should match namespace
        assert!(tag.matches("project"));
        assert!(tag.matches("project:phoenix"));

        // Should match with value
        assert!(tag.matches("project:phoenix:active"));

        // Should not match different patterns
        assert!(!tag.matches("tag"));
        assert!(!tag.matches("project:apollo"));
    }

    #[test]
    fn test_tag_namespaces() {
        let sys_tag = Tag::new("sys:version".to_string(), "1".to_string(), test_actor());
        let user_tag = Tag::new(
            "user:personal".to_string(),
            "test".to_string(),
            test_actor(),
        );
        let auto_tag = Tag::new("auto:exif".to_string(), "data".to_string(), test_actor());

        assert!(sys_tag.is_system());
        assert!(!sys_tag.is_user_defined());

        assert!(user_tag.is_user_defined());
        assert!(!user_tag.is_system());

        assert!(auto_tag.is_auto_generated());
        assert!(!auto_tag.is_user_defined());
    }
}
