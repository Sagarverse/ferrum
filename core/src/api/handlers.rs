//! API request handlers.

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::Utc;

use crate::AppState;
use crate::db::{ProtectedItem, ReplicaRecord};
use crate::config::BackupPolicy;
use super::models::*;

type AppJson = Json<Value>;
type AppResult = Result<AppJson, (StatusCode, AppJson)>;

fn err(status: StatusCode, msg: &str) -> (StatusCode, AppJson) {
    (status, Json(json!({"error": msg})))
}

// ─── Health & Status ─────────────────────────────────────────

pub async fn health_check() -> AppJson {
    Json(json!({
        "status": "ok",
        "version": crate::VERSION,
        "timestamp": Utc::now().to_rfc3339()
    }))
}

pub async fn get_system_status(State(state): State<Arc<AppState>>) -> AppResult {
    let protected = state.db.get_protected_items().map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let replicas = state.db.get_replicas().map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let incidents = state.db.get_incidents().map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let mut total_size = 0;
    let mut dedup_size = 0;
    for item in &protected {
        let store_root = get_item_store_root(&state, item);
        if let Ok(store) = crate::repository::ContentStore::open(&store_root) {
            dedup_size += store.total_size().unwrap_or(0);
        }
        if let Ok(Some(latest_version)) = state.db.get_latest_version(&item.id) {
            total_size += latest_version.total_size;
        }
    }

    let active_incidents = incidents.iter().filter(|i| i.status == "active").count();
    let healthy_replicas = replicas.iter().filter(|r| r.status == "healthy").count();
    let threat_level = state.security_monitor.current_threat_level();

    // Calculate protection score
    let score = calculate_protection_score(
        protected.len(),
        replicas.len(),
        healthy_replicas,
        active_incidents,
    );

    Ok(Json(json!({
        "protection_score": score,
        "protected_items": protected.len(),
        "replicas": {
            "total": replicas.len(),
            "healthy": healthy_replicas,
        },
        "storage": {
            "total_size": total_size,
            "dedup_size": dedup_size,
            "saved": if total_size > dedup_size { total_size - dedup_size } else { 0 },
        },
        "security": {
            "threat_level": threat_level,
            "active_incidents": active_incidents,
        },
        "recovery": {
            "status": "ready",
        }
    })))
}

pub async fn get_metrics(State(state): State<Arc<AppState>>) -> AppResult {
    let items = state.db.get_protected_items().unwrap_or_default();
    let mut total_size = 0;
    let mut dedup_size = 0;
    
    for item in items {
        let store_root = get_item_store_root(&state, &item);
        if let Ok(store) = crate::repository::ContentStore::open(&store_root) {
            dedup_size += store.total_size().unwrap_or(0);
        }
        if let Ok(Some(latest_version)) = state.db.get_latest_version(&item.id) {
            total_size += latest_version.total_size;
        }
    }

    Ok(Json(json!({
        "storage": {
            "original_bytes": total_size,
            "stored_bytes": dedup_size,
            "dedup_ratio": if dedup_size > 0 { total_size as f64 / dedup_size as f64 } else { 1.0 },
        }
    })))
}

// ─── Protected Items ─────────────────────────────────────────

pub async fn get_protected_items(State(state): State<Arc<AppState>>) -> AppResult {
    let items = state.db.get_protected_items()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!({"items": items})))
}

