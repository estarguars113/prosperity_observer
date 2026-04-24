//! Configuration for the Prosperity Lakehouse.

use std::path::PathBuf;

use serde::Deserialize;

/// Lakehouse configuration loaded from file or environment.
#[derive(Debug, Deserialize, Clone)]
pub struct LakehouseConfig {
    /// Local storage path for Delta tables.
    pub storage_path: PathBuf,
    /// Default table name.
    pub default_table: String,
    /// Log level configuration.
    pub log_level: String,
}

impl Default for LakehouseConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from("./lakehouse"),
            default_table: "transactions".to_string(),
            log_level: "info".to_string(),
        }
    }
}
