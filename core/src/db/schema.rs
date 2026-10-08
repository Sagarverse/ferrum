//! Database schema and migrations.

use rusqlite::Connection;
use tracing::info;

/// Current schema version
const SCHEMA_VERSION: u32 = 2;

/// Run all pending migrations
pub fn run_migrations(conn: &Connection) -> anyhow::Result<()> {
    // Create migration tracking table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    )?;

    let current_version: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < SCHEMA_VERSION {
        info!("Running database migrations from v{} to v{}", current_version, SCHEMA_VERSION);
    }

    if current_version < 1 {
        migrate_v1(conn)?;
    }
    if current_version < 2 {
        migrate_v2(conn)?;
    }

    Ok(())
}

/// V1: Initial schema
fn migrate_v1(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "
        -- Application settings (key-value)
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Protected items (files/folders being watched)
        CREATE TABLE IF NOT EXISTS protected_items (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL,
            item_type TEXT NOT NULL CHECK (item_type IN ('file', 'folder')),
            policy_id TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paused', 'unlinked', 'deleted')),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(path)
        );

        CREATE INDEX IF NOT EXISTS idx_protected_items_status ON protected_items(status);
        CREATE INDEX IF NOT EXISTS idx_protected_items_policy ON protected_items(policy_id);

        -- Backup policies
        CREATE TABLE IF NOT EXISTS policies (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            config_json TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Version snapshots
        CREATE TABLE IF NOT EXISTS versions (
            id TEXT PRIMARY KEY,
            protected_item_id TEXT NOT NULL,
            version_number INTEGER NOT NULL,
            manifest_hash TEXT NOT NULL,
            total_size INTEGER NOT NULL DEFAULT 0,
            total_chunks INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'healthy' CHECK (status IN ('healthy', 'suspicious', 'quarantined', 'deleted', 'creating')),
            created_at TEXT NOT NULL,
            metadata TEXT,
            FOREIGN KEY (protected_item_id) REFERENCES protected_items(id),
            UNIQUE(protected_item_id, version_number)
        );

        CREATE INDEX IF NOT EXISTS idx_versions_item ON versions(protected_item_id);
        CREATE INDEX IF NOT EXISTS idx_versions_status ON versions(status);

        -- Content-addressable chunk index
        CREATE TABLE IF NOT EXISTS chunks (
            hash TEXT PRIMARY KEY,
            size INTEGER NOT NULL,
            compressed_size INTEGER NOT NULL,
            ref_count INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_chunks_refcount ON chunks(ref_count);

        -- Replica destinations
        CREATE TABLE IF NOT EXISTS replicas (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            replica_type TEXT NOT NULL CHECK (replica_type IN ('mirror', 'encrypted')),
            encrypted INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'healthy' CHECK (status IN ('healthy', 'degraded', 'disconnected', 'corrupted', 'initializing')),
            last_sync_at TEXT,
            created_at TEXT NOT NULL
        );

        -- Backup/restore jobs
        CREATE TABLE IF NOT EXISTS jobs (
            id TEXT PRIMARY KEY,
            job_type TEXT NOT NULL CHECK (job_type IN ('backup', 'restore', 'verify', 'repair', 'gc', 'sync')),
            policy_id TEXT,
            status TEXT NOT NULL DEFAULT 'running' CHECK (status IN ('running', 'completed', 'failed', 'cancelled', 'queued')),
            started_at TEXT NOT NULL,
            completed_at TEXT,
            files_processed INTEGER NOT NULL DEFAULT 0,
            bytes_processed INTEGER NOT NULL DEFAULT 0,
            error_message TEXT,
            FOREIGN KEY (policy_id) REFERENCES policies(id)
        );

        CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
        CREATE INDEX IF NOT EXISTS idx_jobs_started ON jobs(started_at);

        -- Security incidents
        CREATE TABLE IF NOT EXISTS incidents (
            id TEXT PRIMARY KEY,
            severity TEXT NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
            threat_score INTEGER NOT NULL DEFAULT 0,
            process_name TEXT,
            process_path TEXT,
            affected_files INTEGER NOT NULL DEFAULT 0,
            description TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'investigating', 'resolved', 'false_positive')),
            created_at TEXT NOT NULL,
            resolved_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_incidents_severity ON incidents(severity);
        CREATE INDEX IF NOT EXISTS idx_incidents_status ON incidents(status);

        -- Audit log
        CREATE TABLE IF NOT EXISTS audit_log (
            id TEXT PRIMARY KEY,
            action TEXT NOT NULL,
            source TEXT,
            destination TEXT,
            details TEXT,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_log(created_at);

        -- File metadata cache (for change detection)
        CREATE TABLE IF NOT EXISTS file_metadata (
            path TEXT PRIMARY KEY,
            protected_item_id TEXT NOT NULL,
            size INTEGER NOT NULL,
            modified_at TEXT NOT NULL,
            hash TEXT,
            last_scanned_at TEXT NOT NULL,
            FOREIGN KEY (protected_item_id) REFERENCES protected_items(id)
        );

        CREATE INDEX IF NOT EXISTS idx_file_meta_item ON file_metadata(protected_item_id);

        -- Version manifest entries (file → chunks mapping)
        CREATE TABLE IF NOT EXISTS manifest_entries (
            version_id TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            file_hash TEXT NOT NULL,
            chunk_hashes TEXT NOT NULL,
            permissions TEXT,
            modified_at TEXT NOT NULL,
            PRIMARY KEY (version_id, file_path),
            FOREIGN KEY (version_id) REFERENCES versions(id)
        );

        -- Cloud provider configurations
        CREATE TABLE IF NOT EXISTS cloud_providers (
            id TEXT PRIMARY KEY,
            provider_type TEXT NOT NULL,
            name TEXT NOT NULL,
            config_json TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            last_upload_at TEXT,
            created_at TEXT NOT NULL
        );

        -- Notification queue
        CREATE TABLE IF NOT EXISTS notifications (
            id TEXT PRIMARY KEY,
            notification_type TEXT NOT NULL,
            title TEXT NOT NULL,
            message TEXT NOT NULL,
            severity TEXT NOT NULL DEFAULT 'info',
            read INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read);

        -- Record migration
        INSERT INTO schema_migrations (version) VALUES (1);
        "
    )?;

    info!("Applied migration v1: initial schema");
    Ok(())
}

/// V2: Add enabled flag to replicas and target_replica_id to protected_items
fn migrate_v2(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "
        ALTER TABLE replicas ADD COLUMN enabled INTEGER NOT NULL DEFAULT 1;
        ALTER TABLE protected_items ADD COLUMN target_replica_id TEXT;
        
        INSERT INTO schema_migrations (version) VALUES (2);
        "
    )?;

    info!("Applied migration v2: replicas enabled flag and protected_items target_replica_id");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_v1() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"protected_items".to_string()));
        assert!(tables.contains(&"versions".to_string()));
        assert!(tables.contains(&"chunks".to_string()));
        assert!(tables.contains(&"replicas".to_string()));
        assert!(tables.contains(&"jobs".to_string()));
        assert!(tables.contains(&"incidents".to_string()));
        assert!(tables.contains(&"audit_log".to_string()));
    }

    #[test]
    fn test_migration_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        // Running again should not fail
        run_migrations(&conn).unwrap();
    }
}
