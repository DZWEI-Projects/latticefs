//! Ed25519 identity management.
//!
//! Provides Ed25519 keypair generation, signing, verification, and DID:key encoding.
//! Per LFS-003, identities are represented as DID:key URIs using Ed25519 public keys.

use crate::error::{LatticeError, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

/// Multicodec prefix for Ed25519 public keys.
/// See: https://github.com/multiformats/multicodec
const ED25519_PUB_MULTICODEC: [u8; 2] = [0xed, 0x01];

/// User identity backed by an Ed25519 keypair.
#[derive(Clone)]
pub struct Identity {
    /// Human-readable name for this identity.
    pub name: String,
    /// Ed25519 signing key (private).
    signing_key: SigningKey,
    /// Ed25519 verifying key (public).
    pub verifying_key: VerifyingKey,
}

impl Identity {
    /// Generate a new random identity.
    pub fn generate(name: &str) -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        Self {
            name: name.to_string(),
            signing_key,
            verifying_key,
        }
    }

    /// Create an identity from an existing signing key.
    pub fn from_signing_key(name: &str, signing_key: SigningKey) -> Self {
        let verifying_key = signing_key.verifying_key();
        Self {
            name: name.to_string(),
            signing_key,
            verifying_key,
        }
    }

    /// Create an identity from secret key bytes (32 bytes).
    pub fn from_secret_bytes(name: &str, bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        Self::from_signing_key(name, signing_key)
    }

    /// Get the secret key bytes (for secure storage).
    pub fn secret_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Get the public key bytes.
    pub fn public_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Get the DID:key representation of this identity.
    ///
    /// Format: did:key:z<base58btc(multicodec-prefix + public-key)>
    /// Per LFS-003 Appendix A.
    pub fn did(&self) -> String {
        did_from_public_key(&self.verifying_key)
    }

    /// Sign a message with this identity's private key.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Verify a signature against this identity's public key.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<()> {
        self.verifying_key
            .verify(message, signature)
            .map_err(|_| LatticeError::InvalidSignature)
    }
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("name", &self.name)
            .field("did", &self.did())
            .finish_non_exhaustive()
    }
}

/// Public key wrapper for representing other parties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey(#[serde(with = "public_key_serde")] pub VerifyingKey);

impl PublicKey {
    /// Create from a verifying key.
    pub fn new(key: VerifyingKey) -> Self {
        Self(key)
    }

    /// Create from raw bytes (32 bytes).
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let key = VerifyingKey::from_bytes(bytes)
            .map_err(|e| LatticeError::Serialization(format!("Invalid public key: {}", e)))?;
        Ok(Self(key))
    }

    /// Get the raw bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Get the DID:key representation.
    pub fn did(&self) -> String {
        did_from_public_key(&self.0)
    }

    /// Parse a DID:key string.
    pub fn from_did(did: &str) -> Result<Self> {
        public_key_from_did(did).map(Self)
    }

    /// Verify a signature against this public key.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<()> {
        self.0
            .verify(message, signature)
            .map_err(|_| LatticeError::InvalidSignature)
    }
}

/// Generate a DID:key from a public key.
///
/// Format: did:key:z<base58btc(multicodec-prefix + public-key)>
pub fn did_from_public_key(public_key: &VerifyingKey) -> String {
    let mut bytes = Vec::with_capacity(34);
    bytes.extend_from_slice(&ED25519_PUB_MULTICODEC);
    bytes.extend_from_slice(&public_key.to_bytes());
    format!("did:key:z{}", bs58::encode(&bytes).into_string())
}

