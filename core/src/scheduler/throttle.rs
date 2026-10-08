//! Resource throttling to prevent system overload during backup operations.

use crate::config::ResourceLimits;

/// Resource throttler enforces CPU, memory, and I/O limits
pub struct ResourceThrottle {
    limits: ResourceLimits,
}

impl ResourceThrottle {
    pub fn new(limits: ResourceLimits) -> Self {
        Self { limits }
    }

    /// Get the maximum number of concurrent operations allowed
    pub fn max_concurrent_ops(&self) -> u16 {
        self.limits.max_concurrent_ops
    }

    /// Get the maximum number of background threads
    pub fn max_threads(&self) -> u16 {
        self.limits.max_threads
    }

    /// Check if backup should be paused (e.g., battery too low)
    pub fn should_pause(&self) -> bool {
        // In production: check actual battery level and power source
        false
    }

    /// Calculate an appropriate delay between operations based on CPU limits
    pub fn operation_delay_ms(&self) -> u64 {
        // Higher CPU limit = less delay
        let cpu_factor = 100 - self.limits.max_cpu_percent.min(100);
        (cpu_factor as u64) * 2 // 0-200ms delay
    }

    /// Update limits
    pub fn update_limits(&mut self, limits: ResourceLimits) {
        self.limits = limits;
    }
}
