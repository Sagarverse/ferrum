//! Garbage collection for orphan chunks.
//!
//! Identifies chunks with zero references and removes them
//! from both the database and the content store.

use std::sync::Arc;
use tracing::{info, warn, debug};
use crate::db::Database;
use super::ContentStore;

/// Garbage collection engine
pub struct GarbageCollector {
    db: Arc<Database>,
}

impl GarbageCollector {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Run garbage collection on the given content store.
    /// Returns the number of chunks removed and bytes freed.
    pub fn collect(&self, store: &ContentStore) -> anyhow::Result<GcResult> {
        info!("Starting garbage collection");

        let orphans = self.db.get_orphan_chunks()?;
        let orphan_count = orphans.len();

        if orphans.is_empty() {
            info!("No orphan chunks found");
            return Ok(GcResult {
                chunks_removed: 0,
                bytes_freed: 0,
            });
        }

        info!("Found {} orphan chunks", orphan_count);

        let mut chunks_removed = 0u64;
        let mut bytes_freed = 0u64;

        for hash in &orphans {
            // Get chunk size before deleting
            match store.get_chunk(hash) {
                Ok(data) => {
                    bytes_freed += data.len() as u64;
                    // Delete from store
                    if let Err(e) = store.delete_chunk(hash) {
                        warn!("Failed to delete chunk {} from store: {}", &hash[..12], e);
                        continue;
                    }
                }
                Err(e) => {
                    debug!("Chunk {} not in store (may already be deleted): {}", &hash[..12], e);
                }
            }

            // Delete from database
            if let Err(e) = self.db.delete_chunk(hash) {
                warn!("Failed to delete chunk {} from database: {}", &hash[..12], e);
                continue;
            }

            chunks_removed += 1;
        }

        info!(
            "Garbage collection complete: removed {} chunks, freed {} bytes",
            chunks_removed, bytes_freed
        );

        Ok(GcResult {
            chunks_removed,
            bytes_freed,
        })
    }
}

/// Result of a garbage collection run
#[derive(Debug, Clone, serde::Serialize)]
pub struct GcResult {
    pub chunks_removed: u64,
    pub bytes_freed: u64,
}
