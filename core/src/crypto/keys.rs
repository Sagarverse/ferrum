//! Encryption key management, derivation, and secure storage.
//!
//! Uses Argon2id for key derivation from passwords.
//! Supports AES-256-GCM and XChaCha20-Poly1305 for authenticated encryption.

use aes_gcm::{Aes256Gcm, KeyInit, Nonce as AesNonce};
use aes_gcm::aead::Aead;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use rand::RngCore;
use thiserror::Error;

/// Nonce size for AES-256-GCM (96 bits = 12 bytes)
const AES_GCM_NONCE_SIZE: usize = 12;

/// Key size for AES-256 (256 bits = 32 bytes)
const AES_KEY_SIZE: usize = 32;

/// Salt size for Argon2
const SALT_SIZE: usize = 16;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Invalid key size")]
    InvalidKeySize,

    #[error("Invalid data format")]
    InvalidFormat,
}

/// An encryption key with metadata
#[derive(Clone)]
pub struct EncryptionKey {
    /// The raw key bytes (32 bytes for AES-256)
    key_bytes: Vec<u8>,
    /// Salt used for derivation (if derived from password)
    pub salt: Option<Vec<u8>>,
}

impl EncryptionKey {
    /// Generate a random encryption key
    pub fn generate() -> Self {
        let mut key_bytes = vec![0u8; AES_KEY_SIZE];
        rand::thread_rng().fill_bytes(&mut key_bytes);
        Self {
            key_bytes,
            salt: None,
        }
    }

    /// Derive a key from a password using Argon2id
    pub fn from_password(password: &str) -> Result<Self, CryptoError> {
        let mut salt_bytes = vec![0u8; SALT_SIZE];
        rand::thread_rng().fill_bytes(&mut salt_bytes);

        let salt = SaltString::encode_b64(&salt_bytes)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

        let hash_output = hash.hash.ok_or_else(|| {
            CryptoError::KeyDerivationFailed("No hash output".into())
        })?;

        let hash_bytes = hash_output.as_bytes();
        let key_bytes = if hash_bytes.len() >= AES_KEY_SIZE {
            hash_bytes[..AES_KEY_SIZE].to_vec()
        } else {
            // Pad with BLAKE3 if needed
            let extended = blake3::hash(hash_bytes);
            extended.as_bytes()[..AES_KEY_SIZE].to_vec()
        };

        Ok(Self {
            key_bytes,
            salt: Some(salt_bytes),
        })
    }

    /// Derive a key from a password with a known salt (for decryption)
    pub fn from_password_with_salt(password: &str, salt_bytes: &[u8]) -> Result<Self, CryptoError> {
        let salt = SaltString::encode_b64(salt_bytes)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

        let hash_output = hash.hash.ok_or_else(|| {
            CryptoError::KeyDerivationFailed("No hash output".into())
        })?;

        let hash_bytes = hash_output.as_bytes();
        let key_bytes = if hash_bytes.len() >= AES_KEY_SIZE {
            hash_bytes[..AES_KEY_SIZE].to_vec()
        } else {
            let extended = blake3::hash(hash_bytes);
            extended.as_bytes()[..AES_KEY_SIZE].to_vec()
        };

        Ok(Self {
            key_bytes,
            salt: Some(salt_bytes.to_vec()),
        })
    }

    /// Create a key from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != AES_KEY_SIZE {
            return Err(CryptoError::InvalidKeySize);
        }
        Ok(Self {
            key_bytes: bytes.to_vec(),
            salt: None,
        })
    }

    /// Get the raw key bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.key_bytes
    }
}

impl Drop for EncryptionKey {
    fn drop(&mut self) {
        // Zero out key material on drop
        for b in &mut self.key_bytes {
            *b = 0;
        }
    }
}

/// Encrypt data using AES-256-GCM
///
/// Output format: [nonce (12 bytes)] [ciphertext + tag]
pub fn encrypt_aes256gcm(key: &EncryptionKey, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // Generate random nonce
    let mut nonce_bytes = [0u8; AES_GCM_NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = AesNonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // Prepend nonce to ciphertext
    let mut output = Vec::with_capacity(AES_GCM_NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypt data using AES-256-GCM
///
/// Input format: [nonce (12 bytes)] [ciphertext + tag]
pub fn decrypt_aes256gcm(key: &EncryptionKey, encrypted: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if encrypted.len() < AES_GCM_NONCE_SIZE {
        return Err(CryptoError::InvalidFormat);
    }

    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    let (nonce_bytes, ciphertext) = encrypted.split_at(AES_GCM_NONCE_SIZE);
    let nonce = AesNonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    Ok(plaintext)
}

/// Hash a password for storage (for app lockdown password)
pub fn hash_password(password: &str) -> Result<String, CryptoError> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
    Ok(hash.to_string())
}

/// Verify a password against a stored hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, CryptoError> {
    use argon2::password_hash::PasswordHash;
    use argon2::PasswordVerifier;

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_aes256gcm() {
        let key = EncryptionKey::generate();
        let plaintext = b"Hello, Metallic Mini! This is sensitive data.";

        let encrypted = encrypt_aes256gcm(&key, plaintext).unwrap();
        assert_ne!(encrypted, plaintext);
        assert!(encrypted.len() > plaintext.len()); // nonce + tag overhead

        let decrypted = decrypt_aes256gcm(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = EncryptionKey::generate();
        let key2 = EncryptionKey::generate();
        let plaintext = b"Secret data";

        let encrypted = encrypt_aes256gcm(&key1, plaintext).unwrap();
        let result = decrypt_aes256gcm(&key2, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_from_password() {
        let key = EncryptionKey::from_password("MyStr0ngP@ssw0rd!").unwrap();
        assert_eq!(key.as_bytes().len(), 32);
        assert!(key.salt.is_some());

        // Same password + same salt should produce same key
        let key2 = EncryptionKey::from_password_with_salt(
            "MyStr0ngP@ssw0rd!",
            key.salt.as_ref().unwrap(),
        ).unwrap();
        assert_eq!(key.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_password_hashing() {
        let hash = hash_password("test_password").unwrap();
        assert!(verify_password("test_password", &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_empty_data() {
        let key = EncryptionKey::generate();
        let encrypted = encrypt_aes256gcm(&key, b"").unwrap();
        let decrypted = decrypt_aes256gcm(&key, &encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_large_data() {
        let key = EncryptionKey::generate();
        let plaintext = vec![0xABu8; 1024 * 1024]; // 1 MB
        let encrypted = encrypt_aes256gcm(&key, &plaintext).unwrap();
        let decrypted = decrypt_aes256gcm(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
