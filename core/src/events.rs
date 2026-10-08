//! Engine event bus.  Long-lived background subsystems emit events here so
//! the UI (via the SSE stream in `api/events.rs`) can render a live picture
//! of what's happening to user data.

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

/// A single thing the engine wants the UI to know about.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EngineEvent {
    /// A backup started for `item_id`.
    BackupStarted {
        item_id: Uuid,
        source_path: String,
        started_at: String,
    },
    /// One file finished chunking + storing.  Emitted per file so the
    /// visual page can animate each one moving through the pipeline.
    FileProcessed {
        item_id: Uuid,
        relative_path: String,
        size: u64,
        chunks: u64,
        new_chunks: u64,
        deduplicated: bool,
    },
    /// Backup finished — a new version was created in the database.
    BackupCompleted {
        item_id: Uuid,
        version: u32,
        files: u64,
        bytes: u64,
        chunks: u64,
        duration_ms: u64,
    },
    /// A restore started.
    RestoreStarted {
        item_id: Uuid,
        version: u32,
        destination: String,
    },
    /// One file written during a restore.
    RestoreFileWritten {
        item_id: Uuid,
        relative_path: String,
        bytes: u64,
    },
    /// A restore completed.
    RestoreCompleted {
        item_id: Uuid,
        version: u32,
        files_restored: u64,
        bytes_restored: u64,
        destination: String,
        errors: u64,
    },
    /// Periodic heartbeat so SSE clients can detect dropped connections.
    Heartbeat { timestamp: String },
}

/// The shared event bus.  One per process, owned by `AppState`.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<EngineEvent>,
}

impl EventBus {
    /// Create a bus with the given ring-buffer size.  New subscribers get
    /// only events emitted after they subscribe.
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { sender: tx }
    }

    /// Publish an event.  Returns the number of subscribers that received it.
    pub fn send(&self, event: EngineEvent) -> usize {
        // `send` errors only when there are no receivers; ignore those.
        let n = self.sender.send(event).unwrap_or_else(|e| {
            tracing::debug!("event bus send returned {} (no receivers or lagged)", e);
            0
        });
        n
    }

    /// Subscribe to the bus.  Returns a broadcast::Receiver.
    pub fn subscribe(&self) -> broadcast::Receiver<EngineEvent> {
        self.sender.subscribe()
    }
}
