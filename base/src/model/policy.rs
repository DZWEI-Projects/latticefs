use crate::crypto::Permission;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Policy identifier.
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

/// Policy requirement constraints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Requirement {
    /// Requires approval from specific actors.
    ApprovalFrom(Vec<String>),
    /// Requires a minimum trust level (0-100).
    MinTrust(u8),
    /// Requires a specific tag to be present.
    RequireTag(String),
}

/// Policy definition (LFS-004).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: PolicyID,
    pub name: String,
    pub version: u32,
    pub allow: Vec<Permission>,
    pub deny: Vec<Permission>,
    pub require: Vec<Requirement>,
    /// Retention period in days (optional).
    pub retain_days: Option<u64>,
    /// Whether external sharing is permitted.
    pub external_share: bool,
}

impl Policy {
    pub fn new(name: String) -> Self {
        Self {
            id: PolicyID::new(),
            name,
            version: 1,
            allow: Vec::new(),
            deny: Vec::new(),
            require: Vec::new(),
            retain_days: None,
            external_share: false,
        }
    }

    /// Create a policy from a known template.
    pub fn from_template(name: String, template: PolicyTemplate) -> Self {
        let mut policy = Self::new(name);
        match template {
            PolicyTemplate::ProjectCollab => {
                policy.allow = vec![Permission::Read, Permission::Write, Permission::Comment];
                policy.deny = vec![Permission::Admin];
                policy.require = vec![Requirement::ApprovalFrom(vec![
                    "lead-architect".to_string(),
                ])];
                policy.retain_days = Some(365 * 7);
                policy.external_share = false;
            }
            PolicyTemplate::Personal => {
                policy.allow = vec![
                    Permission::Read,
                    Permission::Write,
                    Permission::Comment,
                    Permission::Share,
                ];
                policy.deny = vec![Permission::Admin];
                policy.require = Vec::new();
                policy.retain_days = None;
                policy.external_share = true;
            }
            PolicyTemplate::Compliance => {
                policy.allow = vec![Permission::Read];
                policy.deny = vec![Permission::Write, Permission::Comment, Permission::Share];
                policy.require = vec![Requirement::MinTrust(90)];
                policy.retain_days = Some(365 * 10);
                policy.external_share = false;
            }
        }
        policy
    }
}

/// Built-in policy templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyTemplate {
    ProjectCollab,
    Personal,
    Compliance,
}

impl std::str::FromStr for PolicyTemplate {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "project-collab" | "project" => Ok(PolicyTemplate::ProjectCollab),
            "personal" => Ok(PolicyTemplate::Personal),
            "compliance" => Ok(PolicyTemplate::Compliance),
            _ => Err(format!("Unknown policy template: {}", s)),
        }
    }
}
