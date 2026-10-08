//! BLAKE3-based integrity verification and content hashing.
//!
//! All content identification in Metallic Mini uses BLAKE3 for:
//! - Chunk identification (content-addressable storage)
//! - File integrity verification
//! - Manifest integrity
//! - Repository health checking

use std::io::Read;
use std::path::Path;
use thiserror::Error;

/// Buffer size for streaming hash operations (64 KB)
const HASH_BUFFER_SIZE: usize = 64 * 1024;

#[derive(Error, Debug)]
pub enum IntegrityError {
    #[error("I/O error during hashing: {0}")]
    Io(#[from] std::io::Error),

    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
}

/// Hash raw bytes using BLAKE3, returning hex-encoded hash
pub fn hash_bytes(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hash.to_hex().to_string()
}

/// Hash a file by streaming (never loads entire file into RAM)
pub fn hash_file(path: &Path) -> Result<String, IntegrityError> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::with_capacity(HASH_BUFFER_SIZE, file);
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; HASH_BUFFER_SIZE];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let hash = hasher.finalize();
    Ok(hash.to_hex().to_string())
}

/// Hash a reader by streaming
pub fn hash_reader<R: Read>(reader: &mut R) -> Result<String, IntegrityError> {
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; HASH_BUFFER_SIZE];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let hash = hasher.finalize();
    Ok(hash.to_hex().to_string())
}

/// Verify a file's integrity against an expected hash
pub fn verify_file(path: &Path, expected_hash: &str) -> Result<bool, IntegrityError> {
    let actual = hash_file(path)?;
    if actual != expected_hash {
        return Err(IntegrityError::HashMismatch {
            expected: expected_hash.to_string(),
            actual,
        });
    }
    Ok(true)
}

/// Verify data integrity against an expected hash
pub fn verify_bytes(data: &[u8], expected_hash: &str) -> Result<bool, IntegrityError> {
    let actual = hash_bytes(data);
    if actual != expected_hash {
        return Err(IntegrityError::HashMismatch {
            expected: expected_hash.to_string(),
            actual,
        });
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_hash_bytes() {
        let data = b"Hello, Metallic Mini!";
        let hash = hash_bytes(data);
        assert_eq!(hash.len(), 64); // BLAKE3 produces 256-bit (32-byte) hashes = 64 hex chars

        // Same data should produce same hash
        let hash2 = hash_bytes(data);
        assert_eq!(hash, hash2);

        // Different data should produce different hash
        let hash3 = hash_bytes(b"Different data");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_hash_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"Test file content for hashing").unwrap();
        drop(file);

        let hash = hash_file(&file_path).unwrap();
        assert_eq!(hash.len(), 64);

        // Verify should pass
        assert!(verify_file(&file_path, &hash).unwrap());
    }

    #[test]
    fn test_verify_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, b"Some content").unwrap();

        let result = verify_file(&file_path, "0000000000000000000000000000000000000000000000000000000000000000");
        assert!(result.is_err());
    }
}
