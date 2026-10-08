//! SQLite database layer for metadata, jobs, and security events.
//!
//! Uses WAL mode for concurrent read access and provides
//! transactional operations for all data mutations.

pub mod schema;

use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;
use tracing::info;

/// Thread-safe database wrapper using SQLite with WAL mode
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open or create the database at the given data directory
    pub fn open(data_dir: &Path) -> anyhow::Result<Self> {
        let db_path = data_dir.join("ferrum.db");
        let conn = Connection::open(&db_path)?;

        // Enable WAL mode for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA synchronous=NORMAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.execute_batch("PRAGMA busy_timeout=5000;")?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        // Run migrations
        db.migrate()?;

        info!("Database opened at {}", db_path.display());
        Ok(db)
    }

    /// Open an in-memory database (for testing)
    pub fn open_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    /// Run schema migrations
    fn migrate(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        schema::run_migrations(&conn)?;
        Ok(())
    }

    // ─── Settings ──────────────────────────────────────────────

    /// Get a setting value by key
    pub fn get_setting(&self, key: &str) -> anyhow::Result<Option<String>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let result = stmt.query_row(rusqlite::params![key], |row| row.get::<_, String>(0));
        match result {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set a setting value
    pub fn set_setting(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    // ─── Protected Items ───────────────────────────────────────

    /// Add a protected item (file or folder)
    pub fn add_protected_item(&self, item: &ProtectedItem) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "INSERT INTO protected_items (id, path, item_type, policy_id, target_replica_id, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                item.id.to_string(),
                item.path,
                item.item_type,
                item.policy_id.to_string(),
                item.target_replica_id.map(|u| u.to_string()),
                item.status,
                item.created_at.to_rfc3339(),
                item.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get all protected items
    pub fn get_protected_items(&self) -> anyhow::Result<Vec<ProtectedItem>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, path, item_type, policy_id, target_replica_id, status, created_at, updated_at FROM protected_items"
        )?;
        let items = stmt.query_map([], |row| {
            Ok(ProtectedItem {
                id: uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                path: row.get(1)?,
                item_type: row.get(2)?,
                policy_id: uuid::Uuid::parse_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                target_replica_id: row.get::<_, Option<String>>(4)?.and_then(|s| uuid::Uuid::parse_str(&s).ok()),
                status: row.get(5)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_default(),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_default(),
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    /// Remove a protected item by ID
    pub fn remove_protected_item(&self, id: &uuid::Uuid) -> anyhow::Result<()> {
        let mut conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let tx = conn.transaction()?;
        let id_str = id.to_string();
        
        tx.execute("DELETE FROM manifest_entries WHERE version_id IN (SELECT id FROM versions WHERE protected_item_id = ?1)", rusqlite::params![&id_str])?;
        tx.execute("DELETE FROM versions WHERE protected_item_id = ?1", rusqlite::params![&id_str])?;
        tx.execute("DELETE FROM file_metadata WHERE protected_item_id = ?1", rusqlite::params![&id_str])?;
        tx.execute("DELETE FROM protected_items WHERE id = ?1", rusqlite::params![&id_str])?;
        
        tx.commit()?;
        Ok(())
    }

    /// Update the target replica for a protected item
    pub fn update_protected_item_replica(&self, item_id: &uuid::Uuid, replica_id: Option<&uuid::Uuid>) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let replica_str = replica_id.map(|u| u.to_string());
        conn.execute(
            "UPDATE protected_items SET target_replica_id = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![
                replica_str,
                chrono::Utc::now().to_rfc3339(),
                item_id.to_string()
            ],
        )?;
        Ok(())
    }

    /// Reset all data (wipes everything except settings and replicas)
    pub fn reset_all_data(&self) -> anyhow::Result<()> {
        let mut conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let tx = conn.transaction()?;
        
        tx.execute("DELETE FROM manifest_entries", [])?;
        tx.execute("DELETE FROM file_metadata", [])?;
        tx.execute("DELETE FROM jobs", [])?;
        tx.execute("DELETE FROM versions", [])?;
        tx.execute("DELETE FROM protected_items", [])?;
        tx.execute("DELETE FROM policies", [])?;
        tx.execute("DELETE FROM chunks", [])?;
        tx.execute("DELETE FROM replicas", [])?;
        tx.execute("DELETE FROM incidents", [])?;
        tx.execute("DELETE FROM audit_log", [])?;
        tx.execute("DELETE FROM notifications", [])?;
        
        tx.commit()?;
        Ok(())
    }

    // ─── Versions ──────────────────────────────────────────────

    /// Store a version record
    pub fn add_version(&self, version: &VersionRecord) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "INSERT INTO versions (id, protected_item_id, version_number, manifest_hash, 
             total_size, total_chunks, status, created_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                version.id.to_string(),
                version.protected_item_id.to_string(),
                version.version_number,
                version.manifest_hash,
                version.total_size as i64,
                version.total_chunks as i64,
                version.status,
                version.created_at.to_rfc3339(),
                version.metadata.as_deref().unwrap_or("{}"),
            ],
        )?;
        Ok(())
    }

    /// Get all versions for a protected item, ordered by version number
    pub fn get_versions(&self, item_id: &uuid::Uuid) -> anyhow::Result<Vec<VersionRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, protected_item_id, version_number, manifest_hash, total_size, 
             total_chunks, status, created_at, metadata
             FROM versions WHERE protected_item_id = ?1 ORDER BY version_number ASC"
        )?;
        let versions = stmt.query_map(rusqlite::params![item_id.to_string()], |row| {
            Ok(VersionRecord {
                id: uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                protected_item_id: uuid::Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                version_number: row.get(2)?,
                manifest_hash: row.get(3)?,
                total_size: row.get::<_, i64>(4)? as u64,
                total_chunks: row.get::<_, i64>(5)? as u64,
                status: row.get(6)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_default(),
                metadata: row.get(8).ok(),
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(versions)
    }

    /// Get the latest healthy version for a protected item
    pub fn get_latest_version(&self, item_id: &uuid::Uuid) -> anyhow::Result<Option<VersionRecord>> {
        let versions = self.get_versions(item_id)?;
        Ok(versions.into_iter().filter(|v| v.status == "healthy").last())
    }

    /// Delete a version
    pub fn delete_version(&self, version_id: &uuid::Uuid) -> anyhow::Result<()> {
        let mut conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let tx = conn.transaction()?;
        
        tx.execute("DELETE FROM manifest_entries WHERE version_id = ?1", rusqlite::params![version_id.to_string()])?;
        tx.execute("DELETE FROM versions WHERE id = ?1", rusqlite::params![version_id.to_string()])?;
        
        tx.commit()?;
        Ok(())
    }

    // ─── Chunks ────────────────────────────────────────────────

    /// Record a chunk in the index
    pub fn add_chunk(&self, chunk: &ChunkRecord) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "INSERT OR IGNORE INTO chunks (hash, size, compressed_size, ref_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                chunk.hash,
                chunk.size as i64,
                chunk.compressed_size as i64,
                chunk.ref_count as i64,
                chunk.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Increment the reference count for a chunk
    pub fn increment_chunk_ref(&self, hash: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "UPDATE chunks SET ref_count = ref_count + 1 WHERE hash = ?1",
            rusqlite::params![hash],
        )?;
        Ok(())
    }

    /// Decrement the reference count for a chunk
    pub fn decrement_chunk_ref(&self, hash: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "UPDATE chunks SET ref_count = ref_count - 1 WHERE hash = ?1 AND ref_count > 0",
            rusqlite::params![hash],
        )?;
        Ok(())
    }

    /// Check if a chunk exists
    pub fn chunk_exists(&self, hash: &str) -> anyhow::Result<bool> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM chunks WHERE hash = ?1",
            rusqlite::params![hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get all chunks with zero references (for garbage collection)
    pub fn get_orphan_chunks(&self) -> anyhow::Result<Vec<String>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let mut stmt = conn.prepare("SELECT hash FROM chunks WHERE ref_count <= 0")?;
        let hashes = stmt.query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(hashes)
    }

    /// Delete a chunk record
    pub fn delete_chunk(&self, hash: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute("DELETE FROM chunks WHERE hash = ?1", rusqlite::params![hash])?;
        Ok(())
    }

    // ─── Replicas ──────────────────────────────────────────────

    /// Add a replica destination
    pub fn add_replica(&self, replica: &ReplicaRecord) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "INSERT INTO replicas (id, name, path, replica_type, encrypted, enabled, status, 
             last_sync_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                replica.id.to_string(),
                replica.name,
                replica.path,
                replica.replica_type,
                replica.encrypted,
                replica.enabled,
                replica.status,
                replica.last_sync_at.map(|d| d.to_rfc3339()),
                replica.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get all replicas
    pub fn get_replicas(&self) -> anyhow::Result<Vec<ReplicaRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, path, replica_type, encrypted, enabled, status, last_sync_at, created_at
             FROM replicas"
        )?;
        let replicas = stmt.query_map([], |row| {
            Ok(ReplicaRecord {
                id: uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                name: row.get(1)?,
                path: row.get(2)?,
                replica_type: row.get(3)?,
                encrypted: row.get(4)?,
                enabled: row.get(5)?,
                status: row.get(6)?,
                last_sync_at: row.get::<_, Option<String>>(7)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&chrono::Utc)),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_default(),
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(replicas)
    }

    /// Update replica status
    pub fn update_replica_status(&self, id: &uuid::Uuid, status: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "UPDATE replicas SET status = ?1 WHERE id = ?2",
            rusqlite::params![status, id.to_string()],
        )?;
        Ok(())
    }

    /// Update replica last sync time
    pub fn update_replica_sync_time(&self, id: &uuid::Uuid) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "UPDATE replicas SET last_sync_at = datetime('now'), status = 'healthy' WHERE id = ?1",
            rusqlite::params![id.to_string()],
        )?;
        Ok(())
    }

    /// Toggle a replica enabled/disabled
    pub fn toggle_replica(&self, replica_id: &uuid::Uuid, enabled: bool) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "UPDATE replicas SET enabled = ?1 WHERE id = ?2",
            rusqlite::params![enabled, replica_id.to_string()],
        )?;
        Ok(())
    }

    // ─── Jobs ──────────────────────────────────────────────────

    /// Record a backup job
    pub fn add_job(&self, job: &JobRecord) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "INSERT INTO jobs (id, job_type, policy_id, status, started_at, completed_at,
             files_processed, bytes_processed, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                job.id.to_string(),
                job.job_type,
                job.policy_id.map(|id| id.to_string()),
                job.status,
                job.started_at.to_rfc3339(),
                job.completed_at.map(|d| d.to_rfc3339()),
                job.files_processed as i64,
                job.bytes_processed as i64,
                job.error_message,
            ],
        )?;
        Ok(())
    }

    /// Update a job's status and metrics
    pub fn update_job(&self, job: &JobRecord) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "UPDATE jobs SET status = ?1, completed_at = ?2, files_processed = ?3,
             bytes_processed = ?4, error_message = ?5 WHERE id = ?6",
            rusqlite::params![
                job.status,
                job.completed_at.map(|d| d.to_rfc3339()),
                job.files_processed as i64,
                job.bytes_processed as i64,
                job.error_message,
                job.id.to_string(),
            ],
        )?;
        Ok(())
    }

    /// Get recent jobs
    pub fn get_recent_jobs(&self, limit: u32) -> anyhow::Result<Vec<JobRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, job_type, policy_id, status, started_at, completed_at,
             files_processed, bytes_processed, error_message
             FROM jobs ORDER BY started_at DESC LIMIT ?1"
        )?;
        let jobs = stmt.query_map(rusqlite::params![limit], |row| {
            Ok(JobRecord {
                id: uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                job_type: row.get(1)?,
                policy_id: row.get::<_, Option<String>>(2)?
                    .and_then(|s| uuid::Uuid::parse_str(&s).ok()),
                status: row.get(3)?,
                started_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_default(),
                completed_at: row.get::<_, Option<String>>(5)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&chrono::Utc)),
                files_processed: row.get::<_, i64>(6)? as u64,
                bytes_processed: row.get::<_, i64>(7)? as u64,
                error_message: row.get(8)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(jobs)
    }

    // ─── Security Incidents ────────────────────────────────────

    /// Record a security incident
    pub fn add_incident(&self, incident: &IncidentRecord) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "INSERT INTO incidents (id, severity, threat_score, process_name, process_path,
             affected_files, description, status, created_at, resolved_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                incident.id.to_string(),
                incident.severity,
                incident.threat_score,
                incident.process_name,
                incident.process_path,
                incident.affected_files as i64,
                incident.description,
                incident.status,
                incident.created_at.to_rfc3339(),
                incident.resolved_at.map(|d| d.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    /// Get all incidents
    pub fn get_incidents(&self) -> anyhow::Result<Vec<IncidentRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, severity, threat_score, process_name, process_path, affected_files,
             description, status, created_at, resolved_at
             FROM incidents ORDER BY created_at DESC"
        )?;
        let incidents = stmt.query_map([], |row| {
            Ok(IncidentRecord {
                id: uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                severity: row.get(1)?,
                threat_score: row.get(2)?,
                process_name: row.get(3)?,
                process_path: row.get(4)?,
                affected_files: row.get::<_, i64>(5)? as u64,
                description: row.get(6)?,
                status: row.get(7)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_default(),
                resolved_at: row.get::<_, Option<String>>(9)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&chrono::Utc)),
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(incidents)
    }

    // ─── Audit Log ─────────────────────────────────────────────

    /// Add an audit log entry
    pub fn add_audit_log(&self, entry: &AuditEntry) -> anyhow::Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "INSERT INTO audit_log (id, action, source, destination, details, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                entry.id.to_string(),
                entry.action,
                entry.source,
                entry.destination,
                entry.details,
                entry.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    // ─── Policies ──────────────────────────────────────────────

    /// Save a backup policy
    pub fn save_policy(&self, policy: &crate::config::BackupPolicy) -> anyhow::Result<()> {
        let json = serde_json::to_string(policy)?;
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO policies (id, name, config_json, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                policy.id.to_string(),
                policy.name,
                json,
                policy.enabled,
                policy.created_at.to_rfc3339(),
                policy.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get all policies
    pub fn get_policies(&self) -> anyhow::Result<Vec<crate::config::BackupPolicy>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let mut stmt = conn.prepare("SELECT config_json FROM policies")?;
        let policies = stmt.query_map([], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?.filter_map(|r| {
            r.ok().and_then(|json| serde_json::from_str(&json).ok())
        }).collect();
        Ok(policies)
    }

    // ─── Statistics ────────────────────────────────────────────

    /// Get total protected data size
    pub fn get_total_protected_size(&self) -> anyhow::Result<u64> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let size: i64 = conn.query_row(
            "SELECT COALESCE(SUM(size), 0) FROM chunks",
            [],
            |row| row.get(0),
        )?;
        Ok(size as u64)
    }

    /// Get total deduplicated size
    pub fn get_total_dedup_size(&self) -> anyhow::Result<u64> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let size: i64 = conn.query_row(
            "SELECT COALESCE(SUM(compressed_size), 0) FROM chunks",
            [],
            |row| row.get(0),
        )?;
        Ok(size as u64)
    }

    /// Get version count for a protected item
    pub fn get_version_count(&self, item_id: &uuid::Uuid) -> anyhow::Result<u64> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock: {}", e))?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM versions WHERE protected_item_id = ?1",
            rusqlite::params![item_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }
}

// ─── Database Record Types ─────────────────────────────────────

/// A protected file or folder
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProtectedItem {
    pub id: uuid::Uuid,
    pub path: String,
    pub item_type: String,  // "file" or "folder"
    pub policy_id: uuid::Uuid,
    pub target_replica_id: Option<uuid::Uuid>,
    pub status: String,     // "active", "paused", "unlinked"
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// A version snapshot record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersionRecord {
    pub id: uuid::Uuid,
    pub protected_item_id: uuid::Uuid,
    pub version_number: u32,
    pub manifest_hash: String,
    pub total_size: u64,
    pub total_chunks: u64,
    pub status: String,    // "healthy", "suspicious", "quarantined", "deleted"
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<String>,
}

/// A chunk stored in the content-addressable store
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkRecord {
    pub hash: String,
    pub size: u64,
    pub compressed_size: u64,
    pub ref_count: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A replica destination
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReplicaRecord {
    pub id: uuid::Uuid,
    pub name: String,
    pub path: String,
    pub replica_type: String,  // "mirror" or "encrypted"
    pub encrypted: bool,
    pub enabled: bool,
    pub status: String,        // "healthy", "degraded", "disconnected", "corrupted"
    pub last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A backup/restore job
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JobRecord {
    pub id: uuid::Uuid,
    pub job_type: String,    // "backup", "restore", "verify", "repair", "gc"
    pub policy_id: Option<uuid::Uuid>,
    pub status: String,      // "running", "completed", "failed", "cancelled"
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub files_processed: u64,
    pub bytes_processed: u64,
    pub error_message: Option<String>,
}

/// A security incident
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IncidentRecord {
    pub id: uuid::Uuid,
    pub severity: String,     // "low", "medium", "high", "critical"
    pub threat_score: u32,
    pub process_name: Option<String>,
    pub process_path: Option<String>,
    pub affected_files: u64,
    pub description: String,
    pub status: String,       // "active", "investigating", "resolved", "false_positive"
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// An audit log entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub id: uuid::Uuid,
    pub action: String,
    pub source: Option<String>,
    pub destination: Option<String>,
    pub details: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
