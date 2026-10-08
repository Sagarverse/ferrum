//! HTTP + WebSocket API server for UI communication.

pub mod server;
pub mod handlers;
pub mod models;
pub mod events;

use std::sync::Arc;

/// Start the API server
pub async fn serve(state: Arc<crate::AppState>, port: u16) -> anyhow::Result<()> {
    server::start(state, port).await
}
