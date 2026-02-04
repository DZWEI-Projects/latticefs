use crate::crypto::Permission;
use crate::error::{LatticeError, Result};
use crate::model::{Object, Policy, Requirement, Tag};
use crate::security::trust_level;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct PolicyContext<'a> {
    pub object: &'a Object,
    pub external_share: bool,
    pub approvals: Vec<String>,
}

impl<'a> PolicyContext<'a> {
    pub fn for_object(object: &'a Object) -> Self {
        let approvals = approvals_from_tags(&object.tags);
        Self {
            object,
            external_share: false,
            approvals,
        }
    }

    pub fn with_external_share(mut self, external_share: bool) -> Self {
        self.external_share = external_share;
        self
    }
}

#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub allowed: BTreeSet<Permission>,
    pub reasons: Vec<String>,
    pub requirements_ok: bool,
}

impl PolicyDecision {
    pub fn allows(&self, permission: Permission) -> bool {
        self.allowed.contains(&permission)
    }
}

#[derive(Debug, Default, Clone)]
pub struct PolicyEngine;

impl PolicyEngine {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate policies for a specific action (most restrictive wins).
    pub fn evaluate(
        &self,
        policies: &[Policy],
        context: &PolicyContext<'_>,
    ) -> PolicyDecision {
        let mut allowed: BTreeSet<Permission> = BTreeSet::from([
            Permission::Read,
            Permission::Comment,
            Permission::Write,
            Permission::Share,
            Permission::Admin,
        ]);
        let mut reasons = Vec::new();
        let mut requirements_ok = true;

        for policy in policies {
            // Apply allow list as restriction if provided.
            if !policy.allow.is_empty() {
                let allow_set: BTreeSet<Permission> = policy.allow.iter().copied().collect();
                allowed = allowed
                    .intersection(&allow_set)
                    .copied()
                    .collect::<BTreeSet<_>>();
            }

            // Apply deny list.
            for denied in &policy.deny {
                if allowed.remove(denied) {
                    reasons.push(format!(
                        "policy '{}' denies permission {}",
                        policy.name, denied
                    ));
                }
            }

            // External share restrictions.
            if context.external_share && !policy.external_share {
                if allowed.remove(&Permission::Share) {
                    reasons.push(format!(
                        "policy '{}' blocks external sharing",
                        policy.name
                    ));
                }
            }

            // Requirement constraints.
            for req in &policy.require {
                if !requirement_satisfied(req, context.object, &context.approvals) {
                    requirements_ok = false;
                    reasons.push(format!(
                        "policy '{}' requirement not satisfied: {:?}",
                        policy.name, req
                    ));
                }
            }
        }

        PolicyDecision {
            allowed,
            reasons,
            requirements_ok,
        }
    }

    /// Authorize a specific permission under policies.
    pub fn authorize(
        &self,
        policies: &[Policy],
        context: &PolicyContext<'_>,
        permission: Permission,
    ) -> Result<()> {
        let decision = self.evaluate(policies, context);

        // If any requirement failed, deny regardless of permission set.
        if !decision.requirements_ok {
            return Err(LatticeError::PolicyViolation {
                reason: decision.reasons.join("; "),
            });
        }

        if !decision.allows(permission) {
            let reason = if decision.reasons.is_empty() {
                format!("permission {} not allowed by policy", permission)
            } else {
                decision.reasons.join("; ")
            };
            return Err(LatticeError::PolicyViolation {
                reason,
            });
        }

        Ok(())
    }
}

fn requirement_satisfied(req: &Requirement, object: &Object, approvals: &[String]) -> bool {
    match req {
        Requirement::ApprovalFrom(required) => required
            .iter()
            .all(|name| approvals.iter().any(|a| a == name)),
        Requirement::MinTrust(min) => trust_level(&object.tags) >= *min,
        Requirement::RequireTag(tag) => has_required_tag(&object.tags, tag),
    }
}

fn approvals_from_tags(tags: &[Tag]) -> Vec<String> {
    tags.iter()
        .filter(|t| t.key == "sys:approved-by")
        .map(|t| t.value.clone())
        .collect()
}

fn has_required_tag(tags: &[Tag], required: &str) -> bool {
    tags.iter().any(|t| t.matches(required))
}
