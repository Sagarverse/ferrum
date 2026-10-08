//! Delete protection: detects source deletion and preserves versions.
//!
//! When a protected file/folder is deleted from the source:
//! - The protected versions in repositories are preserved
//! - The user is notified
//! - The user must explicitly unlink in Metallic Mini to remove protection
//! - Provides restore capability for deleted data

use std::path::Path;
use std::sync::Arc;
use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::db::{Database, AuditEntry};

/// Delete protection manager
pub struct DeleteProtection {
    db: Arc<Database>,
}

impl DeleteProtection {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Check if a protected path has been deleted from the source
    pub fn check_source_deletion(&self, path: &Path) -> bool {
        !path.exists()
    }

    /// Handle a detected source deletion
    pub fn on_source_deleted(&self, item_id: &Uuid, path: &str) -> anyhow::Result<()> {
        warn!("Protected data deletion detected: {}", path);

        // Log the event
        let audit = AuditEntry {
            id: Uuid::new_v4(),
            action: "source_deletion_detected".to_string(),
            source: Some(path.to_string()),
            destination: None,
            details: Some(format!(
                "Protected data at '{}' was deleted from source. Protected versions are preserved.",
                path
            )),
            created_at: Utc::now(),
        };
        self.db.add_audit_log(&audit)?;

        info!(
            "Source deletion recorded for item {}. Protected versions are safe.",
            item_id
        );

        Ok(())
    }

    /// Unlink protection (user explicitly removes protection)
    pub fn unlink_protection(&self, item_id: &Uuid) -> anyhow::Result<()> {
        info!("Unlinking protection for item {}", item_id);

        // Remove the protected item
        self.db.remove_protected_item(item_id)?;

        // Log the action
        let audit = AuditEntry {
            id: Uuid::new_v4(),
            action: "protection_unlinked".to_string(),
            source: Some(item_id.to_string()),
            destination: None,
            details: Some("User explicitly unlinked protection".to_string()),
            created_at: Utc::now(),
        };
        self.db.add_audit_log(&audit)?;

        Ok(())
    }
}
