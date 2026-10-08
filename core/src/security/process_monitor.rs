//! Process monitoring for tracking which processes access protected data.
//! 
//! Note: Full process monitoring requires platform-specific APIs.
//! This module provides the framework; platform implementations are in the platform module.

use serde::Serialize;

/// Information about a monitored process
#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub is_suspicious: bool,
}

/// Process monitor stub (platform-specific implementation needed)
pub struct ProcessMonitor;

impl ProcessMonitor {
    pub fn new() -> Self {
        Self
    }

    /// Get information about a process by PID
    pub fn get_process_info(&self, _pid: u32) -> Option<ProcessInfo> {
        // Platform-specific implementation would go here
        None
    }
}

impl Default for ProcessMonitor {
    fn default() -> Self {
        Self::new()
    }
}
