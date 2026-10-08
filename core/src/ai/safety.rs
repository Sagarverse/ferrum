//! AI safety layer: prevents destructive operations without confirmation.

use serde::Serialize;

/// Actions that require explicit user confirmation
const DESTRUCTIVE_ACTIONS: &[&str] = &[
    "delete", "remove", "unlink", "disable", "reduce_retention",
    "delete_versions", "change_encryption", "remove_recovery",
    "disable_protection", "permanent_delete",
];

/// Check if an AI-proposed action requires user confirmation
pub fn requires_confirmation(action: &str) -> bool {
    let action_lower = action.to_lowercase();
    DESTRUCTIVE_ACTIONS.iter().any(|&d| action_lower.contains(d))
}

/// An action preview shown to the user before execution
#[derive(Debug, Clone, Serialize)]
pub struct ActionPreview {
    pub action: String,
    pub description: String,
    pub is_destructive: bool,
    pub affected_items: Vec<String>,
    pub requires_confirmation: bool,
}

impl ActionPreview {
    pub fn new(action: &str, description: &str) -> Self {
        let is_destructive = requires_confirmation(action);
        Self {
            action: action.to_string(),
            description: description.to_string(),
            is_destructive,
            affected_items: Vec::new(),
            requires_confirmation: is_destructive,
        }
    }
}
