//! Replica repair: rebuilds corrupted replicas from healthy sources.

use std::path::Path;
use tracing::{info, warn};

use crate::repository::ContentStore;

/// Repair a corrupted replica using a healthy source
pub fn repair_replica(
    healthy_store: &ContentStore,
    corrupted_store: &ContentStore,
) -> anyhow::Result<RepairResult> {
    info!("Starting replica repair");

    let corrupted_chunks = {
        let (_, corrupted) = corrupted_store.verify_all()?;
        corrupted
    };

    if corrupted_chunks.is_empty() {
        info!("No corrupted chunks found");
        return Ok(RepairResult {
            chunks_repaired: 0,
            chunks_failed: 0,
        });
    }

    info!("Found {} corrupted chunks, repairing from healthy replica", corrupted_chunks.len());

    let mut repaired = 0u64;
    let mut failed = 0u64;

    for hash in &corrupted_chunks {
        match healthy_store.get_chunk(hash) {
            Ok(data) => {
                // Verify the healthy copy
                let actual_hash = crate::crypto::hash_bytes(&data);
                if actual_hash == *hash {
                    // Write to corrupted store
                    corrupted_store.delete_chunk(hash)?;
                    corrupted_store.put_chunk(hash, &data)?;
                    repaired += 1;
                } else {
                    warn!("Healthy source also has incorrect hash for {}", &hash[..12]);
                    failed += 1;
                }
            }
            Err(e) => {
                warn!("Cannot repair chunk {}: not in healthy source: {}", &hash[..12], e);
                failed += 1;
            }
        }
    }

    info!("Repair complete: {} repaired, {} failed", repaired, failed);

    Ok(RepairResult {
        chunks_repaired: repaired,
        chunks_failed: failed,
    })
}

/// Result of a repair operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct RepairResult {
    pub chunks_repaired: u64,
    pub chunks_failed: u64,
}
