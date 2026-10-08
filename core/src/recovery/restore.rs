//! Restore engine: reassembles files from the content-addressable store.

use std::path::{Path, PathBuf};
use std::io::Write;
use tracing::{info, debug, warn};

use crate::repository::{ContentStore, Manifest, decompress};

/// Restore destination options
pub enum RestoreDestination {
    /// Restore to the original location
    Original,
    /// Restore to a specific path
    Custom(PathBuf),
}

/// Restore a version from the content store to a destination
pub fn restore_version(
    store: &ContentStore,
    manifest: &Manifest,
    destination: &Path,
) -> anyhow::Result<RestoreResult> {
    info!(
        "Restoring version {} ({} files, {} bytes) to {}",
        manifest.version_number,
        manifest.total_files,
        manifest.total_size,
        destination.display()
    );

    std::fs::create_dir_all(destination)?;

    let mut files_restored = 0u64;
    let mut bytes_restored = 0u64;
    let mut errors = Vec::new();

    for (relative_path, entry) in &manifest.files {
        let target_path = destination.join(relative_path);

        // Create parent directories
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        match restore_file(store, entry, &target_path) {
            Ok(size) => {
                files_restored += 1;
                bytes_restored += size;
                debug!("Restored: {}", relative_path);
            }
            Err(e) => {
                warn!("Failed to restore {}: {}", relative_path, e);
                errors.push(format!("{}: {}", relative_path, e));
            }
        }
    }

    info!(
        "Restore complete: {} files, {} bytes, {} errors",
        files_restored, bytes_restored, errors.len()
    );

    Ok(RestoreResult {
        files_restored,
        bytes_restored,
        errors,
    })
}

/// Restore a single file by reassembling its chunks
fn restore_file(
    store: &ContentStore,
    entry: &crate::repository::ManifestFileEntry,
    target_path: &Path,
) -> anyhow::Result<u64> {
    let mut file = std::fs::File::create(target_path)?;
    let mut total_bytes = 0u64;

    for chunk_hash in &entry.chunks {
        // Get the chunk from store (may be compressed)
        let compressed_data = store.get_chunk(chunk_hash)?;

        // Decompress
        let data = decompress(&compressed_data)?;

        file.write_all(&data)?;
        total_bytes += data.len() as u64;
    }

    file.sync_all()?;

    // Verify restored file integrity
    let actual_hash = crate::crypto::hash_file(target_path)?;
    if actual_hash != entry.hash {
        return Err(anyhow::anyhow!(
            "Integrity verification failed after restore: expected {}, got {}",
            &entry.hash[..12], &actual_hash[..12]
        ));
    }

    Ok(total_bytes)
}

/// Result of a restore operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct RestoreResult {
    pub files_restored: u64,
    pub bytes_restored: u64,
    pub errors: Vec<String>,
}

impl RestoreResult {
    pub fn is_successful(&self) -> bool {
        self.errors.is_empty()
    }
}