pub async fn add_protected_item(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddProtectedItemRequest>,
) -> AppResult {
    let path = std::path::Path::new(&req.path);
    if !path.exists() {
        return Err(err(StatusCode::BAD_REQUEST, &format!("Path does not exist: {}", req.path)));
    }
    let item_type = if path.is_dir() { "folder" } else { "file" };

    let default_name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unnamed".to_string());
    let policy = BackupPolicy::new(&req.name.clone().unwrap_or(default_name), vec![req.path.clone()]);
    state.db.save_policy(&policy)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let item = ProtectedItem {
        id: Uuid::new_v4(),
        path: req.path.clone(),
        item_type: item_type.to_string(),
        policy_id: policy.id,
        target_replica_id: None,
        status: "active".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    state.db.add_protected_item(&item)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Set up a default replica pointing at ~/Library/Application Support/Ferrum/repositories
    if state.db.get_replicas().map(|r| r.is_empty()).unwrap_or(true) {
        let store_root = platform_default_repo_root();
        let _ = std::fs::create_dir_all(&store_root);
        let replica = crate::db::ReplicaRecord {
            id: Uuid::new_v4(),
            name: "Local Repository".to_string(),
            path: store_root.to_string_lossy().to_string(),
            replica_type: "mirror".to_string(),
            encrypted: false,
            enabled: true,
            status: "healthy".to_string(),
            last_sync_at: None,
            created_at: Utc::now(),
        };
        let _ = state.db.add_replica(&replica);
    }

    Ok(Json(json!({"item": item, "policy": policy})))
}

/// Default location where the local repository lives, per-platform.
fn platform_default_repo_root() -> std::path::PathBuf {
    if cfg!(target_os = "macos") {
        dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("Ferrum").join("repositories")
    } else if cfg!(target_os = "windows") {
        dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("Ferrum").join("repositories")
    } else {
        dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("ferrum").join("repositories")
    }
}

fn get_item_store_root(state: &AppState, item: &ProtectedItem) -> std::path::PathBuf {
    let mut root = platform_default_repo_root();
    if let Some(rep_id) = item.target_replica_id {
        if let Ok(replicas) = state.db.get_replicas() {
            if let Some(rep) = replicas.iter().find(|r| r.id == rep_id && r.enabled) {
                root = std::path::PathBuf::from(&rep.path);
            }
        }
    }
    root.join(item.id.to_string())
}

pub async fn update_item_replica(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
    Json(req): Json<UpdateItemReplicaRequest>,
) -> AppResult {
    let item_uuid = Uuid::parse_str(&item_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid Item ID"))?;
    
    let rep_uuid = match req.replica_id {
        Some(s) if !s.is_empty() => Some(Uuid::parse_str(&s).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid Replica ID"))?),
        _ => None,
    };

    state.db.update_protected_item_replica(&item_uuid, rep_uuid.as_ref())
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(json!({"status": "updated"})))
}

pub async fn remove_protected_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid ID"))?;

    // Physically delete the backup storage directory from disk
    let item = state.db.get_protected_items().unwrap_or_default().into_iter().find(|i| i.id == uuid);
    let store_root = item.map(|i| get_item_store_root(&state, &i)).unwrap_or_else(|| platform_default_repo_root().join(uuid.to_string()));
    let _ = std::fs::remove_dir_all(&store_root);

    // Remove from the database (cascades to versions & metadata)
    state.db.remove_protected_item(&uuid)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(json!({"status": "removed"})))
}

pub async fn reset_all(State(state): State<Arc<AppState>>) -> AppResult {
    // Wipe all data from the database
    state.db.reset_all_data()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Physically delete the root repository folder and recreate it
    let store_root = platform_default_repo_root();
    let _ = std::fs::remove_dir_all(&store_root);
    let _ = std::fs::create_dir_all(&store_root);

    Ok(Json(json!({"status": "reset_complete"})))
}

// ─── Versions ────────────────────────────────────────────────

pub async fn get_versions(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
) -> AppResult {
    let uuid = Uuid::parse_str(&item_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid ID"))?;

    let versions = state.db.get_versions(&uuid)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(json!({"versions": versions})))
}

pub async fn delete_version(
    State(state): State<Arc<AppState>>,
    Path(version_id): Path<String>,
) -> AppResult {
    let uuid = Uuid::parse_str(&version_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid ID"))?;

    state.db.delete_version(&uuid)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(json!({"status": "removed"})))
}

