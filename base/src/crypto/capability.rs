//! UCAN (User Controlled Authorization Network) capability tokens.
//!
//! Implements capability-based security using UCAN tokens per LFS-003.
//! UCANs are JWT-like tokens that provide:
//! - Cryptographic signatures (Ed25519)
//! - Delegatable permissions
//! - Time-bounded validity
//! - Revocable access

use crate::crypto::identity::{public_key_from_did, Identity, PublicKey};
use crate::error::{LatticeError, Result};
use crate::model::ObjectID;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{Signature, Verifier};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// UCAN version supported by this implementation.
const UCAN_VERSION: &str = "0.10.0";

/// Maximum proof chain depth to prevent DoS.
const MAX_PROOF_CHAIN_DEPTH: usize = 10;

/// Permission level for capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    /// Read object content and metadata.
    Read,
    /// Add comments (future feature).
    Comment,
    /// Create new versions, modify metadata.
    Write,
    /// Create new capabilities for this object.
    Share,
    /// Full control including policy changes.
    Admin,
}

impl Permission {
    /// Check if this permission includes another.
    pub fn includes(&self, other: &Permission) -> bool {
        self >= other
    }

    /// Get the permission level as a numeric value.
    pub fn level(&self) -> u8 {
        match self {
            Permission::Read => 1,
            Permission::Comment => 2,
            Permission::Write => 3,
            Permission::Share => 4,
            Permission::Admin => 5,
        }
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::Read => write!(f, "read"),
            Permission::Comment => write!(f, "comment"),
            Permission::Write => write!(f, "write"),
            Permission::Share => write!(f, "share"),
            Permission::Admin => write!(f, "admin"),
        }
    }
}

impl std::str::FromStr for Permission {
    type Err = LatticeError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "read" => Ok(Permission::Read),
            "comment" => Ok(Permission::Comment),
            "write" => Ok(Permission::Write),
            "share" => Ok(Permission::Share),
            "admin" => Ok(Permission::Admin),
            _ => Err(LatticeError::InvalidPredicate(format!(
                "Unknown permission: {}",
                s
            ))),
        }
    }
}

/// UCAN header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UcanHeader {
    /// Algorithm (always "EdDSA").
    pub alg: String,
    /// Type (always "JWT").
    pub typ: String,
    /// UCAN version.
    pub ucv: String,
}

impl Default for UcanHeader {
    fn default() -> Self {
        Self {
            alg: "EdDSA".to_string(),
            typ: "JWT".to_string(),
            ucv: UCAN_VERSION.to_string(),
        }
    }
}

/// Attenuation (capability grant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attenuation {
    /// Resource URI.
    pub with: String,
    /// Permission granted.
    pub can: Permission,
}

impl Attenuation {
    /// Create an attenuation for an object.
    pub fn for_object(object_id: &ObjectID, permission: Permission) -> Self {
        Self {
            with: format!("latticefs:object:{}", object_id),
            can: permission,
        }
    }

    /// Create an attenuation for an arbitrary resource URI.
    pub fn for_resource(resource: String, permission: Permission) -> Self {
        Self {
            with: resource,
            can: permission,
        }
    }

    /// Parse the object ID from the resource URI.
    pub fn object_id(&self) -> Option<ObjectID> {
        if self.with.starts_with("latticefs:object:") {
            let id_str = &self.with[17..];
            uuid::Uuid::parse_str(id_str)
                .ok()
                .map(ObjectID::from_uuid)
        } else {
            None
        }
    }
}

/// Facts (contextual constraints).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Facts {
    /// LatticeFS version.
    #[serde(rename = "lfs/version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Device binding.
    #[serde(rename = "lfs/device", skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    /// Custom facts.
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

/// Revocation entry for a UCAN capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revocation {
    /// CID of the revoked UCAN.
    pub ucan_cid: String,
    /// Revocation time (Unix seconds).
    pub revoked_at: u64,
    /// DID of revoker.
    pub revoked_by: String,
    /// Optional reason.
    pub reason: Option<String>,
    /// Signature over the revocation payload.
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
struct RevocationPayload<'a> {
    ucan_cid: &'a str,
    revoked_at: u64,
    revoked_by: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: &'a Option<String>,
}

