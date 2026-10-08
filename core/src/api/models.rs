//! API request/response models.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AddProtectedItemRequest {
    pub path: String,
    pub name: Option<String>,
    pub protection_level: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddReplicaRequest {
    pub name: String,
    pub path: String,
    pub encrypted: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TriggerBackupRequest {
    pub item_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RestoreRequest {
    pub item_id: String,
    pub version: String, // "initial", "latest", or version number
    pub destination: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AiChatRequest {
    pub message: String,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct ToggleReplicaRequest {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItemReplicaRequest {
    pub replica_id: Option<String>,
}
