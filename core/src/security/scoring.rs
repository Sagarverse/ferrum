//! Threat scoring system (0-100).

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Threat level classification
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatLevel {
    /// 0-20: Normal operation
    Normal,
    /// 21-50: Elevated monitoring
    Watch,
    /// 51-75: Suspicious activity detected
    Suspicious,
    /// 76-100: Critical threat, quarantine recommended
    Critical,
}

impl ThreatLevel {
    pub fn from_score(score: u32) -> Self {
        match score {
            0..=20 => Self::Normal,
            21..=50 => Self::Watch,
            51..=75 => Self::Suspicious,
            _ => Self::Critical,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Normal => "normal",
            Self::Watch => "watch",
            Self::Suspicious => "suspicious",
            Self::Critical => "critical",
        }
    }
}

/// Complete threat assessment for an event or time period
#[derive(Debug, Clone, Serialize)]
pub struct ThreatAssessment {
    /// Threat score (0-100)
    pub score: u32,
    /// Classified threat level
    pub level: ThreatLevel,
    /// Reasons for the assessment
    pub reasons: Vec<String>,
    /// When this assessment was made
    pub timestamp: DateTime<Utc>,
}

impl ThreatAssessment {
    /// Check if this assessment requires quarantine action
    pub fn requires_quarantine(&self) -> bool {
        self.score >= 70
    }

    /// Check if this assessment should increase monitoring
    pub fn requires_monitoring(&self) -> bool {
        self.score >= 30
    }
}