/// Per-item storage summary — what the visual "How It Works" page needs to
/// show how many chunks, files, bytes, versions, and replicas each protected
/// item has.
pub async fn get_item_storage(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
) -> AppResult {
    let uuid = Uuid::parse_str(&item_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid ID"))?;

    let versions = state.db.get_versions(&uuid)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let replicas = state.db.get_replicas()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let item = state.db.get_protected_items().unwrap_or_default().into_iter().find(|i| i.id == uuid);
    let store_root = item.map(|i| get_item_store_root(&state, &i)).unwrap_or_else(|| platform_default_repo_root().join(uuid.to_string()));
    let (chunk_count, stored_bytes) = match crate::repository::ContentStore::open(&store_root) {
        Ok(store) => (
            store.chunk_count().unwrap_or(0),
            store.total_size().unwrap_or(0),
        ),
        Err(_) => (0, 0),
    };

    let bytes: u64 = versions.iter().map(|v| v.total_size).max().unwrap_or(0);
    let dedup_ratio = if stored_bytes > 0 { bytes as f64 / stored_bytes as f64 } else { 1.0 };

    // Pull encryption flag from the policy that owns this item.
    let policies = state.db.get_policies()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let item_policy = state.db.get_protected_items()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .into_iter()
        .find(|i| i.id == uuid)
        .and_then(|i| policies.into_iter().find(|p| p.id == i.policy_id));
    let encrypted = matches!(
        item_policy.as_ref().map(|p| &p.encryption),
        Some(crate::config::EncryptionAlgorithm::Aes256Gcm)
    );

    Ok(Json(json!({
        "item_id": uuid,
        "files": versions.iter().map(|v| v.total_chunks).max().unwrap_or(0), // approx
        "chunks": chunk_count,
        "bytes": bytes,
        "stored_bytes": stored_bytes,
        "versions": versions.len(),
        "dedup_ratio": dedup_ratio,
        "encrypted": encrypted,
        "replicas": replicas.len(),
    })))
}

// ─── Replicas ────────────────────────────────────────────────

pub async fn get_replicas(State(state): State<Arc<AppState>>) -> AppResult {
    let replicas = state.db.get_replicas()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!({"replicas": replicas})))
}

pub async fn add_replica(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddReplicaRequest>,
) -> AppResult {
    let replica = ReplicaRecord {
        id: Uuid::new_v4(),
        name: req.name,
        path: req.path,
        replica_type: if req.encrypted { "encrypted" } else { "mirror" }.to_string(),
        encrypted: req.encrypted,
        enabled: true,
        status: "initializing".to_string(),
        last_sync_at: None,
        created_at: Utc::now(),
    };

    state.db.add_replica(&replica)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(json!({"replica": replica})))
}

pub async fn toggle_replica(
    State(state): State<Arc<AppState>>,
    Path(replica_id): Path<String>,
    Json(req): Json<ToggleReplicaRequest>,
) -> AppResult {
    let uuid = Uuid::parse_str(&replica_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid ID"))?;

    state.db.toggle_replica(&uuid, req.enabled)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(json!({"status": "updated"})))
}

// ─── Policies ────────────────────────────────────────────────

pub async fn get_policies(State(state): State<Arc<AppState>>) -> AppResult {
    let policies = state.db.get_policies()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!({"policies": policies})))
}

pub async fn create_policy(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreatePolicyRequest>,
) -> AppResult {
    let policy = BackupPolicy::new(req.name, req.sources);
    state.db.save_policy(&policy)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!({"policy": policy})))
}

// ─── Jobs ────────────────────────────────────────────────────

pub async fn get_recent_jobs(State(state): State<Arc<AppState>>) -> AppResult {
    let jobs = state.scheduler.get_recent_jobs(20)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!({"jobs": jobs})))
}

