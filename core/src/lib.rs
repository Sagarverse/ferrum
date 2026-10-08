//! # Metallic Mini Core Engine
//!
//! Professional-grade data protection platform with continuous backup,
//! versioning, ransomware defense & automated recovery.
//!
//! ## Architecture
//!
//! The core engine is organized into layered modules:
//!
//! - **config**: Application configuration, backup policies, resource limits
//! - **db**: SQLite database layer for metadata, jobs, and security events
//! - **crypto**: BLAKE3 hashing, AES-256-GCM encryption, key management
//! - **repository**: Content-addressable storage, chunking, deduplication, compression
//! - **versioning**: Point-in-time snapshots, version timeline, retention policies
//! - **watcher**: Cross-platform filesystem monitoring (FSEvents/ReadDirectoryChangesW)
//! - **sync**: Live synchronization engine with change detection and replication
//! - **security**: Ransomware detection, behavioral scoring, quarantine management
//! - **recovery**: Data restoration, replica repair, delete protection
//! - **scheduler**: Job scheduling, resource throttling, backup policies
//! - **cloud**: Optional encrypted cloud escalation (Google Drive, S3, etc.)
//! - **ai**: AI-assisted configuration via natural language
//! - **api**: Local HTTP + WebSocket API for UI communication
//! - **health**: System health monitoring and protection scoring
//! - **platform**: OS-specific abstractions

pub mod config;
pub mod db;
pub mod crypto;
pub mod events;
pub mod repository;
pub mod versioning;
pub mod watcher;
pub mod sync;
pub mod security;
pub mod recovery;
pub mod scheduler;
pub mod cloud;
pub mod ai;
pub mod api;
pub mod health;
pub mod platform;

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "Metallic Mini";

/// Default API port for the local server
pub const DEFAULT_API_PORT: u16 = 19270;

/// Shared application state accessible by all subsystems
pub struct AppState {
    pub db: std::sync::Arc<db::Database>,
    pub settings: std::sync::Arc<tokio::sync::RwLock<config::Settings>>,
    pub watcher_handle: std::sync::Arc<tokio::sync::RwLock<Option<watcher::WatcherHandle>>>,
    pub sync_engine: std::sync::Arc<sync::SyncEngine>,
    pub security_monitor: std::sync::Arc<security::SecurityMonitor>,
    pub scheduler: std::sync::Arc<scheduler::JobScheduler>,
    pub health_monitor: std::sync::Arc<health::HealthMonitor>,
    /// Publishes engine events for the SSE stream and IPC log tail.
    pub event_bus: events::EventBus,
    pub shutdown_tx: tokio::sync::broadcast::Sender<()>,
}
