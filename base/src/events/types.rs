use crate::model::{ActorID, ObjectID, VersionID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EventKind {
    ObjectCreated {
        object_id: String,
        version_id: String,
        actor: ActorID,
    },
    VersionAdded {
        object_id: String,
        version_id: String,
        parent_version: Option<String>,
        actor: ActorID,
    },
    ShareIssued {
        resource: String,
        capability_cid: String,
        audience: String,
        expires_at: u64,
    },
    PolicyViolation {
        object_id: Option<String>,
        permission: String,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: i64,
    #[serde(flatten)]
    pub kind: EventKind,
}

impl Event {
    pub fn object_created(object_id: &ObjectID, version_id: &VersionID, actor: ActorID) -> Self {
        Self {
            timestamp: crate::model::timestamp_now(),
            kind: EventKind::ObjectCreated {
                object_id: object_id.to_string(),
                version_id: version_id.to_string(),
                actor,
            },
        }
    }

    pub fn version_added(
        object_id: &ObjectID,
        version_id: &VersionID,
        parent_version: Option<&VersionID>,
        actor: ActorID,
    ) -> Self {
        Self {
            timestamp: crate::model::timestamp_now(),
            kind: EventKind::VersionAdded {
                object_id: object_id.to_string(),
                version_id: version_id.to_string(),
                parent_version: parent_version.map(|v| v.to_string()),
                actor,
            },
        }
    }

    pub fn share_issued(resource: String, capability_cid: String, audience: String, expires_at: u64) -> Self {
        Self {
            timestamp: crate::model::timestamp_now(),
            kind: EventKind::ShareIssued {
                resource,
                capability_cid,
                audience,
                expires_at,
            },
        }
    }

    pub fn policy_violation(object_id: Option<String>, permission: String, reason: String) -> Self {
        Self {
            timestamp: crate::model::timestamp_now(),
            kind: EventKind::PolicyViolation {
                object_id,
                permission,
                reason,
            },
        }
    }
}