pub async fn trigger_backup(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TriggerBackupRequest>,
) -> AppResult {
    let uuid = Uuid::parse_str(&req.item_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid item ID"))?;

    // Look up the protected item
    let item = state.db.get_protected_items()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .into_iter()
        .find(|i| i.id == uuid)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "Protected item not found"))?;

    // Determine repository location: per-item folder inside the target replica's path
    let store_root = get_item_store_root(&state, &item);
    let _ = std::fs::create_dir_all(&store_root);

    // Run the sync pipeline (chunks → dedup → store)
    let source = std::path::PathBuf::from(&item.path);
    let result = state.sync_engine.sync_item(&item.id, &source, &store_root).await;

    // Record the job
    let job = crate::db::JobRecord {
        id: Uuid::new_v4(),
        job_type: "backup".to_string(),
        policy_id: Some(item.policy_id),
        status: match &result {
            Ok(_) => "completed".to_string(),
            Err(_) => "failed".to_string(),
        },
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
        files_processed: result.as_ref().map(|r| r.files_synced).unwrap_or(0),
        bytes_processed: result.as_ref().map(|r| r.bytes_synced).unwrap_or(0),
        error_message: result.as_ref().err().map(|e| e.to_string()),
    };
    let _ = state.db.add_job(&job);

    match result {
        Ok(r) => Ok(Json(json!({
            "status": "backup_completed",
            "item_id": item.id,
            "version": r.new_version,
            "files_synced": r.files_synced,
            "bytes_synced": r.bytes_synced,
            "job_id": job.id,
        }))),
        Err(e) => Err(err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Backup failed: {}", e))),
    }
}

// ─── Security ────────────────────────────────────────────────

pub async fn get_security_status(State(state): State<Arc<AppState>>) -> AppResult {
    let threat_level = state.security_monitor.current_threat_level();
    let incidents = state.db.get_incidents()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let active = incidents.iter().filter(|i| i.status == "active").count();

    Ok(Json(json!({
        "threat_level": threat_level,
        "active_incidents": active,
        "total_incidents": incidents.len(),
    })))
}

pub async fn get_incidents(State(state): State<Arc<AppState>>) -> AppResult {
    let incidents = state.db.get_incidents()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!({"incidents": incidents})))
}

// ─── Recovery ────────────────────────────────────────────────

pub async fn restore_version(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RestoreRequest>,
) -> AppResult {
    let item_uuid = Uuid::parse_str(&req.item_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid item ID"))?;

    // Look up the protected item
    let item = state.db.get_protected_items()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .into_iter()
        .find(|i| i.id == item_uuid)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "Protected item not found"))?;

    // Resolve the requested version.  Accept "initial", "latest", or a number string.
    let versions = state.db.get_versions(&item_uuid)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    if versions.is_empty() {
        return Err(err(StatusCode::NOT_FOUND, "No versions exist for this item"));
    }
    let target = match req.version.as_str() {
        "initial" => versions.first().cloned(),
        "latest" => versions.last().cloned(),
        other => versions.iter()
            .find(|v| v.version_number.to_string() == other)
            .cloned()
            .or_else(|| other.parse::<u32>().ok()
                .and_then(|n| versions.iter().find(|v| v.version_number == n).cloned())),
    }
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Requested version not found"))?;

    // Resolve destination.  Default to a sibling of the source.
    let destination = req.destination.unwrap_or_else(|| {
        let src = std::path::PathBuf::from(&item.path);
        let parent = src.parent().unwrap_or_else(|| std::path::Path::new("/"));
        let stem = src.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("restored");
        parent.join(format!("{}-restored", stem)).to_string_lossy().to_string()
    });
    let dest_path = std::path::PathBuf::from(&destination);

    // Open the content store that holds this item's chunks + manifests.
    let store_root = get_item_store_root(&state, &item);
    let store = crate::repository::ContentStore::open(&store_root)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Open store: {}", e)))?;

    // Load the manifest referenced by the version record.
    let manifest = crate::repository::Manifest::load_from_store(&store, &target.manifest_hash)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Load manifest: {}", e)))?;

    // Do the real work — this re-assembles chunks, decompresses, writes files,
    // and verifies each one with BLAKE3.  See core/src/recovery/restore.rs.
    let result = crate::recovery::restore_version(&store, &manifest, &dest_path)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Restore: {}", e)))?;

    // Emit a completion event so the visual page can show what happened.
    let _ = state.event_bus.send(crate::events::EngineEvent::RestoreCompleted {
        item_id: item.id,
        version: target.version_number,
        files_restored: result.files_restored,
        bytes_restored: result.bytes_restored,
        destination: destination.clone(),
        errors: result.errors.len() as u64,
    });

    Ok(Json(json!({
        "status": result.is_successful().then_some("restore_completed").unwrap_or("restore_partial"),
        "item_id": item.id,
        "version": target.version_number,
        "destination": destination,
        "files_restored": result.files_restored,
        "bytes_restored": result.bytes_restored,
        "errors": result.errors,
    })))
}

