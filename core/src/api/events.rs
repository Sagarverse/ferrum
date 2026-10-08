//! Server-Sent Events stream of engine activity.
//!
//! Long-lived HTTP response, `Content-Type: text/event-stream`.  Each event
//! from the bus becomes a single SSE frame:
//!
//! ```
//! event: backup_started
//! data: {"type":"backup_started", ...}
//!
//! ```
//!
//! The visual "How It Works" page in the renderer subscribes to this stream
//! via the IPC bridge and animates each event as it arrives.

use std::sync::Arc;
use std::convert::Infallible;
use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use tokio::sync::broadcast::error::RecvError;

use crate::AppState;

pub async fn stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_bus.subscribe();
    let outbound = async_stream::stream! {
        let mut rx = rx;
        loop {
            match rx.recv().await {
                Ok(event) => {
                    // Encode as JSON.  Use the variant name (e.g. "backup_started")
                    // as the SSE event name so the client can use addEventListener.
                    let kind = match &event {
                        crate::events::EngineEvent::BackupStarted { .. } => "backup_started",
                        crate::events::EngineEvent::FileProcessed { .. } => "file_processed",
                        crate::events::EngineEvent::BackupCompleted { .. } => "backup_completed",
                        crate::events::EngineEvent::RestoreStarted { .. } => "restore_started",
                        crate::events::EngineEvent::RestoreFileWritten { .. } => "restore_file_written",
                        crate::events::EngineEvent::RestoreCompleted { .. } => "restore_completed",
                        crate::events::EngineEvent::Heartbeat { .. } => "heartbeat",
                    };
                    let payload = serde_json::to_string(&event).unwrap_or_default();
                    yield Ok(Event::default()
                        .event(kind)
                        .data(payload));
                }
                Err(RecvError::Lagged(n)) => {
                    // Some subscribers were slow — tell the UI it missed frames.
                    yield Ok(Event::default()
                        .event("lagged")
                        .data(format!("{}", n)));
                }
                Err(RecvError::Closed) => break,
            }
        }
    };

    Sse::new(outbound).keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(15)))
}
