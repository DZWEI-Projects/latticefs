//! OS keyring integration with encrypted file fallback.
//!
//! Provides secure storage for identity private keys using the operating system's
//! native keyring (Keychain on macOS, Secret Service on Linux, Credential Manager on Windows).
//! Falls back to an Argon2id-encrypted file if the keyring is unavailable.

use crate::crypto::identity::Identity;
use crate::error::{LatticeError, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::Argon2;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Service name for keyring entries.
const KEYRING_SERVICE: &str = "latticefs";

/// Key storage backend.
#[derive(Debug, Clone)]
pub enum KeyStorage {
    /// OS keyring (preferred).
    SystemKeyring,
    /// Encrypted file fallback.
    EncryptedFile { path: PathBuf },
}

impl KeyStorage {
    /// Detect the best available storage option.
    pub fn detect() -> Self {
        // Try to access the system keyring
        match keyring::Entry::new(KEYRING_SERVICE, "test-probe") {
            Ok(_) => KeyStorage::SystemKeyring,
            Err(_) => {
                // Fall back to encrypted file
                let path = dirs_path().join("keys.enc");
                KeyStorage::EncryptedFile { path }
            }
        }
    }

    /// Create encrypted file storage at a specific path.
    pub fn encrypted_file(path: PathBuf) -> Self {
        KeyStorage::EncryptedFile { path }
    }
}

/// Key manager for storing and retrieving identity keys.
pub struct KeyManager {
    storage: KeyStorage,
}

impl KeyManager {
    /// Create a new key manager with the given storage backend.
    pub fn new(storage: KeyStorage) -> Self {
        Self { storage }
    }

    /// Create a key manager with auto-detected storage.
    pub fn auto() -> Self {
        Self::new(KeyStorage::detect())
    }

    /// Store an identity's private key.
    ///
    /// For keyring storage, no password is needed (OS handles security).
    /// For encrypted file storage, a password is required.
    pub fn store(&self, identity: &Identity, password: Option<&str>) -> Result<()> {
        match &self.storage {
            KeyStorage::SystemKeyring => {
                let entry = keyring::Entry::new(KEYRING_SERVICE, &identity.name).map_err(|e| {
                    LatticeError::Keyring(format!("Failed to create keyring entry: {}", e))
                })?;

                let secret_hex = hex::encode(identity.secret_bytes());
                entry
                    .set_password(&secret_hex)
                    .map_err(|e| LatticeError::Keyring(format!("Failed to store key: {}", e)))?;

                Ok(())
            }
            KeyStorage::EncryptedFile { path } => {
                let password = password.ok_or_else(|| {
                    LatticeError::Keyring(
                        "Password required for encrypted file storage".to_string(),
                    )
                })?;

                self.store_to_file(path, identity, password)
            }
        }
    }

    /// Load an identity's private key.
    ///
    /// For keyring storage, no password is needed.
    /// For encrypted file storage, a password is required.
    pub fn load(&self, name: &str, password: Option<&str>) -> Result<Identity> {
        match &self.storage {
            KeyStorage::SystemKeyring => {
                let entry = keyring::Entry::new(KEYRING_SERVICE, name).map_err(|e| {
                    LatticeError::Keyring(format!("Failed to access keyring: {}", e))
                })?;

                let secret_hex =
                    entry
                        .get_password()
                        .map_err(|e| LatticeError::IdentityNotFound {
                            name: format!("{}: {}", name, e),
                        })?;

                let secret_bytes = hex::decode(&secret_hex)
                    .map_err(|e| LatticeError::Keyring(format!("Invalid key format: {}", e)))?;

                let secret_array: [u8; 32] = secret_bytes
                    .try_into()
                    .map_err(|_| LatticeError::Keyring("Invalid key length".to_string()))?;

                Ok(Identity::from_secret_bytes(name, &secret_array))
            }
            KeyStorage::EncryptedFile { path } => {
                let password = password.ok_or_else(|| {
                    LatticeError::Keyring(
                        "Password required for encrypted file storage".to_string(),
                    )
                })?;

                self.load_from_file(path, name, password)
            }
        }
    }

    /// Delete an identity's private key.
    pub fn delete(&self, name: &str) -> Result<()> {
        match &self.storage {
            KeyStorage::SystemKeyring => {
                let entry = keyring::Entry::new(KEYRING_SERVICE, name).map_err(|e| {
                    LatticeError::Keyring(format!("Failed to access keyring: {}", e))
                })?;

                entry
                    .delete_password()
                    .map_err(|e| LatticeError::Keyring(format!("Failed to delete key: {}", e)))?;

                Ok(())
            }
            KeyStorage::EncryptedFile { path } => {
                // Load existing keys, remove the one we want to delete, save
                // For simplicity, we don't support deletion in file-based storage yet
                let _ = path;
                Err(LatticeError::Keyring(
                    "Key deletion not supported for file-based storage".to_string(),
                ))
            }
        }
    }

    /// Check if an identity exists in storage.
    pub fn exists(&self, name: &str) -> bool {
        match &self.storage {
            KeyStorage::SystemKeyring => {
                if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, name) {
                    entry.get_password().is_ok()
                } else {
                    false
                }
            }
            KeyStorage::EncryptedFile { path } => {
                // Check if file exists and contains the key
                // For simplicity, just check if the file exists
                path.exists()
            }
        }
    }

    /// Store identity to encrypted file.
    fn store_to_file(&self, path: &Path, identity: &Identity, password: &str) -> Result<()> {
        // Derive key from password using Argon2id
        let salt = generate_salt();
        let key = derive_key(password, &salt)?;

        // Encrypt the secret key
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| LatticeError::Encryption(format!("Failed to create cipher: {}", e)))?;

        let nonce = generate_nonce();
        let nonce_obj = Nonce::from_slice(&nonce);

        let ciphertext = cipher
            .encrypt(nonce_obj, identity.secret_bytes().as_ref())
            .map_err(|e| LatticeError::Encryption(format!("Encryption failed: {}", e)))?;

        // Create the encrypted key file
        let encrypted_key = EncryptedKeyFile {
            version: 1,
            name: identity.name.clone(),
            salt,
            nonce,
            ciphertext,
        };

        // Write to file
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&encrypted_key)
            .map_err(|e| LatticeError::Serialization(format!("Failed to serialize: {}", e)))?;

        std::fs::write(path, json)?;

        Ok(())
    }

    /// Load identity from encrypted file.
    fn load_from_file(&self, path: &Path, name: &str, password: &str) -> Result<Identity> {
        // Read file
        let json = std::fs::read_to_string(path)?;

        let encrypted_key: EncryptedKeyFile = serde_json::from_str(&json)
            .map_err(|e| LatticeError::Serialization(format!("Failed to parse key file: {}", e)))?;

        if encrypted_key.name != name {
            return Err(LatticeError::IdentityNotFound {
                name: name.to_string(),
            });
        }

        // Derive key from password
        let key = derive_key(password, &encrypted_key.salt)?;

        // Decrypt
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| LatticeError::Decryption(format!("Failed to create cipher: {}", e)))?;

        let nonce = Nonce::from_slice(&encrypted_key.nonce);

        let plaintext = cipher
            .decrypt(nonce, encrypted_key.ciphertext.as_ref())
            .map_err(|e| LatticeError::Decryption(format!("Decryption failed: {}", e)))?;

        let secret_array: [u8; 32] = plaintext
            .try_into()
            .map_err(|_| LatticeError::Decryption("Invalid key length".to_string()))?;

        Ok(Identity::from_secret_bytes(name, &secret_array))
    }
}