impl Revocation {
    /// Create a signed revocation entry.
    pub fn new(ucan_cid: String, revoker: &Identity, reason: Option<String>) -> Result<Self> {
        let revoked_at = current_timestamp();
        let revoked_by = revoker.did();

        let payload = RevocationPayload {
            ucan_cid: &ucan_cid,
            revoked_at,
            revoked_by: &revoked_by,
            reason: &reason,
        };

        let bytes = serde_json::to_vec(&payload)
            .map_err(|e| LatticeError::Serialization(format!("Revocation serialize: {}", e)))?;

        let signature = revoker.sign(&bytes).to_bytes().to_vec();

        Ok(Self {
            ucan_cid,
            revoked_at,
            revoked_by,
            reason,
            signature,
        })
    }

    /// Verify the revocation signature.
    pub fn verify(&self) -> Result<()> {
        let payload = RevocationPayload {
            ucan_cid: &self.ucan_cid,
            revoked_at: self.revoked_at,
            revoked_by: &self.revoked_by,
            reason: &self.reason,
        };

        let bytes = serde_json::to_vec(&payload)
            .map_err(|e| LatticeError::Serialization(format!("Revocation serialize: {}", e)))?;

        let issuer_key = public_key_from_did(&self.revoked_by)?;
        let signature_bytes: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| LatticeError::InvalidRevocationSignature)?;
        let signature = Signature::from_bytes(&signature_bytes);

        issuer_key
            .verify(bytes.as_ref(), &signature)
            .map_err(|_| LatticeError::InvalidRevocationSignature)
    }
}

/// In-memory revocation list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RevocationList {
    pub revocations: Vec<Revocation>,
}

impl RevocationList {
    /// Add a revocation (verifies signature).
    pub fn add(&mut self, revocation: Revocation) -> Result<()> {
        revocation.verify()?;
        if !self
            .revocations
            .iter()
            .any(|r| r.ucan_cid == revocation.ucan_cid)
        {
            self.revocations.push(revocation);
        }
        Ok(())
    }

    /// Check if a CID is revoked (verifies signature).
    pub fn is_revoked(&self, cid: &str) -> Result<bool> {
        if let Some(revocation) = self.revocations.iter().find(|r| r.ucan_cid == cid) {
            revocation.verify()?;
            return Ok(true);
        }
        Ok(false)
    }
}

/// Interface for revocation checking.
pub trait RevocationChecker {
    fn is_revoked(&self, cid: &str) -> Result<bool>;
}

impl RevocationChecker for RevocationList {
    fn is_revoked(&self, cid: &str) -> Result<bool> {
        self.is_revoked(cid)
    }
}

impl RevocationChecker for crate::storage::MetadataStore {
    fn is_revoked(&self, cid: &str) -> Result<bool> {
        crate::storage::MetadataStore::is_revoked(self, cid)
    }
}

/// UCAN payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UcanPayload {
    /// Issuer DID.
    pub iss: String,
    /// Audience DID.
    pub aud: String,
    /// Subject (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    /// Expiration time (Unix timestamp).
    pub exp: u64,
    /// Not-before time (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<u64>,
    /// Nonce (optional, for replay protection).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nnc: Option<String>,
    /// Attenuations (capabilities granted).
    pub att: Vec<Attenuation>,
    /// Proof chain (parent UCANs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prf: Vec<String>,
    /// Facts (contextual constraints).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fct: Option<Facts>,
}

/// A UCAN capability token.
#[derive(Debug, Clone)]
pub struct Capability {
    /// Raw JWT token.
    pub token: String,
    /// Parsed header.
    pub header: UcanHeader,
    /// Parsed payload.
    pub payload: UcanPayload,
    /// Signature bytes.
    pub signature: Vec<u8>,
}

impl Capability {
    /// Create a new capability.
    pub fn create(
        issuer: &Identity,
        audience: &PublicKey,
        object_id: &ObjectID,
        permission: Permission,
        expires_in: Duration,
    ) -> Result<Self> {
        let now = current_timestamp();
        let exp = now + expires_in.as_secs();

        let header = UcanHeader::default();
        let payload = UcanPayload {
            iss: issuer.did(),
            aud: audience.did(),
            sub: Some(format!("latticefs:object:{}", object_id)),
            exp,
            nbf: Some(now),
            nnc: Some(uuid::Uuid::now_v7().to_string()),
            att: vec![Attenuation::for_object(object_id, permission)],
            prf: vec![],
            fct: Some(Facts {
                version: Some("0.1".to_string()),
                ..Default::default()
            }),
        };

        Self::sign(header, payload, issuer)
    }

