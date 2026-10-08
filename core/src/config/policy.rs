//! Backup policies, protection levels, and retention rules.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Protection level determines the aggressiveness of backup and security measures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionLevel {
    /// Simple continuous protection, single replica
    Essentials,
    /// Versions + ransomware detection + multiple replicas
    Professional,
    /// Multiple replicas + encryption + advanced detection + emergency cloud
    Maximum,
    /// User-configured custom settings
    Custom,
}

impl Default for ProtectionLevel {
    fn default() -> Self {
        Self::Professional
    }
}

/// How frequently backups should occur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BackupFrequency {
    /// Monitor continuously and backup on every change
    Continuous,
    /// Real-time with a small debounce window
    RealTime,
    /// Every N minutes
    Interval { minutes: u32 },
    /// Hourly
    Hourly,
    /// Daily at a specific time
    Daily { hour: u8, minute: u8 },
    /// Weekly on a specific day
    Weekly { day: chrono::Weekday, hour: u8, minute: u8 },
    /// Custom cron expression
    Cron { expression: String },
    /// Only when system is idle
    IdleOnly,
    /// Manual (user-triggered only)
    Manual,
}

impl Default for BackupFrequency {
    fn default() -> Self {
        Self::Continuous
    }
}

/// Compression algorithm selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// LZ4 - fast compression, moderate ratio
    Lz4,
    /// Zstd - balanced speed and ratio
    Zstd { level: i32 },
}

impl Default for CompressionAlgorithm {
    fn default() -> Self {
        Self::Lz4
    }
}

/// Encryption algorithm selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EncryptionAlgorithm {
    /// No encryption
    None,
    /// AES-256-GCM (hardware accelerated on modern CPUs)
    Aes256Gcm,
    /// XChaCha20-Poly1305 (software-friendly)
    XChaCha20Poly1305,
}

impl Default for EncryptionAlgorithm {
    fn default() -> Self {
        Self::Aes256Gcm
    }
}

/// Ransomware detection policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RansomwarePolicy {
    /// Whether ransomware detection is enabled
    pub enabled: bool,
    /// Threshold for suspicious activity (0-100)
    pub quarantine_threshold: u8,
    /// Whether to automatically quarantine on critical threat
    pub auto_quarantine: bool,
    /// Whether to trigger emergency cloud escalation
    pub emergency_cloud_escalation: bool,
    /// Maximum file changes per second before alerting
    pub max_changes_per_second: u32,
    /// Maximum deletions per minute before alerting
    pub max_deletions_per_minute: u32,
}

impl Default for RansomwarePolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            quarantine_threshold: 70,
            auto_quarantine: true,
            emergency_cloud_escalation: false,
            max_changes_per_second: 50,
            max_deletions_per_minute: 100,
        }
    }
}

/// Retention policy for version management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Keep all versions for this many hours
    pub keep_all_hours: u32,
    /// Keep daily versions for this many days
    pub keep_daily_days: u32,
    /// Keep weekly versions for this many weeks
    pub keep_weekly_weeks: u32,
    /// Keep monthly versions for this many months
    pub keep_monthly_months: u32,
    /// Maximum total versions to keep (0 = unlimited)
    pub max_versions: u32,
    /// Maximum total storage for versions in bytes (0 = unlimited)
    pub max_storage_bytes: u64,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            keep_all_hours: 24,
            keep_daily_days: 30,
            keep_weekly_weeks: 12,
            keep_monthly_months: 12,
            max_versions: 0,
            max_storage_bytes: 0,
        }
    }
}

/// Resource limits for backup operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU usage percentage (1-100)
    pub max_cpu_percent: u8,
    /// Maximum memory usage in MB
    pub max_memory_mb: u32,
    /// Maximum concurrent file operations
    pub max_concurrent_ops: u16,
    /// Maximum background threads
    pub max_threads: u16,
    /// Maximum disk I/O bandwidth in MB/s (0 = unlimited)
    pub max_io_mbps: u32,
    /// Maximum network bandwidth in MB/s (0 = unlimited)
    pub max_network_mbps: u32,
    /// Only backup when on AC power
    pub ac_power_only: bool,
    /// Pause backup on battery below this percentage
    pub battery_threshold: u8,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: 25,
            max_memory_mb: 512,
            max_concurrent_ops: 4,
            max_threads: 2,
            max_io_mbps: 0,
            max_network_mbps: 0,
            ac_power_only: false,
            battery_threshold: 20,
        }
    }
}

/// File classification for smart backup prioritization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileClassification {
    Critical,
    Important,
    Standard,
    Temporary,
    Cache,
}

/// A complete backup policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPolicy {
    /// Unique policy identifier
    pub id: Uuid,
    /// Human-readable policy name
    pub name: String,
    /// Source paths to protect
    pub sources: Vec<String>,
    /// Glob patterns to exclude
    pub exclusions: Vec<String>,
    /// Replica destination IDs
    pub replica_ids: Vec<Uuid>,
    /// Backup frequency
    pub frequency: BackupFrequency,
    /// Protection level
    pub protection_level: ProtectionLevel,
    /// Retention policy
    pub retention: RetentionPolicy,
    /// Encryption algorithm
    pub encryption: EncryptionAlgorithm,
    /// Compression algorithm
    pub compression: CompressionAlgorithm,
    /// Ransomware detection policy
    pub ransomware: RansomwarePolicy,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Whether this policy is active
    pub enabled: bool,
    /// Cloud escalation target (cloud provider config ID)
    pub cloud_target: Option<Uuid>,
    /// When this policy was created
    pub created_at: DateTime<Utc>,
    /// When this policy was last modified
    pub updated_at: DateTime<Utc>,
}

impl BackupPolicy {
    /// Create a new backup policy with sensible defaults
    pub fn new(name: impl Into<String>, sources: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            sources,
            exclusions: default_exclusions(),
            replica_ids: Vec::new(),
            frequency: BackupFrequency::default(),
            protection_level: ProtectionLevel::default(),
            retention: RetentionPolicy::default(),
            encryption: EncryptionAlgorithm::default(),
            compression: CompressionAlgorithm::default(),
            ransomware: RansomwarePolicy::default(),
            resource_limits: ResourceLimits::default(),
            enabled: true,
            cloud_target: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Default file exclusion patterns for common cache/temp files
pub fn default_exclusions() -> Vec<String> {
    vec![
        // OS files
        ".DS_Store".into(),
        "Thumbs.db".into(),
        "desktop.ini".into(),
        // Editor temps
        "*.tmp".into(),
        "*.swp".into(),
        "*.swo".into(),
        "*~".into(),
        // Build artifacts
        "node_modules/**".into(),
        "target/**".into(),
        ".git/**".into(),
        "__pycache__/**".into(),
        "*.pyc".into(),
        // Adobe cache
        "Media Cache Files/**".into(),
        "Peak Files/**".into(),
        "Adobe Premiere Pro Auto-Save/**".into(),
        // Video render cache
        "*.bak".into(),
        "*.cache".into(),
        // IDE caches
        ".idea/**".into(),
        ".vscode/**".into(),
        ".vs/**".into(),
    ]
}
