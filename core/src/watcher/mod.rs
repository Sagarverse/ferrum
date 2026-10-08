//! Cross-platform filesystem monitoring.

pub mod events;
pub mod debouncer;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config as NotifyConfig};
use tokio::sync::mpsc;
use tracing::{info, error, debug};

pub use events::*;

/// Handle to a running filesystem watcher
pub struct WatcherHandle {
    _watcher: RecommendedWatcher,
    watched_paths: Vec<PathBuf>,
}

impl WatcherHandle {
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }
}

/// Start watching the given paths for filesystem events.
/// Returns a handle and a receiver for debounced events.
pub fn start_watching(
    paths: Vec<PathBuf>,
) -> anyhow::Result<(WatcherHandle, mpsc::Receiver<FsEvent>)> {
    let (tx, rx) = mpsc::channel::<FsEvent>(4096);

    let event_tx = tx.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        match res {
            Ok(event) => {
                if let Some(fs_event) = FsEvent::from_notify_event(&event) {
                    if let Err(e) = event_tx.blocking_send(fs_event) {
                        error!("Failed to send fs event: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("File watcher error: {}", e);
            }
        }
    })?;

    for path in &paths {
        if path.exists() {
            watcher.watch(path, RecursiveMode::Recursive)?;
            info!("Watching: {}", path.display());
        } else {
            error!("Path does not exist, skipping: {}", path.display());
        }
    }

    let handle = WatcherHandle {
        _watcher: watcher,
        watched_paths: paths,
    };

    Ok((handle, rx))
}
