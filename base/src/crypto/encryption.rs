//! AES-256-GCM encryption for objects.
//!
//! Provides per-object encryption with unique keys.
//! Per LFS-003 section 10, each version can be encrypted independently
//! using AES-256-GCM for authenticated encryption.

use crate::error::{LatticeError, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// AES-256 key size in bytes.
const KEY_SIZE: usize = 32;

/// AES-GCM nonce size in bytes.
const NONCE_SIZE: usize = 12;

/// Per-object encryption key (AES-256).
#[derive(Clone)]
pub struct ObjectKey([u8; KEY_SIZE]);

impl ObjectKey {
    /// Generate a new random encryption key.
    pub fn generate() -> Self {
        let mut key = [0u8; KEY_SIZE];
        rand::thread_rng().fill_bytes(&mut key);
        ObjectKey(key)
    }

    /// Create from existing bytes.
    pub fn from_bytes(bytes: [u8; KEY_SIZE]) -> Self {
        ObjectKey(bytes)
    }

    /// Get the raw key bytes.
    ///
    /// **Warning:** Handle with care - this is sensitive key material.
    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.0
    }

    /// Encrypt data with this key.
    ///
    /// Returns encrypted data containing the nonce and ciphertext.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        let cipher = Aes256Gcm::new_from_slice(&self.0)
            .map_err(|e| LatticeError::Encryption(format!("Failed to create cipher: {}", e)))?;

        let nonce = generate_nonce();
        let nonce_obj = Nonce::from_slice(&nonce);

        let ciphertext = cipher
            .encrypt(nonce_obj, plaintext)
            .map_err(|e| LatticeError::Encryption(format!("Encryption failed: {}", e)))?;

        Ok(EncryptedData { nonce, ciphertext })
    }

    /// Decrypt data with this key.
    ///
    /// Returns the original plaintext.
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.0)
            .map_err(|e| LatticeError::Decryption(format!("Failed to create cipher: {}", e)))?;

        let nonce = Nonce::from_slice(&encrypted.nonce);

        cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| LatticeError::Decryption(format!("Decryption failed: {}", e)))
    }
}

impl std::fmt::Debug for ObjectKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ObjectKey([REDACTED])")
    }
}

/// Encrypted data with nonce.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// AES-GCM nonce (12 bytes).
    pub nonce: [u8; NONCE_SIZE],
    /// Ciphertext with authentication tag.
    pub ciphertext: Vec<u8>,
}

impl EncryptedData {
    /// Get the total size in bytes (nonce + ciphertext).
    pub fn size(&self) -> usize {
        self.nonce.len() + self.ciphertext.len()
    }

    /// Serialize to bytes (nonce || ciphertext).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.size());
        bytes.extend_from_slice(&self.nonce);
        bytes.extend_from_slice(&self.ciphertext);
        bytes
    }

    /// Deserialize from bytes (nonce || ciphertext).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < NONCE_SIZE {
            return Err(LatticeError::Decryption(format!(
                "Encrypted data too short: {} bytes",
                bytes.len()
            )));
        }

        let nonce: [u8; NONCE_SIZE] = bytes[..NONCE_SIZE]
            .try_into()
            .map_err(|_| LatticeError::Decryption("Invalid nonce".to_string()))?;

        let ciphertext = bytes[NONCE_SIZE..].to_vec();

        Ok(EncryptedData { nonce, ciphertext })
    }
}

/// Generate a random nonce for AES-GCM.
fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

/// Encrypt a chunk of data with a given key.
///
/// Convenience function for encrypting object chunks.
pub fn encrypt_chunk(key: &ObjectKey, chunk: &[u8]) -> Result<EncryptedData> {
    key.encrypt(chunk)
}

/// Decrypt a chunk of data with a given key.
///
/// Convenience function for decrypting object chunks.
pub fn decrypt_chunk(key: &ObjectKey, encrypted: &EncryptedData) -> Result<Vec<u8>> {
    key.decrypt(encrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = ObjectKey::generate();
        let plaintext = b"Hello, NeuralFS! This is secret data.";

        let encrypted = key.encrypt(plaintext).unwrap();
        let decrypted = key.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let key = ObjectKey::generate();
        let plaintext = b"";

        let encrypted = key.encrypt(plaintext).unwrap();
        let decrypted = key.decrypt(&encrypted).unwrap();

        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_encrypt_decrypt_large() {
        let key = ObjectKey::generate();
        let plaintext: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();

        let encrypted = key.encrypt(&plaintext).unwrap();
        let decrypted = key.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = ObjectKey::generate();
        let key2 = ObjectKey::generate();
        let plaintext = b"Secret data";

        let encrypted = key1.encrypt(plaintext).unwrap();
        let result = key2.decrypt(&encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = ObjectKey::generate();
        let plaintext = b"Secret data";

        let mut encrypted = key.encrypt(plaintext).unwrap();
        // Tamper with ciphertext
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF;
        }

        let result = key.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypted_data_bytes_roundtrip() {
        let key = ObjectKey::generate();
        let plaintext = b"Test data";

        let encrypted = key.encrypt(plaintext).unwrap();
        let bytes = encrypted.to_bytes();
        let restored = EncryptedData::from_bytes(&bytes).unwrap();

        assert_eq!(encrypted.nonce, restored.nonce);
        assert_eq!(encrypted.ciphertext, restored.ciphertext);

        // And we can still decrypt
        let decrypted = key.decrypt(&restored).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypted_data_size() {
        let key = ObjectKey::generate();
        let plaintext = b"Hello";

        let encrypted = key.encrypt(plaintext).unwrap();

        // Size should be nonce (12) + ciphertext (plaintext + 16 byte auth tag)
        assert_eq!(encrypted.size(), 12 + plaintext.len() + 16);
    }

    #[test]
    fn test_different_encryptions_different_nonces() {
        let key = ObjectKey::generate();
        let plaintext = b"Same data";

        let encrypted1 = key.encrypt(plaintext).unwrap();
        let encrypted2 = key.encrypt(plaintext).unwrap();

        // Nonces should be different (random)
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        // Ciphertexts should also be different
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    }

    #[test]
    fn test_object_key_from_bytes() {
        let bytes = [42u8; 32];
        let key = ObjectKey::from_bytes(bytes);

        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn test_encrypt_chunk_convenience() {
        let key = ObjectKey::generate();
        let chunk = b"Chunk data";

        let encrypted = encrypt_chunk(&key, chunk).unwrap();
        let decrypted = decrypt_chunk(&key, &encrypted).unwrap();

        assert_eq!(chunk.as_slice(), decrypted.as_slice());
    }
}