    /// Create a new capability for an arbitrary resource URI (e.g., view snapshot).
    pub fn create_for_resource(
        issuer: &Identity,
        audience: &PublicKey,
        resource: String,
        permission: Permission,
        expires_in: Duration,
    ) -> Result<Self> {
        let now = current_timestamp();
        let exp = now + expires_in.as_secs();

        let header = UcanHeader::default();
        let payload = UcanPayload {
            iss: issuer.did(),
            aud: audience.did(),
            sub: Some(resource.clone()),
            exp,
            nbf: Some(now),
            nnc: Some(uuid::Uuid::now_v7().to_string()),
            att: vec![Attenuation::for_resource(resource, permission)],
            prf: vec![],
            fct: Some(Facts {
                version: Some("0.1".to_string()),
                ..Default::default()
            }),
        };

        Self::sign(header, payload, issuer)
    }

    /// Compute the CID of this capability (BLAKE3 of token).
    pub fn cid(&self) -> String {
        let hash = blake3::hash(self.token.as_bytes());
        hex::encode(hash.as_bytes())
    }

    /// Sign a UCAN with the given header and payload.
    fn sign(header: UcanHeader, payload: UcanPayload, issuer: &Identity) -> Result<Self> {
        let header_json = serde_json::to_string(&header)
            .map_err(|e| LatticeError::Serialization(format!("Header serialization: {}", e)))?;
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| LatticeError::Serialization(format!("Payload serialization: {}", e)))?;

