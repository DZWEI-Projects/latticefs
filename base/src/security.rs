//! Shared policy helpers for executable gating and trust levels.

use crate::model::Tag;

/// Derive a numeric trust level from tags (default 75).
pub fn trust_level(tags: &[Tag]) -> u8 {
    tags.iter()
        .find(|t| t.key == "sys:trust")
        .and_then(|t| t.value.parse::<u8>().ok())
        .unwrap_or(75)
}

/// Check whether the object is tagged as executable.
pub fn has_executable_tag(tags: &[Tag]) -> bool {
    tags.iter()
        .any(|t| t.key == "auto:executable" && t.value == "true")
}

/// Whether an executable is allowed to be read/executed based on trust.
pub fn is_quarantined_executable(tags: &[Tag]) -> bool {
    has_executable_tag(tags) && trust_level(tags) < 90
}
