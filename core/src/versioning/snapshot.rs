//! Snapshot engine: creates point-in-time snapshots of protected data.
//!
//! Each snapshot is a complete manifest of all files and their chunks.
//! Uses copy-on-write semantics — never overwrites a committed version.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use chrono::Utc;
use tracing::{info, debug, warn};
use uuid::Uuid;

use crate::config::CompressionAlgorithm;
use crate::db::{Database, VersionRecord};
use crate::events::EventBus;
use crate::repository::{
    ContentStore, Manifest, ManifestFileEntry,
    chunk_file, compress, DedupEngine,
};

/// Snapshot engine creates and manages version snapshots
pub struct SnapshotEngine {
    db: Arc<Database>,
    store: ContentStore,
    dedup: DedupEngine,
    compression: CompressionAlgorithm,
    event_bus: EventBus,
}

impl SnapshotEngine {
    /// Create a new snapshot engine
    pub fn new(
        db: Arc<Database>,
        store_path: &Path,
        compression: CompressionAlgorithm,
        event_bus: EventBus,
    ) -> anyhow::Result<Self> {
        let store = ContentStore::open(store_path)?;
        let dedup = DedupEngine::new(Arc::clone(&db));

        Ok(Self {
            db,
            store,
            dedup,
            compression,
            event_bus,
        })
    }

    /// Create a snapshot of the given source path
    pub fn create_snapshot(
        &self,
        protected_item_id: &Uuid,
        source_path: &Path,
    ) -> anyhow::Result<VersionRecord> {
        let version_id = Uuid::new_v4();
        let now = Utc::now();

        // Determine version number
        let existing_versions = self.db.get_versions(protected_item_id)?;
        let version_number = existing_versions.last()
            .map(|v| v.version_number + 1)
            .unwrap_or(1);

        info!(
            "Creating snapshot v{} for {} (source: {})",
            version_number,
            protected_item_id,
            source_path.display()
        );

        // Build manifest by walking the source
        let mut manifest = Manifest::new(
            &version_id.to_string(),
            &protected_item_id.to_string(),
            version_number,
        );

        if source_path.is_file() {
            // Single file protection
            let relative = source_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let entry = self.process_file(protected_item_id, &relative, source_path, source_path.parent().unwrap_or(Path::new("")))?;
            manifest.add_file(&relative, entry);
        } else if source_path.is_dir() {
            // Directory protection — walk all files
            for entry in walkdir::WalkDir::new(source_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let file_path = entry.path();

                // Get relative path
                let relative = file_path.strip_prefix(source_path)
                    .unwrap_or(file_path)
                    .to_string_lossy()
                    .to_string();

                // Skip if it matches exclusion patterns (basic check)
                if should_exclude(&relative) {
                    debug!("Excluding: {}", relative);
                    continue;
                }

                match self.process_file(protected_item_id, &relative, file_path, source_path) {
                    Ok(file_entry) => {
                        manifest.add_file(&relative, file_entry);
                    }
                    Err(e) => {
                        warn!("Failed to process {}: {}", relative, e);
                        // Continue with other files
                    }
                }
            }
        }

        // Save manifest to content store
        let manifest_hash = manifest.save_to_store(&self.store)?;

        // Record version in database
        let version = VersionRecord {
            id: version_id,
            protected_item_id: *protected_item_id,
            version_number,
            manifest_hash: manifest_hash.clone(),
            total_size: manifest.total_size,
            total_chunks: manifest.total_chunks,
            status: "healthy".to_string(),
            created_at: now,
            metadata: Some(serde_json::json!({
                "total_files": manifest.total_files,
                "source_path": source_path.display().to_string(),
            }).to_string()),
        };

        self.db.add_version(&version)?;

        info!(
            "Snapshot v{} created: {} files, {} bytes, {} chunks, manifest: {}",
            version_number,
            manifest.total_files,
            manifest.total_size,
            manifest.total_chunks,
            &manifest_hash[..12]
        );

        Ok(version)
    }

