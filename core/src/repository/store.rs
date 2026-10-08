//! Content-addressable chunk store.
//!
//! Stores chunks in a flat directory structure organized by hash prefix:
//!   repository/chunks/ab/cd/abcdef1234...
//!
//! This provides efficient lookup and avoids too many files in a single directory.

use std::path::{Path, PathBuf};
use std::io::Write;
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Chunk not found: {0}")]
    ChunkNotFound(String),

    #[error("Integrity error: expected {expected}, got {actual}")]
    IntegrityError { expected: String, actual: String },

    #[error("Repository not initialized at {0}")]
    NotInitialized(String),
}

/// A content-addressable chunk store
pub struct ContentStore {
    /// Root directory of the chunk store
    root: PathBuf,
}

impl ContentStore {
    /// Create or open a content store at the given path
    pub fn open(root: &Path) -> Result<Self, StoreError> {
        let chunks_dir = root.join("chunks");
        std::fs::create_dir_all(&chunks_dir)?;

        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    /// Get the file path for a chunk by its hash
    fn chunk_path(&self, hash: &str) -> PathBuf {
        let (prefix1, rest) = hash.split_at(2.min(hash.len()));
        let (prefix2, _) = rest.split_at(2.min(rest.len()));
        self.root.join("chunks").join(prefix1).join(prefix2).join(hash)
    }

    /// Check if a chunk exists in the store
    pub fn has_chunk(&self, hash: &str) -> bool {
        self.chunk_path(hash).exists()
    }

    /// Store a chunk (with optional compression already applied)
    /// Returns true if the chunk was newly stored, false if it already existed (dedup)
    pub fn put_chunk(&self, hash: &str, data: &[u8]) -> Result<bool, StoreError> {
        let path = self.chunk_path(hash);

        // Deduplication: if the chunk already exists, skip
        if path.exists() {
            debug!("Chunk {} already exists, deduplicating", &hash[..12]);
            return Ok(false);
        }

        // Create parent directories
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write to a temporary file first, then rename (atomic)
        let tmp_path = path.with_extension("tmp");
        let mut file = std::fs::File::create(&tmp_path)?;
        file.write_all(data)?;
        file.sync_all()?;
        drop(file);

        // Atomic rename
        std::fs::rename(&tmp_path, &path)?;

        debug!("Stored chunk {} ({} bytes)", &hash[..12], data.len());
        Ok(true)
    }

    /// Retrieve a chunk by its hash
    pub fn get_chunk(&self, hash: &str) -> Result<Vec<u8>, StoreError> {
        let path = self.chunk_path(hash);

        if !path.exists() {
            return Err(StoreError::ChunkNotFound(hash.to_string()));
        }

        let data = std::fs::read(&path)?;
        Ok(data)
    }

    /// Verify a chunk's integrity
    pub fn verify_chunk(&self, hash: &str) -> Result<bool, StoreError> {
        let data = self.get_chunk(hash)?;
        let actual_hash = crate::crypto::hash_bytes(&data);

        if actual_hash != hash {
            warn!("Chunk integrity failure: expected {}, got {}", hash, actual_hash);
            return Err(StoreError::IntegrityError {
                expected: hash.to_string(),
                actual: actual_hash,
            });
        }

        Ok(true)
    }

    /// Delete a chunk from the store
    pub fn delete_chunk(&self, hash: &str) -> Result<(), StoreError> {
        let path = self.chunk_path(hash);
        if path.exists() {
            std::fs::remove_file(&path)?;
            debug!("Deleted chunk {}", &hash[..12]);

            // Clean up empty parent directories
            if let Some(parent) = path.parent() {
                let _ = std::fs::remove_dir(parent); // Only succeeds if empty
                if let Some(grandparent) = parent.parent() {
                    let _ = std::fs::remove_dir(grandparent);
                }
            }
        }
        Ok(())
    }

    /// Get the total size of all chunks in the store
    pub fn total_size(&self) -> Result<u64, StoreError> {
        let chunks_dir = self.root.join("chunks");
        if !chunks_dir.exists() {
            return Ok(0);
        }

        let mut total = 0u64;
        for entry in walkdir::WalkDir::new(&chunks_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && !e.path().extension().is_some_and(|ext| ext == "tmp"))
        {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }

        Ok(total)
    }

    /// Count all chunks in the store
    pub fn chunk_count(&self) -> Result<u64, StoreError> {
        let chunks_dir = self.root.join("chunks");
        if !chunks_dir.exists() {
            return Ok(0);
        }

        let count = walkdir::WalkDir::new(&chunks_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && !e.path().extension().is_some_and(|ext| ext == "tmp"))
            .count();

        Ok(count as u64)
    }

    /// List all chunk hashes in the store
    pub fn list_chunks(&self) -> Result<Vec<String>, StoreError> {
        let chunks_dir = self.root.join("chunks");
        if !chunks_dir.exists() {
            return Ok(Vec::new());
        }

        let hashes: Vec<String> = walkdir::WalkDir::new(&chunks_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && !e.path().extension().is_some_and(|ext| ext == "tmp"))
            .filter_map(|e| e.file_name().to_str().map(String::from))
            .collect();

        Ok(hashes)
    }

    /// Verify integrity of all chunks in the store
    /// Returns (healthy_count, corrupted_hashes)
    pub fn verify_all(&self) -> Result<(u64, Vec<String>), StoreError> {
        let hashes = self.list_chunks()?;
        let mut healthy = 0u64;
        let mut corrupted = Vec::new();

        for hash in &hashes {
            match self.verify_chunk(hash) {
                Ok(true) => healthy += 1,
                _ => corrupted.push(hash.clone()),
            }
        }

        Ok((healthy, corrupted))
    }

    /// Get the root path of the store
    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        let dir = tempfile::tempdir().unwrap();
        let store = ContentStore::open(dir.path()).unwrap();

        let data = b"Hello, content-addressable storage!";
        let hash = crate::crypto::hash_bytes(data);

        // Store chunk
        let is_new = store.put_chunk(&hash, data).unwrap();
        assert!(is_new);

        // Retrieve chunk
        let retrieved = store.get_chunk(&hash).unwrap();
        assert_eq!(retrieved, data);

        // Verify integrity
        assert!(store.verify_chunk(&hash).unwrap());
    }