/// Encrypted key file format.
#[derive(Debug, Serialize, Deserialize)]
struct EncryptedKeyFile {
    /// Format version.
    version: u32,
    /// Identity name.
    name: String,
    /// Argon2id salt.
    salt: [u8; 16],
    /// AES-GCM nonce.
    nonce: [u8; 12],
    /// Encrypted secret key.
    ciphertext: Vec<u8>,
}

/// Derive an AES-256 key from a password using Argon2id.
fn derive_key(password: &str, salt: &[u8; 16]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| LatticeError::Keyring(format!("Key derivation failed: {}", e)))?;
    Ok(key)
}

/// Generate a random salt for Argon2.
fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Generate a random nonce for AES-GCM.
fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

/// Get the default LatticeFS config directory.
fn dirs_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("latticefs")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_encrypted_file_roundtrip() {
        let temp_dir = tempdir().unwrap();
        let key_path = temp_dir.path().join("test.key");

        let storage = KeyStorage::EncryptedFile { path: key_path };
        let manager = KeyManager::new(storage);

        let identity = Identity::generate("test-user");
        let password = "super-secret-password";

        // Store
        manager.store(&identity, Some(password)).unwrap();

        // Load
        let loaded = manager.load("test-user", Some(password)).unwrap();

        assert_eq!(identity.secret_bytes(), loaded.secret_bytes());
        assert_eq!(identity.did(), loaded.did());
    }

    #[test]
    fn test_encrypted_file_wrong_password() {
        let temp_dir = tempdir().unwrap();
        let key_path = temp_dir.path().join("test.key");

        let storage = KeyStorage::EncryptedFile { path: key_path };
        let manager = KeyManager::new(storage);

        let identity = Identity::generate("test-user");

        // Store with one password
        manager.store(&identity, Some("correct")).unwrap();

        // Try to load with wrong password
        let result = manager.load("test-user", Some("wrong"));
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypted_file_wrong_name() {
        let temp_dir = tempdir().unwrap();
        let key_path = temp_dir.path().join("test.key");

        let storage = KeyStorage::EncryptedFile { path: key_path };
        let manager = KeyManager::new(storage);

        let identity = Identity::generate("test-user");

        // Store
        manager.store(&identity, Some("password")).unwrap();

        // Try to load with wrong name
        let result = manager.load("wrong-user", Some("password"));
        assert!(result.is_err());
    }

    #[test]
    fn test_key_derivation_deterministic() {
        let password = "test-password";
        let salt = [1u8; 16];

        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_key_derivation_different_salts() {
        let password = "test-password";
        let salt1 = [1u8; 16];
        let salt2 = [2u8; 16];

        let key1 = derive_key(password, &salt1).unwrap();
        let key2 = derive_key(password, &salt2).unwrap();

        assert_ne!(key1, key2);
    }
}