// ─── AI Chat ─────────────────────────────────────────────────

pub async fn ai_chat(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AiChatRequest>,
) -> AppResult {
    let system_prompt = r#"You are the AI assistant for Ferrum, a local data protection software.
Your job is to help the user manage their backups. 
Always respond in strict JSON format:
{
  "response": "Your conversational reply to the user",
  "action": null OR { "type": "protect"|"remove"|"list", "path": "/optional/file/path" }
}
If the user wants to protect a file/folder, return action type "protect" with the path.
If they want to list backups, return action type "list".
If they want to remove a file, return action type "remove" with the path.
Otherwise, action should be null."#;

    let body = serde_json::json!({
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": req.message }
        ],
        "jsonMode": true,
        "model": "openai"
    });

    let client = reqwest::Client::new();
    match client.post("https://text.pollinations.ai/")
        .json(&body)
        .send()
        .await 
    {
        Ok(resp) => {
            if let Ok(text) = resp.text().await {
                Ok(Json(json!({
                    "response": text,
                    "tool_calls": []
                })))
            } else {
                let fallback = generate_fallback_response(&req.message, &state).await;
                Ok(Json(json!({ "response": fallback, "ai_available": false })))
            }
        }
        Err(e) => {
            let fallback = generate_fallback_response(&req.message, &state).await;
            Ok(Json(json!({ "response": fallback, "ai_available": false, "error": e.to_string() })))
        }
    }
}

/// Generate a helpful response when AI is unavailable
async fn generate_fallback_response(message: &str, state: &AppState) -> String {
    let msg_lower = message.to_lowercase();

    if msg_lower.contains("protect") || msg_lower.contains("backup") {
        "To protect a folder, use the 'Protected Data' page in the sidebar, or I can help you set it up. Click the '+' button to add a new folder for protection.".to_string()
    } else if msg_lower.contains("status") || msg_lower.contains("health") {
        let items = state.db.get_protected_items().unwrap_or_default();
        let replicas = state.db.get_replicas().unwrap_or_default();
        format!(
            "System Status: {} protected items, {} replicas configured. Check the dashboard for detailed health information.",
            items.len(), replicas.len()
        )
    } else if msg_lower.contains("restore") || msg_lower.contains("recover") {
        "To restore data, go to the 'Recovery' page, select the protected item, choose a version, and click 'Restore'.".to_string()
    } else if msg_lower.contains("version") {
        "You can browse version history on the 'Versions' page. Each version shows a timeline of changes with the ability to restore any point.".to_string()
    } else {
        "I'm your Ferrum assistant. I can help you protect folders, check backup status, restore versions, and configure policies. What would you like to do?".to_string()
    }
}

// ─── Settings ────────────────────────────────────────────────

pub async fn get_settings(State(state): State<Arc<AppState>>) -> AppResult {
    let settings = state.settings.read().await;
    Ok(Json(serde_json::to_value(&*settings).unwrap_or(json!({}))))
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<crate::config::Settings>,
) -> AppResult {
    let mut settings = state.settings.write().await;
    *settings = new_settings.clone();
    settings.save(&state.db)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!({"status": "updated"})))
}

// ─── Helpers ─────────────────────────────────────────────────

fn calculate_protection_score(
    protected_count: usize,
    replica_count: usize,
    healthy_replicas: usize,
    active_incidents: usize,
) -> u32 {
    if protected_count == 0 {
        return 0;
    }

    let mut score: u32 = 50; // Base score for having protection

    // Bonus for replicas
    if replica_count > 0 {
        score += 15;
        if replica_count >= 2 {
            score += 10;
        }
    }

    // Bonus for healthy replicas
    if replica_count > 0 && healthy_replicas == replica_count {
        score += 15;
    }

    // Penalty for incidents
    if active_incidents > 0 {
        score = score.saturating_sub(20 * active_incidents as u32);
    }

    // Bonus for having protection at all
    score += 10;

    score.min(100)
}
