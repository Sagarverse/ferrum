//! Security engine: ransomware detection, behavioral scoring, quarantine.

pub mod detector;
pub mod rules;
pub mod scoring;
pub mod quarantine;
pub mod incident;
pub mod process_monitor;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::config::Settings;
use crate::db::Database;
use crate::watcher::FsEvent;

pub use detector::*;
pub use scoring::*;
pub use quarantine::*;
pub use incident::*;

/// Security monitor runs in the background and analyzes filesystem events
pub struct SecurityMonitor {
    db: Arc<Database>,
    settings: Arc<RwLock<Settings>>,
    detector: ThreatDetector,
    quarantine: QuarantineManager,
}

impl SecurityMonitor {
    pub fn new(db: Arc<Database>, settings: Arc<RwLock<Settings>>) -> Self {
        let detector = ThreatDetector::new();
        let quarantine = QuarantineManager::new(Arc::clone(&db));

        Self {
            db,
            settings,
            detector,
            quarantine,
        }
    }

    /// Run the security monitor loop
    pub async fn run(&self) {
        info!("Security monitor started");
        // In production, this would poll the event queue and analyze events.
        // For now, we'll use a periodic health check loop.
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            // Periodic health check
        }
    }

    /// Analyze a filesystem event for suspicious activity
    pub fn analyze_event(&self, event: &FsEvent) -> ThreatAssessment {
        self.detector.analyze(event)
    }

    /// Get the quarantine manager
    pub fn quarantine(&self) -> &QuarantineManager {
        &self.quarantine
    }

    /// Get the current threat level
    pub fn current_threat_level(&self) -> ThreatLevel {
        self.detector.current_threat_level()
    }
}
