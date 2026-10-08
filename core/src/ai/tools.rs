//! Tool definitions for AI-assisted configuration.

use serde_json::json;
use super::ToolDefinition;

/// Get all available tools for the AI
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "get_protected_folders".into(),
            description: "Get list of all protected folders and their status".into(),
            parameters: json!({"type": "object", "properties": {}}),
        },
        ToolDefinition {
            name: "protect_folder".into(),
            description: "Enable protection on a folder or file path".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to protect"},
                    "protection_level": {"type": "string", "enum": ["essentials", "professional", "maximum"]}
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "get_storage_status".into(),
            description: "Get current storage usage and health".into(),
            parameters: json!({"type": "object", "properties": {}}),
        },
        ToolDefinition {
            name: "get_versions".into(),
            description: "Get version history for a protected item".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Protected path"}
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "restore_version".into(),
            description: "Restore data from a specific version".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "version": {"type": "string", "enum": ["initial", "latest", "specific"]},
                    "version_number": {"type": "integer"},
                    "destination": {"type": "string"}
                },
                "required": ["path", "version"]
            }),
        },
        ToolDefinition {
            name: "create_backup_policy".into(),
            description: "Create a new backup policy with specified settings".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "sources": {"type": "array", "items": {"type": "string"}},
                    "frequency": {"type": "string"},
                    "retention_days": {"type": "integer"},
                    "encryption": {"type": "boolean"},
                    "cloud_escalation": {"type": "boolean"}
                },
                "required": ["name", "sources"]
            }),
        },
        ToolDefinition {
            name: "get_security_status".into(),
            description: "Get current security and threat status".into(),
            parameters: json!({"type": "object", "properties": {}}),
        },
        ToolDefinition {
            name: "get_repository_health".into(),
            description: "Get health status of all backup repositories".into(),
            parameters: json!({"type": "object", "properties": {}}),
        },
        ToolDefinition {
            name: "set_schedule".into(),
            description: "Configure backup schedule".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "policy_id": {"type": "string"},
                    "schedule": {"type": "string", "description": "Schedule description"}
                },
                "required": ["policy_id", "schedule"]
            }),
        },
        ToolDefinition {
            name: "get_system_metrics".into(),
            description: "Get system performance metrics".into(),
            parameters: json!({"type": "object", "properties": {}}),
        },
    ]
}
