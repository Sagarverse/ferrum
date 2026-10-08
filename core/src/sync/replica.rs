//! Replica management: tracking and managing backup destinations.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;
use chrono::Utc;

use crate::db::{Database, ReplicaRecord};

/// Replica manager handles multiple backup destinations
pub struct ReplicaManager {
    db: Arc<Database>,
}

impl ReplicaManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Register a new replica destination
    pub fn add_replica(
        &self,
        name: &str,
        path: &Path,
        encrypted: bool,
    ) -> anyhow::Result<ReplicaRecord> {
        let replica = ReplicaRecord {
            id: Uuid::new_v4(),
            name: name.to_string(),
            path: path.display().to_string(),
            replica_type: if encrypted { "encrypted" } else { "mirror" }.to_string(),
            encrypted,
            enabled: true,
            status: "initializing".to_string(),
            last_sync_at: None,
            created_at: Utc::now(),
        };

        // Create the directory structure
        std::fs::create_dir_all(path)?;

        self.db.add_replica(&replica)?;
        info!("Added replica '{}' at {}", name, path.display());
        Ok(replica)
    }

    /// Get all registered replicas
    pub fn get_replicas(&self) -> anyhow::Result<Vec<ReplicaRecord>> {
        self.db.get_replicas()
    }

    /// Check if a replica's path is accessible
    pub fn check_replica_health(&self, replica: &ReplicaRecord) -> ReplicaHealth {
        let path = Path::new(&replica.path);

        if !path.exists() {
            return ReplicaHealth {
                status: "disconnected".to_string(),
                accessible: false,
                message: "Replica path does not exist or drive is disconnected".to_string(),
            };
        }

        // Check if we can write
        let test_file = path.join(".ferrum_health_check");
        match std::fs::write(&test_file, b"health_check") {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
                ReplicaHealth {
                    status: "healthy".to_string(),
                    accessible: true,
                    message: "Replica is accessible and writable".to_string(),
                }
            }
            Err(e) => {
                ReplicaHealth {
                    status: "degraded".to_string(),
                    accessible: false,
                    message: format!("Cannot write to replica: {}", e),
                }
            }
        }
    }

    /// Update replica status
    pub fn update_status(&self, id: &Uuid, status: &str) -> anyhow::Result<()> {
        self.db.update_replica_status(id, status)
    }

    /// Record a successful sync
    pub fn record_sync(&self, id: &Uuid) -> anyhow::Result<()> {
        self.db.update_replica_sync_time(id)
    }
}

/// Health status of a replica
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReplicaHealth {
    pub status: String,
    pub accessible: bool,
    pub message: String,
}
