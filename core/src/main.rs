//! Ferrum Daemon
//!
//! Entry point for the background protection service. Initializes all subsystems:
//! - SQLite databases
//! - File system watcher
//! - Sync engine
//! - Security monitor
//! - Job scheduler
//! - HTTP + WebSocket API server

use std::sync::Arc;
use tracing::{info, error};
use ferrum::{
    api, config, db, events, health, scheduler, security, sync, watcher,
    AppState, APP_NAME, VERSION, DEFAULT_API_PORT,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ferrum=info,tower_http=info".into()),
        )
        .with_target(true)
        .with_thread_ids(true)
        .init();

    info!("{} v{} starting...", APP_NAME, VERSION);

    // Determine data directory
    let data_dir = platform::get_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    info!("Data directory: {}", data_dir.display());

    // Initialize database
    let db = Arc::new(db::Database::open(&data_dir)?);
    info!("Database initialized");

    // Load settings
    let settings = config::Settings::load(&db)?;
    let settings = Arc::new(tokio::sync::RwLock::new(settings));

    // Create shutdown channel
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    // ONE shared event bus for the whole process. Subscribers (SSE) read from
    // AppState.event_bus; emitters (SyncEngine, handlers) must send through the
    // same instance or events get lost.
    let event_bus = events::EventBus::new(1024);

    // Initialize subsystems
    let sync_engine = Arc::new(sync::SyncEngine::new(
        Arc::clone(&db),
        Arc::clone(&settings),
        event_bus.clone(),
    ));

    let security_monitor = Arc::new(security::SecurityMonitor::new(
        Arc::clone(&db),
        Arc::clone(&settings),
    ));

    let scheduler = Arc::new(scheduler::JobScheduler::new(
        Arc::clone(&db),
        Arc::clone(&settings),
        Arc::clone(&sync_engine),
    ));

    let health_monitor = Arc::new(health::HealthMonitor::new(
        Arc::clone(&db),
        Arc::clone(&settings),
    ));

    // Build shared state — hand the SAME event bus to AppState so the SSE
    // handler subscribes to the same bus that SyncEngine emits on.
    let state = Arc::new(AppState {
        db: Arc::clone(&db),
        settings: Arc::clone(&settings),
        watcher_handle: Arc::new(tokio::sync::RwLock::new(None)),
        sync_engine: Arc::clone(&sync_engine),
        security_monitor: Arc::clone(&security_monitor),
        scheduler: Arc::clone(&scheduler),
        health_monitor: Arc::clone(&health_monitor),
        event_bus: event_bus.clone(),
        shutdown_tx: shutdown_tx.clone(),
    });

    // Start the scheduler
    let scheduler_handle = {
        let scheduler = Arc::clone(&scheduler);
        let mut shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            tokio::select! {
                _ = scheduler.run() => {},
                _ = shutdown_rx.recv() => {
                    info!("Scheduler shutting down");
                },
            }
        })
    };

    // Start the security monitor
    let security_handle = {
        let monitor = Arc::clone(&security_monitor);
        let mut shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            tokio::select! {
                _ = monitor.run() => {},
                _ = shutdown_rx.recv() => {
                    info!("Security monitor shutting down");
                },
            }
        })
    };

    // Start the health monitor
    let health_handle = {
        let monitor = Arc::clone(&health_monitor);
        let mut shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            tokio::select! {
                _ = monitor.run() => {},
                _ = shutdown_rx.recv() => {
                    info!("Health monitor shutting down");
                },
            }
        })
    };

    // Start API server
    let port = {
        let s = settings.read().await;
        s.api_port.unwrap_or(DEFAULT_API_PORT)
    };

    info!("Starting API server on port {}", port);

    let api_handle = {
        let state = Arc::clone(&state);
        let mut shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            tokio::select! {
                result = api::serve(state, port) => {
                    if let Err(e) = result {
                        error!("API server error: {}", e);
                    }
                },
                _ = shutdown_rx.recv() => {
                    info!("API server shutting down");
                },
            }
        })
    };

    info!("{} v{} is running. API available at http://localhost:{}", APP_NAME, VERSION, port);

    // Wait for shutdown signal (Ctrl+C)
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    let _ = shutdown_tx.send(());

    // Wait for all tasks
    let _ = tokio::join!(scheduler_handle, security_handle, health_handle, api_handle);

    info!("{} shut down cleanly", APP_NAME);
    Ok(())
}

mod platform {
    use std::path::PathBuf;
    use anyhow::Result;

    pub fn get_data_dir() -> Result<PathBuf> {
        let dir = if cfg!(target_os = "macos") {
            dirs::data_local_dir()
                .map(|d| d.join("Ferrum"))
                .unwrap_or_else(|| PathBuf::from("./ferrum-data"))
        } else if cfg!(target_os = "windows") {
            dirs::data_local_dir()
                .map(|d| d.join("Ferrum"))
                .unwrap_or_else(|| PathBuf::from("C:\\ProgramData\\Ferrum"))
        } else {
            dirs::data_local_dir()
                .map(|d| d.join("ferrum"))
                .unwrap_or_else(|| PathBuf::from("./ferrum-data"))
        };
        Ok(dir)
    }
}
