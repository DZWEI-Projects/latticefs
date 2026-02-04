//! Cryptographic primitives for LatticeFS.
//!
//! This module provides:
//! - Ed25519 identity management (`identity`)
//! - OS keyring integration (`keyring`)
//! - AES-256-GCM encryption (`encryption`)
//! - UCAN capability tokens (`capability`)

pub mod capability;
pub mod encryption;
pub mod identity;
pub mod keyring;

pub use capability::{
    Attenuation, Capability, Facts, Permission, Revocation, RevocationChecker, RevocationList,
    UcanHeader, UcanPayload,
};
pub use encryption::{decrypt_chunk, encrypt_chunk, EncryptedData, ObjectKey};
pub use identity::{did_from_public_key, public_key_from_did, Identity, PublicKey};
pub use keyring::{KeyManager, KeyStorage};