/// Parse a DID:key string to extract the public key.
pub fn public_key_from_did(did: &str) -> Result<VerifyingKey> {
    // Check prefix
    if !did.starts_with("did:key:z") {
        return Err(LatticeError::Serialization(format!(
            "Invalid DID format: expected 'did:key:z' prefix, got '{}'",
            did
        )));
    }

    // Decode base58btc (after 'z' prefix)
    let encoded = &did[9..]; // Skip "did:key:z"
    let decoded = bs58::decode(encoded)
        .into_vec()
        .map_err(|e| LatticeError::Serialization(format!("Invalid base58btc in DID: {}", e)))?;

    // Check multicodec prefix
    if decoded.len() != 34 {
        return Err(LatticeError::Serialization(format!(
            "Invalid DID length: expected 34 bytes, got {}",
            decoded.len()
        )));
    }

    if decoded[0..2] != ED25519_PUB_MULTICODEC {
        return Err(LatticeError::Serialization(format!(
            "Invalid multicodec prefix: expected {:?}, got {:?}",
            ED25519_PUB_MULTICODEC,
            &decoded[0..2]
        )));
    }

    // Extract public key
    let key_bytes: [u8; 32] = decoded[2..34]
        .try_into()
        .map_err(|_| LatticeError::Serialization("Invalid key bytes length".to_string()))?;

    VerifyingKey::from_bytes(&key_bytes)
        .map_err(|e| LatticeError::Serialization(format!("Invalid Ed25519 public key: {}", e)))
}

/// Serde module for serializing VerifyingKey as bytes.
mod public_key_serde {
    use ed25519_dalek::VerifyingKey;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(key: &VerifyingKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        key.to_bytes().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<VerifyingKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: [u8; 32] = Deserialize::deserialize(deserializer)?;
        VerifyingKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation() {
        let identity = Identity::generate("test-user");
        assert_eq!(identity.name, "test-user");
        assert!(identity.did().starts_with("did:key:z"));
    }

    #[test]
    fn test_identity_from_secret_bytes() {
        let identity1 = Identity::generate("test");
        let secret = identity1.secret_bytes();

        let identity2 = Identity::from_secret_bytes("test", &secret);
        assert_eq!(identity1.public_bytes(), identity2.public_bytes());
        assert_eq!(identity1.did(), identity2.did());
    }

    #[test]
    fn test_sign_verify_roundtrip() {
        let identity = Identity::generate("signer");
        let message = b"Hello, LatticeFS!";

        let signature = identity.sign(message);
        assert!(identity.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_verify_wrong_message() {
        let identity = Identity::generate("signer");
        let message = b"Hello, LatticeFS!";
        let wrong_message = b"Wrong message";

        let signature = identity.sign(message);
        assert!(identity.verify(wrong_message, &signature).is_err());
    }

    #[test]
    fn test_verify_wrong_key() {
        let identity1 = Identity::generate("signer1");
        let identity2 = Identity::generate("signer2");
        let message = b"Hello, LatticeFS!";

        let signature = identity1.sign(message);
        assert!(identity2.verify(message, &signature).is_err());
    }

    #[test]
    fn test_did_roundtrip() {
        let identity = Identity::generate("test");
        let did = identity.did();

        let public_key = public_key_from_did(&did).unwrap();
        assert_eq!(identity.public_bytes(), public_key.to_bytes());
    }

    #[test]
    fn test_did_format() {
        let identity = Identity::generate("test");
        let did = identity.did();

        // DID should start with "did:key:z"
        assert!(did.starts_with("did:key:z"));
        // Base58btc encoding should be roughly 46 chars for 34 bytes
        assert!(did.len() > 55);
    }

    #[test]
    fn test_public_key_serde() {
        let identity = Identity::generate("test");
        let public_key = PublicKey::new(identity.verifying_key);

        let json = serde_json::to_string(&public_key).unwrap();
        let deserialized: PublicKey = serde_json::from_str(&json).unwrap();

        assert_eq!(public_key.to_bytes(), deserialized.to_bytes());
    }

    #[test]
    fn test_public_key_from_did() {
        let identity = Identity::generate("test");
        let did = identity.did();

        let public_key = PublicKey::from_did(&did).unwrap();
        assert_eq!(identity.public_bytes(), public_key.to_bytes());
    }

    #[test]
    fn test_public_key_verify() {
        let identity = Identity::generate("signer");
        let public_key = PublicKey::new(identity.verifying_key);
        let message = b"Test message";

        let signature = identity.sign(message);
        assert!(public_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_invalid_did_format() {
        // Missing prefix
        assert!(public_key_from_did("key:z123").is_err());
        // Wrong prefix
        assert!(public_key_from_did("did:web:example.com").is_err());
        // Invalid base58
        assert!(public_key_from_did("did:key:z!!!invalid").is_err());
    }
}