        let header_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());

        let signing_input = format!("{}.{}", header_b64, payload_b64);
        let signature = issuer.sign(signing_input.as_bytes());
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

        let token = format!("{}.{}", signing_input, signature_b64);

        Ok(Self {
            token,
            header,
            payload,
            signature: signature.to_bytes().to_vec(),
        })
    }

    /// Parse a UCAN token string.
    pub fn parse(token: &str) -> Result<Self> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(LatticeError::Serialization(
                "Invalid UCAN format: expected 3 parts".to_string(),
            ));
        }

        let header_bytes = URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|e| LatticeError::Serialization(format!("Header decode: {}", e)))?;
        let payload_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|e| LatticeError::Serialization(format!("Payload decode: {}", e)))?;
        let signature = URL_SAFE_NO_PAD
            .decode(parts[2])
            .map_err(|e| LatticeError::Serialization(format!("Signature decode: {}", e)))?;

        let header: UcanHeader = serde_json::from_slice(&header_bytes)
            .map_err(|e| LatticeError::Serialization(format!("Header parse: {}", e)))?;
        let payload: UcanPayload = serde_json::from_slice(&payload_bytes)
            .map_err(|e| LatticeError::Serialization(format!("Payload parse: {}", e)))?;

        Ok(Self {
            token: token.to_string(),
            header,
            payload,
            signature,
        })
    }

    /// Validate the capability.
    ///
    /// Checks signature, expiration, and proof chain.
    pub fn validate<R: RevocationChecker>(&self, revocations: &R) -> Result<()> {
        // Check signature
        self.verify_signature()?;

        // Check expiration
        let now = current_timestamp();
        if now >= self.payload.exp {
            return Err(LatticeError::CapabilityExpired);
        }

        // Check not-before
        if let Some(nbf) = self.payload.nbf {
            if now < nbf {
                return Err(LatticeError::CapabilityNotYetValid);
            }
        }

        // Check revocation
        if revocations.is_revoked(&self.cid())? {
            return Err(LatticeError::CapabilityRevoked);
        }

        // Verify proof chain
        self.verify_proof_chain(0, revocations)?;

        Ok(())
    }

    /// Verify the signature.
    fn verify_signature(&self) -> Result<()> {
        let issuer_key = public_key_from_did(&self.payload.iss)?;

        let parts: Vec<&str> = self.token.split('.').collect();
        let signing_input = format!("{}.{}", parts[0], parts[1]);

        let signature_bytes: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| LatticeError::InvalidSignature)?;

        let signature = Signature::from_bytes(&signature_bytes);

        issuer_key
            .verify(signing_input.as_bytes(), &signature)
            .map_err(|_| LatticeError::InvalidSignature)
    }

    /// Verify the proof chain recursively.
    fn verify_proof_chain<R: RevocationChecker>(&self, depth: usize, revocations: &R) -> Result<()> {
        if depth > MAX_PROOF_CHAIN_DEPTH {
            return Err(LatticeError::InvalidProofChain(format!(
                "Proof chain too deep: max {} levels",
                MAX_PROOF_CHAIN_DEPTH
            )));
        }

        for proof_token in &self.payload.prf {
            let proof = Capability::parse(proof_token)?;

            // Validate the proof
            proof.validate(revocations)?;

            // Verify delegation chain: this UCAN's issuer must be the proof's audience
            if self.payload.iss != proof.payload.aud {
                return Err(LatticeError::InvalidProofChain(
                    "Issuer is not the audience of proof".to_string(),
                ));
            }

            // Verify attenuation: capabilities must be subset of proof
            for att in &self.payload.att {
                let proof_has_permission = proof.payload.att.iter().any(|p| {
                    p.with == att.with && p.can.includes(&att.can)
                });

                if !proof_has_permission {
                    return Err(LatticeError::InvalidAttenuation {
                        from: format!("{:?}", proof.payload.att),
                        to: format!("{:?}", att),
                    });
                }
            }

            // Recursively verify the proof's chain
            proof.verify_proof_chain(depth + 1, revocations)?;
        }

        Ok(())
    }

    /// Delegate this capability to another party.
    ///
    /// The new capability will have reduced or equal permissions.
    pub fn delegate(
        &self,
        delegator: &Identity,
        new_audience: &PublicKey,
        attenuated_permission: Permission,
        expires_in: Duration,
        revocations: &impl RevocationChecker,
    ) -> Result<Self> {
        // Ensure this capability is valid before delegating.
        self.validate(revocations)?;

        // Verify the delegator is the audience of this UCAN
        if delegator.did() != self.payload.aud {
            return Err(LatticeError::Unauthorized {
                permission: "delegate".to_string(),
                object: "capability".to_string(),
            });
        }

        // Verify attenuation (new permission must be <= current)
        for att in &self.payload.att {
            if !att.can.includes(&attenuated_permission) {
                return Err(LatticeError::InvalidAttenuation {
                    from: att.can.to_string(),
                    to: attenuated_permission.to_string(),
                });
            }
        }

        let now = current_timestamp();
        // Expiration must not exceed parent's expiration
        let exp = std::cmp::min(now + expires_in.as_secs(), self.payload.exp);

        let header = UcanHeader::default();
        let payload = UcanPayload {
            iss: delegator.did(),
            aud: new_audience.did(),
            sub: self.payload.sub.clone(),
            exp,
            nbf: Some(now),
            nnc: Some(uuid::Uuid::now_v7().to_string()),
            att: self
                .payload
                .att
                .iter()
                .map(|a| Attenuation {
                    with: a.with.clone(),
                    can: attenuated_permission,
                })
                .collect(),
            prf: vec![self.token.clone()],
            fct: Some(Facts {
                version: Some("0.1".to_string()),
                ..Default::default()
            }),
        };

        Self::sign(header, payload, delegator)
    }

    /// Check if this capability grants a specific permission for an object.
    pub fn has_permission(&self, object_id: &ObjectID, permission: Permission) -> bool {
        let object_uri = format!("latticefs:object:{}", object_id);
        self.payload
            .att
            .iter()
            .any(|a| a.with == object_uri && a.can.includes(&permission))
    }

    /// Check if this capability grants a specific permission for a resource URI.
    pub fn has_permission_for_resource(&self, resource: &str, permission: Permission) -> bool {
        self.payload
            .att
            .iter()
            .any(|a| a.with == resource && a.can.includes(&permission))
    }

    /// Get the object ID this capability refers to (if any).
    pub fn object_id(&self) -> Option<ObjectID> {
        if let Some(sub) = &self.payload.sub {
            if let Some(id) = object_id_from_resource(sub) {
                return Some(id);
            }
        }

        self.payload
            .att
            .iter()
            .find_map(|a| object_id_from_resource(&a.with))
    }

    /// Create a signed revocation for this capability.
    pub fn revoke<R: RevocationChecker>(
        &self,
        revoker: &Identity,
        reason: Option<String>,
        admin_capability: Option<&Capability>,
        revocations: &R,
    ) -> Result<Revocation> {
        if revoker.did() != self.payload.iss {
            let admin = admin_capability.ok_or_else(|| LatticeError::Unauthorized {
                permission: "revoke".to_string(),
                object: "capability".to_string(),
            })?;

            admin.validate(revocations)?;

            if admin.payload.aud != revoker.did() {
                return Err(LatticeError::Unauthorized {
                    permission: "revoke".to_string(),
                    object: "capability".to_string(),
                });
            }

            let object_id = self.object_id().ok_or_else(|| LatticeError::InvalidPredicate(
                "Capability missing object subject".to_string(),
            ))?;

            if !admin.has_permission(&object_id, Permission::Admin) {
                return Err(LatticeError::Unauthorized {
                    permission: "admin".to_string(),
                    object: object_id.to_string(),
                });
            }
        }

        Revocation::new(self.cid(), revoker, reason)
    }

    /// Get the issuer's DID.
    pub fn issuer(&self) -> &str {
        &self.payload.iss
    }

    /// Get the audience's DID.
    pub fn audience(&self) -> &str {
        &self.payload.aud
    }

    /// Get the expiration timestamp.
    pub fn expires_at(&self) -> u64 {
        self.payload.exp
    }

    /// Check if the capability is expired.
    pub fn is_expired(&self) -> bool {
        current_timestamp() >= self.payload.exp
    }
}

