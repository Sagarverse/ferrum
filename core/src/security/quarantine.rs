//! Quarantine manager: freezes sync propagation to protect healthy replicas.

use std::sync::Arc;
use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::db::{Database, IncidentRecord};

/// Quarantine manager protects healthy replicas during threats
pub struct QuarantineManager {
    db: Arc<Database>,
}

impl QuarantineManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Activate quarantine mode
    pub fn activate(
        &self,
        severity: &str,
        threat_score: u32,
        process_name: Option<&str>,
        affected_files: u64,
        description: &str,
    ) -> anyhow::Result<IncidentRecord> {
        warn!(
            "QUARANTINE ACTIVATED: {} (score: {}, affected: {} files)",
            description, threat_score, affected_files
        );

        let incident = IncidentRecord {
            id: Uuid::new_v4(),
            severity: severity.to_string(),
            threat_score,
            process_name: process_name.map(String::from),
            process_path: None,
            affected_files,
            description: description.to_string(),
            status: "active".to_string(),
            created_at: Utc::now(),
            resolved_at: None,
        };

        self.db.add_incident(&incident)?;

        info!("Incident {} created", incident.id);
        Ok(incident)
    }

    /// Get all active incidents
    pub fn get_active_incidents(&self) -> anyhow::Result<Vec<IncidentRecord>> {
        let all = self.db.get_incidents()?;
        Ok(all.into_iter().filter(|i| i.status == "active").collect())
    }

    /// Check if quarantine is currently active
    pub fn is_active(&self) -> anyhow::Result<bool> {
        let active = self.get_active_incidents()?;
        Ok(!active.is_empty())
    }
}