    #[test]
    fn test_deduplication() {
        let dir = tempfile::tempdir().unwrap();
        let store = ContentStore::open(dir.path()).unwrap();

        let data = b"Duplicate data";
        let hash = crate::crypto::hash_bytes(data);

        // First store should be new
        assert!(store.put_chunk(&hash, data).unwrap());

        // Second store should be deduplicated
        assert!(!store.put_chunk(&hash, data).unwrap());
    }

    #[test]
    fn test_delete_chunk() {
        let dir = tempfile::tempdir().unwrap();
        let store = ContentStore::open(dir.path()).unwrap();

        let data = b"Data to delete";
        let hash = crate::crypto::hash_bytes(data);

        store.put_chunk(&hash, data).unwrap();
        assert!(store.has_chunk(&hash));

        store.delete_chunk(&hash).unwrap();
        assert!(!store.has_chunk(&hash));
    }

    #[test]
    fn test_chunk_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let store = ContentStore::open(dir.path()).unwrap();

        let result = store.get_chunk("nonexistent_hash");
        assert!(result.is_err());
    }

    #[test]
    fn test_total_size_and_count() {
        let dir = tempfile::tempdir().unwrap();
        let store = ContentStore::open(dir.path()).unwrap();

        let data1 = b"Chunk one data";
        let data2 = b"Chunk two data here";
        let hash1 = crate::crypto::hash_bytes(data1);
        let hash2 = crate::crypto::hash_bytes(data2);

        store.put_chunk(&hash1, data1).unwrap();
        store.put_chunk(&hash2, data2).unwrap();

        assert_eq!(store.chunk_count().unwrap(), 2);
        assert_eq!(store.total_size().unwrap(), (data1.len() + data2.len()) as u64);
    }
}
