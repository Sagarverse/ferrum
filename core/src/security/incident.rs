//! Incident tracking and management.

use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use crate::db::{Database, IncidentRecord};

/// Incident manager for tracking security events
pub struct IncidentManager {
    db: Arc<Database>,
}

impl IncidentManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Get all incidents
    pub fn get_all(&self) -> anyhow::Result<Vec<IncidentRecord>> {
        self.db.get_incidents()
    }

    /// Get active incidents
    pub fn get_active(&self) -> anyhow::Result<Vec<IncidentRecord>> {
        let all = self.db.get_incidents()?;
        Ok(all.into_iter().filter(|i| i.status == "active" || i.status == "investigating").collect())
    }

    /// Resolve an incident
    pub fn resolve(&self, _id: &Uuid) -> anyhow::Result<()> {
        // In a complete implementation, update the incident status in DB
        Ok(())
    }

    /// Mark as false positive
    pub fn mark_false_positive(&self, _id: &Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}
