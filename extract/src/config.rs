//! Configuration for the Prosperity Lakehouse.

use std::path::PathBuf;

/// Storage backend mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageMode {
    /// Local filesystem storage.
    Local,
    /// AWS S3 storage.
    Aws,
    /// Azure Blob Storage.
    Azure,
}

/// Lakehouse configuration loaded from environment variables and CLI.
#[derive(Debug, Clone)]
pub struct LakehouseConfig {
    /// Storage backend mode.
    pub mode: StorageMode,
    /// Local storage path (used for Local mode).
    pub local_path: PathBuf,
    /// Table/directory name within the storage backend.
    pub table_name: String,
    /// AWS S3 bucket name (used for Aws mode).
    pub aws_bucket: Option<String>,
    /// Azure container name (used for Azure mode).
    pub azure_container: Option<String>,
    /// Azure storage account name (used for Azure mode).
    pub azure_account: Option<String>,
}

impl LakehouseConfig {
    /// Build configuration from storage mode and environment variables.
    pub fn from_mode(mode: StorageMode) -> Self {
        Self {
            mode,
            local_path: std::env::var("LAKEHOUSE_LOCAL_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./lakehouse")),
            table_name: std::env::var("LAKEHOUSE_TABLE_NAME")
                .unwrap_or_else(|_| "transactions".to_string()),
            aws_bucket: std::env::var("AWS_S3_BUCKET").ok(),
            azure_container: std::env::var("AZURE_CONTAINER_NAME").ok(),
            azure_account: std::env::var("AZURE_STORAGE_ACCOUNT_NAME").ok(),
        }
    }
}

impl Default for LakehouseConfig {
    fn default() -> Self {
        Self::from_mode(StorageMode::Local)
    }
}
