//! Event debouncer: coalesces rapid filesystem events to reduce noise.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use super::events::{FsEvent, FsEventKind};

/// Debounce window duration
const DEBOUNCE_MS: u64 = 500;

/// Event debouncer that coalesces rapid changes to the same file
pub struct EventDebouncer {
    pending: HashMap<PathBuf, DebouncedEntry>,
    window: Duration,
}

struct DebouncedEntry {
    event: FsEvent,
    first_seen: Instant,
    last_seen: Instant,
}

impl EventDebouncer {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
            window: Duration::from_millis(DEBOUNCE_MS),
        }
    }

    /// Add an event. Returns events that have settled (debounce window expired).
    pub fn add_event(&mut self, event: FsEvent) -> Vec<FsEvent> {
        let now = Instant::now();

        // Collect settled events first
        let settled = self.collect_settled(now);

        // Add or update this event
        if let Some(existing) = self.pending.get_mut(&event.path) {
            // Update existing — keep the most significant event kind
            existing.last_seen = now;
            existing.event.kind = merge_event_kinds(&existing.event.kind, &event.kind);
            existing.event.timestamp = event.timestamp;
        } else {
            self.pending.insert(event.path.clone(), DebouncedEntry {
                event,
                first_seen: now,
                last_seen: now,
            });
        }

        settled
    }

    /// Collect all events that have settled (no updates within debounce window)
    pub fn collect_settled(&mut self, now: Instant) -> Vec<FsEvent> {
        let mut settled = Vec::new();
        let mut to_remove = Vec::new();

        for (path, entry) in &self.pending {
            if now.duration_since(entry.last_seen) >= self.window {
                settled.push(entry.event.clone());
                to_remove.push(path.clone());
            }
        }

        for path in to_remove {
            self.pending.remove(&path);
        }

        settled
    }

    /// Flush all pending events regardless of debounce window
    pub fn flush(&mut self) -> Vec<FsEvent> {
        let events: Vec<FsEvent> = self.pending.values()
            .map(|e| e.event.clone())
            .collect();
        self.pending.clear();
        events
    }

    /// Number of pending events
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

/// Merge two event kinds — prefer the more significant one
fn merge_event_kinds(existing: &FsEventKind, new: &FsEventKind) -> FsEventKind {
    match (existing, new) {
        // Delete takes precedence over everything
        (_, FsEventKind::Deleted) => FsEventKind::Deleted,
        // Create + modify = create (file was just created and written to)
        (FsEventKind::Created, FsEventKind::Modified) => FsEventKind::Created,
        // Otherwise use the new event
        _ => new.clone(),
    }
}

impl Default for EventDebouncer {
    fn default() -> Self {
        Self::new()
    }
}
