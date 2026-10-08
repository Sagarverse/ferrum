//! Protection score calculation (0-100).

use serde::Serialize;

/// System health score breakdown
#[derive(Debug, Clone, Serialize)]
pub struct HealthScore {
    /// Overall score (0-100)
    pub overall: u32,
    /// Individual component scores
    pub components: HealthComponents,
}

/// Health scores for individual components
#[derive(Debug, Clone, Serialize)]
pub struct HealthComponents {
    pub protection: u32,
    pub replicas: u32,
    pub security: u32,
    pub storage: u32,
    pub recovery: u32,
}
