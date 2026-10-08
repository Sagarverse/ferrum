//! Filesystem event types.

use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::Serialize;

/// A filesystem event detected by the watcher
#[derive(Debug, Clone, Serialize)]
pub struct FsEvent {
    /// Type of event
    pub kind: FsEventKind,
    /// Affected path
    pub path: PathBuf,
    /// When the event was detected
    pub timestamp: DateTime<Utc>,
}

/// Kinds of filesystem events
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FsEventKind {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf },
    MetadataChanged,
}

impl FsEvent {
    /// Convert a notify event to our event type
    pub fn from_notify_event(event: &notify::Event) -> Option<Self> {
        let path = event.paths.first()?.clone();
        let kind = match &event.kind {
            notify::EventKind::Create(_) => FsEventKind::Created,
            notify::EventKind::Modify(notify::event::ModifyKind::Data(_)) => FsEventKind::Modified,
            notify::EventKind::Modify(notify::event::ModifyKind::Name(_)) => {
                if event.paths.len() > 1 {
                    FsEventKind::Renamed { from: event.paths[0].clone() }
                } else {
                    FsEventKind::Modified
                }
            }
            notify::EventKind::Modify(notify::event::ModifyKind::Metadata(_)) => FsEventKind::MetadataChanged,
            notify::EventKind::Modify(_) => FsEventKind::Modified,
            notify::EventKind::Remove(_) => FsEventKind::Deleted,
            _ => return None,
        };

        let resolved_path = if matches!(&kind, FsEventKind::Renamed { .. }) && event.paths.len() > 1 {
            event.paths[1].clone()
        } else {
            path
        };

        Some(Self {
            kind,
            path: resolved_path,
            timestamp: Utc::now(),
        })
    }

    /// Check if this is a destructive event
    pub fn is_destructive(&self) -> bool {
        matches!(self.kind, FsEventKind::Deleted)
    }
}
