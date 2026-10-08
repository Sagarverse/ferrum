//! Job scheduling and resource throttling.

pub mod jobs;
pub mod throttle;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::config::Settings;
use crate::db::Database;
use crate::sync::SyncEngine;

/// Job scheduler manages backup jobs and resource allocation
pub struct JobScheduler {
    db: Arc<Database>,
    settings: Arc<RwLock<Settings>>,
    sync_engine: Arc<SyncEngine>,
}

impl JobScheduler {
    pub fn new(
        db: Arc<Database>,
        settings: Arc<RwLock<Settings>>,
        sync_engine: Arc<SyncEngine>,
    ) -> Self {
        Self { db, settings, sync_engine }
    }

    /// Run the scheduler loop
    pub async fn run(&self) {
        info!("Job scheduler started");
        loop {
            // Check for scheduled jobs
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

            // In production: check policies, trigger backups based on schedule
        }
    }

    /// Get recent job history
    pub fn get_recent_jobs(&self, limit: u32) -> anyhow::Result<Vec<crate::db::JobRecord>> {
        self.db.get_recent_jobs(limit)
    }
}
