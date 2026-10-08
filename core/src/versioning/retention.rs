//! Retention policy enforcement.
//!
//! Applies grandfather-father-son (GFS) rotation and other retention
//! strategies to manage version lifecycle.

use std::sync::Arc;
use chrono::{Utc, Duration};
use tracing::{info, debug};
use uuid::Uuid;

use crate::config::RetentionPolicy;
use crate::db::Database;

/// Retention engine enforces version lifecycle policies
pub struct RetentionEngine {
    db: Arc<Database>,
}

impl RetentionEngine {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Apply retention policy to a protected item's versions.
    /// Returns the number of versions marked for deletion.
    pub fn apply_retention(
        &self,
        item_id: &Uuid,
        policy: &RetentionPolicy,
    ) -> anyhow::Result<u64> {
        let versions = self.db.get_versions(item_id)?;
        let now = Utc::now();
        let mut to_delete = Vec::new();

        // Always keep at least version 1 (initial) and the latest
        let min_keep = 2;
        if versions.len() <= min_keep {
            return Ok(0);
        }

        // Identify versions to keep based on policy
        let keep_all_cutoff = now - Duration::hours(policy.keep_all_hours as i64);
        let keep_daily_cutoff = now - Duration::days(policy.keep_daily_days as i64);
        let keep_weekly_cutoff = now - Duration::weeks(policy.keep_weekly_weeks as i64);
        let keep_monthly_cutoff = now - Duration::days(policy.keep_monthly_months as i64 * 30);

        let mut kept_daily: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut kept_weekly: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut kept_monthly: std::collections::HashSet<String> = std::collections::HashSet::new();

        for (i, version) in versions.iter().enumerate() {
            // Always keep first and last
            if i == 0 || i == versions.len() - 1 {
                continue;
            }

            let created = version.created_at;

            // Keep all versions within keep_all window
            if created >= keep_all_cutoff {
                continue;
            }

            // Daily retention
            if created >= keep_daily_cutoff {
                let day_key = created.format("%Y-%m-%d").to_string();
                if kept_daily.contains(&day_key) {
                    to_delete.push(version.id);
                } else {
                    kept_daily.insert(day_key);
                }
                continue;
            }

            // Weekly retention
            if created >= keep_weekly_cutoff {
                let week_key = created.format("%Y-W%W").to_string();
                if kept_weekly.contains(&week_key) {
                    to_delete.push(version.id);
                } else {
                    kept_weekly.insert(week_key);
                }
                continue;
            }

            // Monthly retention
            if created >= keep_monthly_cutoff {
                let month_key = created.format("%Y-%m").to_string();
                if kept_monthly.contains(&month_key) {
                    to_delete.push(version.id);
                } else {
                    kept_monthly.insert(month_key);
                }
                continue;
            }

            // Beyond all retention windows — delete
            to_delete.push(version.id);
        }

        // Apply max_versions limit
        if policy.max_versions > 0 {
            let current_count = versions.len() - to_delete.len();
            if current_count > policy.max_versions as usize {
                let excess = current_count - policy.max_versions as usize;
                // Remove oldest non-protected versions
                let remaining: Vec<_> = versions.iter()
                    .filter(|v| !to_delete.contains(&v.id))
                    .collect();

                for v in remaining.iter().skip(1).take(excess) {
                    if v.id != versions.last().unwrap().id {
                        to_delete.push(v.id);
                    }
                }
            }
        }

        let delete_count = to_delete.len() as u64;

        if delete_count > 0 {
            info!(
                "Retention policy: marking {} versions for deletion on item {}",
                delete_count, item_id
            );
        }

        // Note: actual version deletion and chunk dereferencing would be done
        // by the garbage collection system. Here we just mark them.
        // In a full implementation, we'd update version status to "deleted"
        // and then run GC to clean up orphan chunks.

        Ok(delete_count)
    }
}