    /// Process a single file: chunk, deduplicate, compress, store
    fn process_file(
        &self,
        item_id: &Uuid,
        relative_path: &str,
        file_path: &Path,
        _base_path: &Path,
    ) -> anyhow::Result<ManifestFileEntry> {
        let metadata = std::fs::metadata(file_path)?;
        let file_hash = crate::crypto::hash_file(file_path)?;

        // Chunk the file
        let chunks = chunk_file(file_path)?;
        let mut chunk_hashes = Vec::with_capacity(chunks.len());
        let mut new_chunks = 0u64;
        let mut deduplicated = false;

        for chunk in &chunks {
            let exists_in_store = self.store.has_chunk(&chunk.hash);
            let is_dedup = self.dedup.check_and_ref(&chunk.hash)?;

            if is_dedup {
                deduplicated = true;
            }

            if !exists_in_store {
                // Compress the chunk
                let compressed = compress(&chunk.data, &self.compression)?;

                // Store the compressed chunk on disk
                self.store.put_chunk(&chunk.hash, &compressed)?;

                // Record in database if brand new
                if !is_dedup {
                    self.dedup.record_chunk(
                        &chunk.hash,
                        chunk.size,
                        compressed.len() as u64,
                    )?;
                    new_chunks += 1;
                }
            }

            chunk_hashes.push(chunk.hash.clone());
        }

        let modified_at = metadata.modified()
            .map(|t| chrono::DateTime::<Utc>::from(t).to_rfc3339())
            .unwrap_or_default();

        // Notify the UI that this file moved through the pipeline.
        self.event_bus.send(crate::events::EngineEvent::FileProcessed {
            item_id: *item_id,
            relative_path: relative_path.to_string(),
            size: metadata.len(),
            chunks: chunk_hashes.len() as u64,
            new_chunks,
            deduplicated,
        });

        Ok(ManifestFileEntry {
            size: metadata.len(),
            hash: file_hash,
            modified_at,
            chunks: chunk_hashes,
            permissions: None,
            is_symlink: metadata.file_type().is_symlink(),
            symlink_target: None,
        })
    }

    /// Get the content store reference
    pub fn store(&self) -> &ContentStore {
        &self.store
    }

    /// Get the dedup engine reference
    pub fn dedup(&self) -> &DedupEngine {
        &self.dedup
    }
}

/// Basic exclusion check (simplified glob matching)
fn should_exclude(relative_path: &str) -> bool {
    let exclusions = [
        ".DS_Store", "Thumbs.db", "desktop.ini",
        ".git/", "node_modules/", "target/",
        "__pycache__/", ".idea/", ".vscode/",
    ];

    for pattern in &exclusions {
        if relative_path.contains(pattern) {
            return true;
        }
    }

    // Check file extensions
    let ext_exclusions = [".tmp", ".swp", ".swo", ".pyc"];
    for ext in &ext_exclusions {
        if relative_path.ends_with(ext) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let source_dir = dir.path().join("source");
        let store_dir = dir.path().join("store");

        // Create test files
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(source_dir.join("file1.txt"), "Hello, World!").unwrap();
        std::fs::write(source_dir.join("file2.txt"), "Another file").unwrap();

        let sub_dir = source_dir.join("subdir");
        std::fs::create_dir_all(&sub_dir).unwrap();
        std::fs::write(sub_dir.join("nested.txt"), "Nested file content").unwrap();

        let db = Arc::new(Database::open_memory().unwrap());
        let engine = SnapshotEngine::new(
            db.clone(),
            &store_dir,
            CompressionAlgorithm::Lz4,
            crate::events::EventBus::new(8),
        ).unwrap();

        let item_id = Uuid::new_v4();
        let version = engine.create_snapshot(&item_id, &source_dir).unwrap();

        assert_eq!(version.version_number, 1);
        assert_eq!(version.status, "healthy");
        assert!(version.total_size > 0);

        // Create a second snapshot
        std::fs::write(source_dir.join("file3.txt"), "New file").unwrap();
        let version2 = engine.create_snapshot(&item_id, &source_dir).unwrap();
        assert_eq!(version2.version_number, 2);
    }
}
