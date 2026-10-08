//! Application settings and global configuration.

use serde::{Deserialize, Serialize};
use crate::db::Database;

/// Global application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// API server port
    pub api_port: Option<u16>,

    /// Whether the first-run wizard has been completed
    pub first_run_completed: bool,

    /// AI provider configuration
    pub ai_provider: AiProviderConfig,

    /// Notification settings
    pub notifications: NotificationSettings,

    /// Theme preference
    pub theme: Theme,

    /// Whether to start on system boot
    pub start_on_boot: bool,

    /// Whether to minimize to system tray
    pub minimize_to_tray: bool,

    /// Application password hash (for lockdown mode)
    pub app_password_hash: Option<String>,

    /// Maximum log retention in days
    pub log_retention_days: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_port: None,
            first_run_completed: false,
            ai_provider: AiProviderConfig::default(),
            notifications: NotificationSettings::default(),
            theme: Theme::Dark,
            start_on_boot: true,
            minimize_to_tray: true,
            app_password_hash: None,
            log_retention_days: 90,
        }
    }
}

impl Settings {
    /// Load settings from the database, or create defaults
    pub fn load(db: &Database) -> anyhow::Result<Self> {
        match db.get_setting("app_settings")? {
            Some(json_str) => {
                let settings: Settings = serde_json::from_str(&json_str)?;
                Ok(settings)
            }
            None => {
                let settings = Settings::default();
                db.set_setting("app_settings", &serde_json::to_string(&settings)?)?;
                Ok(settings)
            }
        }
    }

    /// Save settings to the database
    pub fn save(&self, db: &Database) -> anyhow::Result<()> {
        db.set_setting("app_settings", &serde_json::to_string(self)?)?;
        Ok(())
    }
}

/// AI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    /// Provider type
    pub provider: AiProvider,
    /// API endpoint URL
    pub endpoint: String,
    /// Model name
    pub model: String,
    /// API key (encrypted, stored separately)
    pub api_key_id: Option<String>,
}

impl Default for AiProviderConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::Ollama,
            endpoint: "http://localhost:11434".into(),
            model: "llama3.1".into(),
            api_key_id: None,
        }
    }
}

/// Supported AI providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiProvider {
    /// Local Ollama instance
    Ollama,
    /// OpenAI-compatible API
    OpenAi,
    /// Google Gemini API
    Gemini,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Enable OS notifications
    pub os_notifications: bool,
    /// Enable in-app notifications
    pub in_app_notifications: bool,
    /// Enable email notifications
    pub email_notifications: bool,
    /// Email address for notifications
    pub email_address: Option<String>,
    /// Notify on backup completion
    pub notify_backup_complete: bool,
    /// Notify on errors
    pub notify_errors: bool,
    /// Notify on security events
    pub notify_security: bool,
    /// Notify on replica disconnection
    pub notify_replica_disconnect: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            os_notifications: true,
            in_app_notifications: true,
            email_notifications: false,
            email_address: None,
            notify_backup_complete: false,
            notify_errors: true,
            notify_security: true,
            notify_replica_disconnect: true,
        }
    }
}

/// UI theme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Dark,
    Light,
    System,
}
