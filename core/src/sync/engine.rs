//! Sync engine: orchestrates change detection → chunk → dedup → store → replicate.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error};
use uuid::Uuid;

use crate::config::{Settings, CompressionAlgorithm};
use crate::db::Database;
use crate::events::{EngineEvent, EventBus};
use crate::versioning::SnapshotEngine;
use chrono::Utc;

/// Sync engine orchestrates the backup pipeline
pub struct SyncEngine {
    db: Arc<Database>,
    settings: Arc<RwLock<Settings>>,
    /// Whether sync is currently paused (e.g., due to quarantine)
    paused: Arc<RwLock<bool>>,
    /// Bus for publishing pipeline events to the UI.
    event_bus: EventBus,
}

impl SyncEngine {
    pub fn new(db: Arc<Database>, settings: Arc<RwLock<Settings>>, event_bus: EventBus) -> Self {
        Self {
            db,
            settings,
            paused: Arc::new(RwLock::new(false)),
            event_bus,
        }
    }

    /// Run a sync cycle for a protected item
    pub async fn sync_item(
        &self,
        item_id: &Uuid,
        source_path: &Path,
        store_path: &Path,
    ) -> anyhow::Result<SyncResult> {
        // Check if paused
        if *self.paused.read().await {
            return Ok(SyncResult {
                status: SyncStatus::Paused,
                files_synced: 0,
                bytes_synced: 0,
                new_version: None,
            });
        }

        let started = std::time::Instant::now();
        info!("Starting sync for item {} (source: {})", item_id, source_path.display());

        // Tell the UI a backup started.
        self.event_bus.send(EngineEvent::BackupStarted {
            item_id: *item_id,
            source_path: source_path.display().to_string(),
            started_at: Utc::now().to_rfc3339(),
        });

        // Create snapshot
        let engine = SnapshotEngine::new(
            Arc::clone(&self.db),
            store_path,
            CompressionAlgorithm::Lz4,
            self.event_bus.clone(),
        )?;

        let version = engine.create_snapshot(item_id, source_path)?;

        let elapsed = started.elapsed().as_millis() as u64;

        info!(
            "Sync complete for item {}: v{}, {} bytes",
            item_id, version.version_number, version.total_size
        );

        // Tell the UI the backup finished.
        self.event_bus.send(EngineEvent::BackupCompleted {
            item_id: *item_id,
            version: version.version_number,
            files: version.total_chunks.max(1),
            bytes: version.total_size,
            chunks: version.total_chunks,
            duration_ms: elapsed,
        });

        Ok(SyncResult {
            status: SyncStatus::Completed,
            files_synced: version.total_chunks, // approximate
            bytes_synced: version.total_size,
            new_version: Some(version.version_number),
        })
    }

    /// Pause sync (e.g., when quarantine is active)
    pub async fn pause(&self) {
        *self.paused.write().await = true;
        info!("Sync engine paused");
    }

    /// Resume sync
    pub async fn resume(&self) {
        *self.paused.write().await = false;
        info!("Sync engine resumed");
    }

    /// Check if sync is paused
    pub async fn is_paused(&self) -> bool {
        *self.paused.read().await
    }
}

/// Result of a sync operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncResult {
    pub status: SyncStatus,
    pub files_synced: u64,
    pub bytes_synced: u64,
    pub new_version: Option<u32>,
}

/// Sync operation status
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Completed,
    Paused,
    Failed,
    Quarantined,
}
