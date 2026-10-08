//! Version manifests mapping file paths to ordered lists of chunk hashes.
//!
//! A manifest is the complete description of a version snapshot:
//! every file, its metadata, and the chunks that compose it.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Manifest not found: {0}")]
    NotFound(String),
}

/// A complete version manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Version ID
    pub version_id: String,
    /// Protected item ID
    pub protected_item_id: String,
    /// Version number
    pub version_number: u32,
    /// When this manifest was created
    pub created_at: String,
    /// Total number of files
    pub total_files: u64,
    /// Total original size of all files
    pub total_size: u64,
    /// Total number of unique chunks
    pub total_chunks: u64,
    /// File entries: relative path → file entry
    pub files: BTreeMap<String, ManifestFileEntry>,
}

/// A single file entry in a manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFileEntry {
    /// File size in bytes
    pub size: u64,
    /// BLAKE3 hash of the complete file
    pub hash: String,
    /// Last modified timestamp
    pub modified_at: String,
    /// Ordered list of chunk hashes composing this file
    pub chunks: Vec<String>,
    /// File permissions (platform-specific)
    pub permissions: Option<String>,
    /// Whether this file is a symlink
    pub is_symlink: bool,
    /// Symlink target (if is_symlink)
    pub symlink_target: Option<String>,
}

impl Manifest {
    /// Create a new empty manifest
    pub fn new(version_id: &str, protected_item_id: &str, version_number: u32) -> Self {
        Self {
            version_id: version_id.to_string(),
            protected_item_id: protected_item_id.to_string(),
            version_number,
            created_at: chrono::Utc::now().to_rfc3339(),
            total_files: 0,
            total_size: 0,
            total_chunks: 0,
            files: BTreeMap::new(),
        }
    }

    /// Add a file entry to the manifest
    pub fn add_file(&mut self, relative_path: &str, entry: ManifestFileEntry) {
        self.total_size += entry.size;
        self.total_chunks += entry.chunks.len() as u64;
        self.total_files += 1;
        self.files.insert(relative_path.to_string(), entry);
    }

    /// Serialize the manifest to JSON
    pub fn to_json(&self) -> Result<String, ManifestError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserialize a manifest from JSON
    pub fn from_json(json: &str) -> Result<Self, ManifestError> {
        Ok(serde_json::from_str(json)?)
    }

    /// Save the manifest to the content store and return its hash
    pub fn save_to_store(&self, store: &super::ContentStore) -> Result<String, ManifestError> {
        let json = self.to_json()?;
        let hash = crate::crypto::hash_bytes(json.as_bytes());
        store.put_chunk(&hash, json.as_bytes())
            .map_err(|e| ManifestError::Io(std::io::Error::new(
                std::io::ErrorKind::Other, e.to_string()
            )))?;
        Ok(hash)
    }

    /// Load a manifest from the content store by hash
    pub fn load_from_store(store: &super::ContentStore, hash: &str) -> Result<Self, ManifestError> {
        let data = store.get_chunk(hash)
            .map_err(|e| ManifestError::NotFound(e.to_string()))?;
        let json = String::from_utf8_lossy(&data);
        Self::from_json(&json)
    }

    /// Compare this manifest with another to find changed files
    pub fn diff(&self, other: &Manifest) -> ManifestDiff {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut deleted = Vec::new();
        let mut unchanged = Vec::new();

        // Find added and modified files
        for (path, entry) in &self.files {
            match other.files.get(path) {
                None => added.push(path.clone()),
                Some(old_entry) => {
                    if entry.hash != old_entry.hash {
                        modified.push(path.clone());
                    } else {
                        unchanged.push(path.clone());
                    }
                }
            }
        }

        // Find deleted files
        for path in other.files.keys() {
            if !self.files.contains_key(path) {
                deleted.push(path.clone());
            }
        }

        ManifestDiff {
            added,
            modified,
            deleted,
            unchanged,
        }
    }
}

/// Differences between two manifests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestDiff {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
    pub unchanged: Vec<String>,
}

impl ManifestDiff {
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.modified.is_empty() || !self.deleted.is_empty()
    }

    pub fn total_changes(&self) -> usize {
        self.added.len() + self.modified.len() + self.deleted.len()
    }
}

/// Save a manifest to a file path
pub fn save_manifest_to_file(manifest: &Manifest, path: &Path) -> Result<(), ManifestError> {
    let json = manifest.to_json()?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load a manifest from a file path
pub fn load_manifest_from_file(path: &Path) -> Result<Manifest, ManifestError> {
    let json = std::fs::read_to_string(path)?;
    Manifest::from_json(&json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_roundtrip() {
        let mut manifest = Manifest::new("v1", "item1", 1);
        manifest.add_file("test.txt", ManifestFileEntry {
            size: 100,
            hash: "abc123".to_string(),
            modified_at: "2026-01-01T00:00:00Z".to_string(),
            chunks: vec!["chunk1".to_string(), "chunk2".to_string()],
            permissions: None,
            is_symlink: false,
            symlink_target: None,
        });

        let json = manifest.to_json().unwrap();
        let loaded = Manifest::from_json(&json).unwrap();

        assert_eq!(loaded.version_id, "v1");
        assert_eq!(loaded.total_files, 1);
        assert_eq!(loaded.total_size, 100);
        assert!(loaded.files.contains_key("test.txt"));
    }

    #[test]
    fn test_manifest_diff() {
        let mut old = Manifest::new("v1", "item1", 1);
        old.add_file("a.txt", ManifestFileEntry {
            size: 100, hash: "hash_a".into(),
            modified_at: String::new(), chunks: vec![], permissions: None,
            is_symlink: false, symlink_target: None,
        });
        old.add_file("b.txt", ManifestFileEntry {
            size: 200, hash: "hash_b".into(),
            modified_at: String::new(), chunks: vec![], permissions: None,
            is_symlink: false, symlink_target: None,
        });
        old.add_file("c.txt", ManifestFileEntry {
            size: 300, hash: "hash_c".into(),
            modified_at: String::new(), chunks: vec![], permissions: None,
            is_symlink: false, symlink_target: None,
        });

        let mut new = Manifest::new("v2", "item1", 2);
        new.add_file("a.txt", ManifestFileEntry {
            size: 100, hash: "hash_a".into(), // unchanged
            modified_at: String::new(), chunks: vec![], permissions: None,
            is_symlink: false, symlink_target: None,
        });
        new.add_file("b.txt", ManifestFileEntry {
            size: 250, hash: "hash_b_modified".into(), // modified
            modified_at: String::new(), chunks: vec![], permissions: None,
            is_symlink: false, symlink_target: None,
        });
        new.add_file("d.txt", ManifestFileEntry {
            size: 400, hash: "hash_d".into(), // added
            modified_at: String::new(), chunks: vec![], permissions: None,
            is_symlink: false, symlink_target: None,
        });
        // c.txt is deleted

        let diff = new.diff(&old);
        assert_eq!(diff.added, vec!["d.txt"]);
        assert_eq!(diff.modified, vec!["b.txt"]);
        assert_eq!(diff.deleted, vec!["c.txt"]);
        assert_eq!(diff.unchanged, vec!["a.txt"]);
        assert!(diff.has_changes());
        assert_eq!(diff.total_changes(), 3);
    }
}
