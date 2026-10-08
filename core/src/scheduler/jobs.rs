//! Job definitions and lifecycle management.

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Job status
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Job type
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    Backup,
    Restore,
    Verify,
    Repair,
    GarbageCollection,
    Sync,
}

/// A scheduled or running job
#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: JobType,
    pub status: JobStatus,
    pub progress: f64,
    pub files_processed: u64,
    pub bytes_processed: u64,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl Job {
    pub fn new(job_type: JobType) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_type,
            status: JobStatus::Queued,
            progress: 0.0,
            files_processed: 0,
            bytes_processed: 0,
            started_at: None,
            completed_at: None,
            error: None,
        }
    }
}
