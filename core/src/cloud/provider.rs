//! Cloud provider abstraction layer.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Cloud provider trait — all cloud providers implement this interface
#[async_trait]
pub trait CloudProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Upload encrypted data to cloud storage
    async fn upload(&self, remote_path: &str, data: &[u8]) -> anyhow::Result<()>;

    /// Download data from cloud storage
    async fn download(&self, remote_path: &str) -> anyhow::Result<Vec<u8>>;

    /// Check if a file exists on cloud storage
    async fn exists(&self, remote_path: &str) -> anyhow::Result<bool>;

    /// Delete a file from cloud storage
    async fn delete(&self, remote_path: &str) -> anyhow::Result<()>;

    /// List files at a remote path
    async fn list(&self, remote_path: &str) -> anyhow::Result<Vec<String>>;

    /// Get available storage space (if supported)
    async fn available_space(&self) -> anyhow::Result<Option<u64>>;
}

/// Cloud provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    pub provider_type: CloudProviderType,
    pub name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
}

/// Supported cloud provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CloudProviderType {
    GoogleDrive,
    OneDrive,
    Dropbox,
    S3,
    BackblazeB2,
    S3Compatible,
}

/// Stub Google Drive provider (full implementation would use OAuth2 + Google APIs)
pub struct GoogleDriveProvider {
    _config: serde_json::Value,
}

impl GoogleDriveProvider {
    pub fn new(config: serde_json::Value) -> Self {
        Self { _config: config }
    }
}

#[async_trait]
impl CloudProvider for GoogleDriveProvider {
    fn name(&self) -> &str { "Google Drive" }

    async fn upload(&self, _remote_path: &str, _data: &[u8]) -> anyhow::Result<()> {
        // TODO: Implement Google Drive upload via OAuth2 + REST API
        Err(anyhow::anyhow!("Google Drive upload not yet implemented"))
    }

    async fn download(&self, _remote_path: &str) -> anyhow::Result<Vec<u8>> {
        Err(anyhow::anyhow!("Google Drive download not yet implemented"))
    }

    async fn exists(&self, _remote_path: &str) -> anyhow::Result<bool> {
        Ok(false)
    }

    async fn delete(&self, _remote_path: &str) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Google Drive delete not yet implemented"))
    }

    async fn list(&self, _remote_path: &str) -> anyhow::Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn available_space(&self) -> anyhow::Result<Option<u64>> {
        Ok(None)
    }
}
