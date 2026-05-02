//! Error types for the Prosperity Lakehouse.

/// Errors that can occur during lakehouse operations.
#[derive(Debug, thiserror::Error)]
pub enum LakehouseError {
    /// Delta Lake operation failure.
    #[error("Delta operation failed: {0}")]
    DeltaError(#[from] deltalake::DeltaTableError),

    /// Arrow processing error.
    #[error("Arrow processing error: {0}")]
    ArrowError(#[from] deltalake::arrow::error::ArrowError),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// IO operation failure.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
