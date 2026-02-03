use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Policy identifier (stub for Phase 1)
/// Full policy engine will be implemented in Phase 4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyID(pub Uuid);

impl PolicyID {
    pub fn new() -> Self {
        PolicyID(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        PolicyID(uuid)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Default for PolicyID {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PolicyID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
