//! System health monitoring and protection scoring.

pub mod score;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::config::Settings;
use crate::db::Database;

pub use score::*;

/// Health monitor tracks overall system health
pub struct HealthMonitor {
    db: Arc<Database>,
    settings: Arc<RwLock<Settings>>,
}

impl HealthMonitor {
    pub fn new(db: Arc<Database>, settings: Arc<RwLock<Settings>>) -> Self {
        Self { db, settings }
    }

    /// Run the health monitoring loop
    pub async fn run(&self) {
        info!("Health monitor started");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            // Periodic health checks
            if let Ok(score) = self.calculate_health() {
                if score < 50 {
                    tracing::warn!("Health score degraded: {}", score);
                }
            }
        }
    }

    /// Calculate the overall health score
    pub fn calculate_health(&self) -> anyhow::Result<u32> {
        let mut score: u32 = 100;

        // Check replica health
        let replicas = self.db.get_replicas()?;
        for replica in &replicas {
            if replica.status != "healthy" {
                score = score.saturating_sub(15);
            }
        }

        // Check for active incidents
        let incidents = self.db.get_incidents()?;
        let active = incidents.iter().filter(|i| i.status == "active").count();
        score = score.saturating_sub(active as u32 * 20);

        Ok(score.min(100))
    }
}