/// Get the current Unix timestamp.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn object_id_from_resource(resource: &str) -> Option<ObjectID> {
    if resource.starts_with("latticefs:object:") {
        let id_str = &resource[17..];
        uuid::Uuid::parse_str(id_str)
            .ok()
            .map(ObjectID::from_uuid)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_capability() {
        let issuer = Identity::generate("alice");
        let audience = PublicKey::new(Identity::generate("bob").verifying_key);
        let object_id = ObjectID::new();

        let cap = Capability::create(
            &issuer,
            &audience,
            &object_id,
            Permission::Read,
            Duration::from_secs(3600),
        )
        .unwrap();

        assert!(cap.has_permission(&object_id, Permission::Read));
        assert!(!cap.has_permission(&object_id, Permission::Write));
    }

    #[test]
    fn test_parse_roundtrip() {
        let issuer = Identity::generate("alice");
        let audience = PublicKey::new(Identity::generate("bob").verifying_key);
        let object_id = ObjectID::new();

        let cap = Capability::create(
            &issuer,
            &audience,
            &object_id,
            Permission::Write,
            Duration::from_secs(3600),
        )
        .unwrap();

        let parsed = Capability::parse(&cap.token).unwrap();
        assert_eq!(parsed.payload.iss, cap.payload.iss);
        assert_eq!(parsed.payload.aud, cap.payload.aud);
    }

    #[test]
    fn test_validate_signature() {
        let issuer = Identity::generate("alice");
        let audience = PublicKey::new(Identity::generate("bob").verifying_key);
        let object_id = ObjectID::new();

        let cap = Capability::create(
            &issuer,
            &audience,
            &object_id,
            Permission::Read,
            Duration::from_secs(3600),
        )
        .unwrap();

        let revocations = RevocationList::default();
        assert!(cap.validate(&revocations).is_ok());
    }

    #[test]
    fn test_expired_capability() {
        let issuer = Identity::generate("alice");
        let audience = PublicKey::new(Identity::generate("bob").verifying_key);
        let object_id = ObjectID::new();

        // Create with 0 duration (immediately expires)
        let cap = Capability::create(
            &issuer,
            &audience,
            &object_id,
            Permission::Read,
            Duration::from_secs(0),
        )
        .unwrap();

        assert!(cap.is_expired());
        let revocations = RevocationList::default();
        assert!(matches!(
            cap.validate(&revocations),
            Err(LatticeError::CapabilityExpired)
        ));
    }

    #[test]
    fn test_delegate_capability() {
        let alice = Identity::generate("alice");
        let bob = Identity::generate("bob");
        let charlie = PublicKey::new(Identity::generate("charlie").verifying_key);
        let object_id = ObjectID::new();

        // Alice creates capability for Bob with Write permission
        let bob_cap = Capability::create(
            &alice,
            &PublicKey::new(bob.verifying_key),
            &object_id,
            Permission::Write,
            Duration::from_secs(3600),
        )
        .unwrap();

        // Bob delegates to Charlie with Read permission (attenuation)
        let charlie_cap = bob_cap
            .delegate(
                &bob,
                &charlie,
                Permission::Read,
                Duration::from_secs(1800),
                &RevocationList::default(),
            )
            .unwrap();

        // Validate the delegated capability
        assert!(charlie_cap.validate(&RevocationList::default()).is_ok());
        assert!(charlie_cap.has_permission(&object_id, Permission::Read));
        assert!(!charlie_cap.has_permission(&object_id, Permission::Write));
    }

    #[test]
    fn test_delegate_escalation_fails() {
        let alice = Identity::generate("alice");
        let bob = Identity::generate("bob");
        let charlie = PublicKey::new(Identity::generate("charlie").verifying_key);
        let object_id = ObjectID::new();

        // Alice creates capability for Bob with Read permission
        let bob_cap = Capability::create(
            &alice,
            &PublicKey::new(bob.verifying_key),
            &object_id,
            Permission::Read,
            Duration::from_secs(3600),
        )
        .unwrap();

        // Bob tries to delegate to Charlie with Write permission (escalation - should fail)
        let result = bob_cap.delegate(
            &bob,
            &charlie,
            Permission::Write,
            Duration::from_secs(1800),
            &RevocationList::default(),
        );

        assert!(matches!(result, Err(LatticeError::InvalidAttenuation { .. })));
    }

    #[test]
    fn test_permission_hierarchy() {
        assert!(Permission::Admin.includes(&Permission::Share));
        assert!(Permission::Admin.includes(&Permission::Write));
        assert!(Permission::Admin.includes(&Permission::Read));
        assert!(Permission::Write.includes(&Permission::Read));
        assert!(!Permission::Read.includes(&Permission::Write));
    }

    #[test]
    fn test_permission_from_str() {
        assert_eq!("read".parse::<Permission>().unwrap(), Permission::Read);
        assert_eq!("WRITE".parse::<Permission>().unwrap(), Permission::Write);
        assert_eq!("Admin".parse::<Permission>().unwrap(), Permission::Admin);
        assert!("invalid".parse::<Permission>().is_err());
    }

    #[test]
    fn test_attenuation_object_id() {
        let object_id = ObjectID::new();
        let att = Attenuation::for_object(&object_id, Permission::Read);

        assert_eq!(att.object_id(), Some(object_id));
    }

    #[test]
    fn test_unauthorized_delegator() {
        let alice = Identity::generate("alice");
        let bob = Identity::generate("bob");
        let mallory = Identity::generate("mallory");
        let charlie = PublicKey::new(Identity::generate("charlie").verifying_key);
        let object_id = ObjectID::new();

        // Alice creates capability for Bob
        let bob_cap = Capability::create(
            &alice,
            &PublicKey::new(bob.verifying_key),
            &object_id,
            Permission::Read,
            Duration::from_secs(3600),
        )
        .unwrap();

        // Mallory (not the audience) tries to delegate
        let result = bob_cap.delegate(
            &mallory,
            &charlie,
            Permission::Read,
            Duration::from_secs(1800),
            &RevocationList::default(),
        );

        assert!(matches!(result, Err(LatticeError::Unauthorized { .. })));
    }

    #[test]
    fn test_revocation_signature() {
        let alice = Identity::generate("alice");
        let object_id = ObjectID::new();

        let cap = Capability::create(
            &alice,
            &PublicKey::new(Identity::generate("bob").verifying_key),
            &object_id,
            Permission::Read,
            Duration::from_secs(3600),
        )
        .unwrap();

        let revocation = cap
            .revoke(&alice, Some("test".to_string()), None, &RevocationList::default())
            .unwrap();

        assert!(revocation.verify().is_ok());
    }

    #[test]
    fn test_revocation_blocks_validation() {
        let alice = Identity::generate("alice");
        let bob = PublicKey::new(Identity::generate("bob").verifying_key);
        let object_id = ObjectID::new();

        let cap = Capability::create(
            &alice,
            &bob,
            &object_id,
            Permission::Read,
            Duration::from_secs(3600),
        )
        .unwrap();

        let mut revocations = RevocationList::default();
        let rev = cap
            .revoke(&alice, None, None, &RevocationList::default())
            .unwrap();
        revocations.add(rev).unwrap();

        assert!(matches!(
            cap.validate(&revocations),
            Err(LatticeError::CapabilityRevoked)
        ));
    }

    #[test]
    fn test_revocation_cascades() {
        let alice = Identity::generate("alice");
        let bob = Identity::generate("bob");
        let charlie = PublicKey::new(Identity::generate("charlie").verifying_key);
        let object_id = ObjectID::new();

        let parent = Capability::create(
            &alice,
            &PublicKey::new(bob.verifying_key),
            &object_id,
            Permission::Write,
            Duration::from_secs(3600),
        )
        .unwrap();

        let child = parent
            .delegate(
                &bob,
                &charlie,
                Permission::Read,
                Duration::from_secs(1800),
                &RevocationList::default(),
            )
            .unwrap();

        let mut revocations = RevocationList::default();
        let rev = parent
            .revoke(&alice, None, None, &RevocationList::default())
            .unwrap();
        revocations.add(rev).unwrap();

        assert!(matches!(
            child.validate(&revocations),
            Err(LatticeError::CapabilityRevoked)
        ));
    }
}
