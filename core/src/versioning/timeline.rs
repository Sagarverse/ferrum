//! Version timeline: browsing and querying version history.

use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;
use crate::db::{Database, VersionRecord};

/// Version timeline browser
pub struct Timeline {
    db: Arc<Database>,
}

impl Timeline {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Get all versions for a protected item
    pub fn get_versions(&self, item_id: &Uuid) -> anyhow::Result<Vec<VersionInfo>> {
        let versions = self.db.get_versions(item_id)?;
        Ok(versions.into_iter().map(VersionInfo::from).collect())
    }

    /// Get the initial (first) version
    pub fn get_initial_version(&self, item_id: &Uuid) -> anyhow::Result<Option<VersionInfo>> {
        let versions = self.db.get_versions(item_id)?;
        Ok(versions.into_iter().next().map(VersionInfo::from))
    }

    /// Get the latest healthy version
    pub fn get_latest_version(&self, item_id: &Uuid) -> anyhow::Result<Option<VersionInfo>> {
        let version = self.db.get_latest_version(item_id)?;
        Ok(version.map(VersionInfo::from))
    }

    /// Get the version count
    pub fn get_version_count(&self, item_id: &Uuid) -> anyhow::Result<u64> {
        self.db.get_version_count(item_id)
    }

    /// Get versions within a time range
    pub fn get_versions_in_range(
        &self,
        item_id: &Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<VersionInfo>> {
        let all = self.db.get_versions(item_id)?;
        Ok(all.into_iter()
            .filter(|v| v.created_at >= start && v.created_at <= end)
            .map(VersionInfo::from)
            .collect())
    }
}

/// User-friendly version information
#[derive(Debug, Clone, Serialize)]
pub struct VersionInfo {
    pub id: Uuid,
    pub version_number: u32,
    pub status: String,
    pub total_size: u64,
    pub total_chunks: u64,
    pub created_at: DateTime<Utc>,
    pub is_healthy: bool,
    pub label: String,
}

impl From<VersionRecord> for VersionInfo {
    fn from(v: VersionRecord) -> Self {
        let label = match v.version_number {
            1 => "Initial version".to_string(),
            _ => format!("Version {}", v.version_number),
        };

        Self {
            id: v.id,
            version_number: v.version_number,
            is_healthy: v.status == "healthy",
            status: v.status,
            total_size: v.total_size,
            total_chunks: v.total_chunks,
            created_at: v.created_at,
            label,
        }
    }
}
