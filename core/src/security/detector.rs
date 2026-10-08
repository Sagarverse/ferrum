//! Threat detector: analyzes filesystem events for suspicious patterns.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{info, warn};

use crate::watcher::{FsEvent, FsEventKind};
use super::scoring::{ThreatAssessment, ThreatLevel};

/// Time window for rate analysis (5 seconds)
const RATE_WINDOW_SECS: u64 = 5;

/// Threshold for mass deletion alert
const MASS_DELETE_THRESHOLD: usize = 20;

/// Threshold for mass modification alert
const MASS_MODIFY_THRESHOLD: usize = 50;

/// Known ransomware file extensions
const RANSOMWARE_EXTENSIONS: &[&str] = &[
    ".encrypted", ".locked", ".crypto", ".crypt", ".enc",
    ".locky", ".cerber", ".zepto", ".odin", ".thor",
    ".zzzzz", ".micro", ".mp3", // yes, some ransomware uses .mp3
    ".xxx", ".ttt", ".abc", ".aaa",
    ".wncry", ".wnry", ".wcry", ".wncrypt",
    ".WNCRY", ".WNCRYT",
    ".cryptolocker", ".cryptowall",
    ".legion", ".darkness",
];

/// Threat detector uses multiple signals to identify suspicious behavior
pub struct ThreatDetector {
    /// Recent events for rate analysis
    recent_events: Mutex<EventWindow>,
    /// Current aggregate threat level
    current_level: Mutex<ThreatLevel>,
}

struct EventWindow {
    creates: VecDeque<Instant>,
    modifies: VecDeque<Instant>,
    deletes: VecDeque<Instant>,
    renames: VecDeque<Instant>,
    extension_changes: VecDeque<Instant>,
}

impl EventWindow {
    fn new() -> Self {
        Self {
            creates: VecDeque::new(),
            modifies: VecDeque::new(),
            deletes: VecDeque::new(),
            renames: VecDeque::new(),
            extension_changes: VecDeque::new(),
        }
    }

    fn prune(&mut self, window: Duration) {
        let cutoff = Instant::now() - window;
        self.creates.retain(|t| *t > cutoff);
        self.modifies.retain(|t| *t > cutoff);
        self.deletes.retain(|t| *t > cutoff);
        self.renames.retain(|t| *t > cutoff);
        self.extension_changes.retain(|t| *t > cutoff);
    }
}

impl ThreatDetector {
    pub fn new() -> Self {
        Self {
            recent_events: Mutex::new(EventWindow::new()),
            current_level: Mutex::new(ThreatLevel::Normal),
        }
    }

    /// Analyze a filesystem event and return a threat assessment
    pub fn analyze(&self, event: &FsEvent) -> ThreatAssessment {
        let now = Instant::now();
        let window = Duration::from_secs(RATE_WINDOW_SECS);

        let mut events = self.recent_events.lock().unwrap();
        events.prune(window);

        // Record the event
        match &event.kind {
            FsEventKind::Created => events.creates.push_back(now),
            FsEventKind::Modified => events.modifies.push_back(now),
            FsEventKind::Deleted => events.deletes.push_back(now),
            FsEventKind::Renamed { .. } => events.renames.push_back(now),
            FsEventKind::MetadataChanged => {}
        }

        // Check for ransomware extension
        let has_ransomware_ext = check_ransomware_extension(&event.path);
        if has_ransomware_ext {
            events.extension_changes.push_back(now);
        }

        // Calculate threat score
        let mut score: u32 = 0;
        let mut reasons = Vec::new();

        // Mass deletion detection
        let delete_rate = events.deletes.len();
        if delete_rate > MASS_DELETE_THRESHOLD {
            score += 40;
            reasons.push(format!("Mass deletion: {} files in {}s", delete_rate, RATE_WINDOW_SECS));
        } else if delete_rate > MASS_DELETE_THRESHOLD / 2 {
            score += 15;
            reasons.push(format!("Elevated deletion rate: {} files", delete_rate));
        }

        // Mass modification detection
        let modify_rate = events.modifies.len();
        if modify_rate > MASS_MODIFY_THRESHOLD {
            score += 30;
            reasons.push(format!("Mass modification: {} files in {}s", modify_rate, RATE_WINDOW_SECS));
        } else if modify_rate > MASS_MODIFY_THRESHOLD / 2 {
            score += 10;
            reasons.push(format!("Elevated modification rate: {} files", modify_rate));
        }

        // Ransomware extension detection
        let ext_changes = events.extension_changes.len();
        if ext_changes > 5 {
            score += 50;
            reasons.push(format!("Ransomware extensions detected: {} files", ext_changes));
        } else if has_ransomware_ext {
            score += 25;
            reasons.push("Suspicious file extension detected".to_string());
        }

        // Rapid rename detection (potential encryption renaming)
        let rename_rate = events.renames.len();
        if rename_rate > 30 {
            score += 20;
            reasons.push(format!("Rapid renames: {} in {}s", rename_rate, RATE_WINDOW_SECS));
        }

        // Cap at 100
        score = score.min(100);

        // Determine threat level
        let level = ThreatLevel::from_score(score);

        // Update current level
        *self.current_level.lock().unwrap() = level.clone();

        if score > 50 {
            warn!("Threat score {}: {:?}", score, reasons);
        }

        ThreatAssessment {
            score,
            level,
            reasons,
            timestamp: event.timestamp,
        }
    }

    /// Get the current aggregate threat level
    pub fn current_threat_level(&self) -> ThreatLevel {
        self.current_level.lock().unwrap().clone()
    }
}

/// Check if a file path has a known ransomware extension
fn check_ransomware_extension(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = format!(".{}", ext.to_string_lossy().to_lowercase());
        RANSOMWARE_EXTENSIONS.contains(&ext_str.as_str())
    } else {
        false
    }
}

impl Default for ThreatDetector {
    fn default() -> Self {
        Self::new()
    }
}
