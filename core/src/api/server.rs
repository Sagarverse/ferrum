//! Axum HTTP + WebSocket server.

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, delete},
    extract::State,
    Json,
};
use tower_http::cors::{CorsLayer, Any};
use tracing::info;

use crate::AppState;
use super::{events as event_stream, handlers};

/// Start the Axum server
pub async fn start(state: Arc<AppState>, port: u16) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Health & Status
        .route("/api/health", get(handlers::health_check))
        .route("/api/status", get(handlers::get_system_status))
        .route("/api/metrics", get(handlers::get_metrics))

        // Protected Items
        .route("/api/protected", get(handlers::get_protected_items))
        .route("/api/protected", post(handlers::add_protected_item))
        .route("/api/protected/:id", delete(handlers::remove_protected_item))
        .route("/api/protected/:id/replica", post(handlers::update_item_replica))

        // Versions
        .route("/api/versions/:item_id", get(handlers::get_versions))
        .route("/api/versions/delete/:version_id", delete(handlers::delete_version))
        .route("/api/items/:item_id/storage", get(handlers::get_item_storage))

        // Replicas
        .route("/api/replicas", get(handlers::get_replicas))
        .route("/api/replicas", post(handlers::add_replica))
        .route("/api/replicas/:id/toggle", post(handlers::toggle_replica))

        // Policies
        .route("/api/policies", get(handlers::get_policies))
        .route("/api/policies", post(handlers::create_policy))

        // Jobs
        .route("/api/jobs", get(handlers::get_recent_jobs))
        .route("/api/jobs/backup", post(handlers::trigger_backup))
        // Alias used by the UI's "Back up now" button.
        .route("/api/backup/trigger", post(handlers::trigger_backup))

        // Security
        .route("/api/security/status", get(handlers::get_security_status))
        .route("/api/incidents", get(handlers::get_incidents))

        // Recovery
        .route("/api/restore", post(handlers::restore_version))

        // Live event stream (SSE) — feeds the visual "How It Works" page.
        .route("/api/events/stream", get(event_stream::stream))

        // AI Chat
        .route("/api/ai/chat", post(handlers::ai_chat))

        // Settings
        .route("/api/settings", get(handlers::get_settings))
        .route("/api/settings", post(handlers::update_settings))
        
        // Factory Reset
        .route("/api/reset", post(handlers::reset_all))

        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    info!("API server listening on http://127.0.0.1:{}", port);

    axum::serve(listener, app).await?;
    Ok(())
}
