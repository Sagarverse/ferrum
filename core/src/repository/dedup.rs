//! Deduplication engine.
//!
//! Uses BLAKE3 content hashes to identify duplicate chunks.
//! Chunks with the same hash are stored only once.

use std::sync::Arc;
use crate::db::Database;

/// Deduplication engine that tracks chunk references
pub struct DedupEngine {
    db: Arc<Database>,
}

impl DedupEngine {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Check if a chunk already exists. If it does, increment its ref count
    /// and return true (deduplicated). If not, return false (needs storage).
    pub fn check_and_ref(&self, hash: &str) -> anyhow::Result<bool> {
        if self.db.chunk_exists(hash)? {
            self.db.increment_chunk_ref(hash)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Record a new chunk in the database
    pub fn record_chunk(&self, hash: &str, size: u64, compressed_size: u64) -> anyhow::Result<()> {
        let record = crate::db::ChunkRecord {
            hash: hash.to_string(),
            size,
            compressed_size,
            ref_count: 1,
            created_at: chrono::Utc::now(),
        };
        self.db.add_chunk(&record)?;
        Ok(())
    }

    /// Dereference a chunk (decrement ref count)
    pub fn deref_chunk(&self, hash: &str) -> anyhow::Result<()> {
        self.db.decrement_chunk_ref(hash)?;
        Ok(())
    }

    /// Get chunks with zero references (candidates for garbage collection)
    pub fn get_orphans(&self) -> anyhow::Result<Vec<String>> {
        self.db.get_orphan_chunks()
    }

    /// Get deduplication statistics
    pub fn get_stats(&self) -> anyhow::Result<DedupStats> {
        let total_size = self.db.get_total_protected_size()?;
        let dedup_size = self.db.get_total_dedup_size()?;

        let ratio = if dedup_size > 0 {
            total_size as f64 / dedup_size as f64
        } else {
            1.0
        };

        let saved = if total_size > dedup_size {
            total_size - dedup_size
        } else {
            0
        };

        Ok(DedupStats {
            total_original_size: total_size,
            total_stored_size: dedup_size,
            dedup_ratio: ratio,
            space_saved: saved,
        })
    }
}

/// Deduplication statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct DedupStats {
    /// Total size of all original data
    pub total_original_size: u64,
    /// Total size after deduplication and compression
    pub total_stored_size: u64,
    /// Deduplication ratio (original / stored)
    pub dedup_ratio: f64,
    /// Bytes saved by deduplication
    pub space_saved: u64,
}
