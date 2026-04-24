//! Core lakehouse logic for managing Delta tables.

use std::sync::Arc;

use anyhow::{Context, Result};
use deltalake::arrow::array::StringArray;
use deltalake::arrow::datatypes::{DataType as ArrowDataType, Field as ArrowField, Schema as ArrowSchema};
use deltalake::arrow::record_batch::RecordBatch;
use deltalake::kernel::{DataType as DeltaDataType, PrimitiveType, StructField};
use deltalake::protocol::SaveMode;
use deltalake::{DeltaOps, DeltaTable};
use tokio::fs;
use tracing::{debug, info, instrument};

use crate::config::LakehouseConfig;
use crate::error::LakehouseError;
use crate::indicators::Indicator;

/// High-level lakehouse client for reading and writing Delta tables.
#[derive(Debug, Clone)]
pub struct ProsperityLakehouse {
    config: LakehouseConfig,
}

impl ProsperityLakehouse {
    /// Create a new lakehouse instance.
    #[instrument(skip(config))]
    pub async fn new(config: LakehouseConfig) -> Result<Self> {
        fs::create_dir_all(&config.storage_path)
            .await
            .context("Failed to create lakehouse storage directory")?;

        info!(path = ?config.storage_path, "Lakehouse initialized");

        Ok(Self { config })
    }

    /// Get or create a Delta table at the specified path.
    ///
    /// Uses `SaveMode::Ignore` so the table is created if it doesn't exist,
    /// or loaded as-is if it already exists.
    #[instrument(skip(self))]
    pub async fn get_table(&self, table_name: &str) -> Result<DeltaTable, LakehouseError> {
        let table_path = self.config.storage_path.join(table_name);

        debug!(table_name, path = ?table_path, "Accessing Delta table");

        let table = DeltaOps::try_from_uri(table_path.to_string_lossy())
            .await?
            .create()
            .with_columns([
                StructField::new("id".to_string(), DeltaDataType::Primitive(PrimitiveType::String), false),
                StructField::new("name".to_string(), DeltaDataType::Primitive(PrimitiveType::String), false),
                StructField::new("url".to_string(), DeltaDataType::Primitive(PrimitiveType::String), false),
                StructField::new("category".to_string(), DeltaDataType::Primitive(PrimitiveType::String), false),
                StructField::new("data".to_string(), DeltaDataType::Primitive(PrimitiveType::String), true),
            ])
            .with_save_mode(SaveMode::Ignore)
            .await?;

        Ok(table)
    }

    /// Write a record batch to a Delta table.
    #[instrument(skip(self, batch))]
    pub async fn write_batch(&self, table_name: &str, batch: RecordBatch) -> Result<(), LakehouseError> {
        let table = self.get_table(table_name).await?;

        let version = DeltaOps::from(table)
            .write(vec![batch])
            .await?
            .version();

        info!(table_name, version, "Successfully wrote batch to Delta table");

        Ok(())
    }

    /// Convert fetched indicators into an Arrow [`RecordBatch`].
    pub fn indicators_to_record_batch(
        indicators: &[(Indicator, serde_json::Value)],
    ) -> Result<RecordBatch, LakehouseError> {
        let ids: Vec<&str> = indicators.iter().map(|(i, _)| i.id.as_str()).collect();
        let names: Vec<&str> = indicators.iter().map(|(i, _)| i.name.as_str()).collect();
        let urls: Vec<&str> = indicators.iter().map(|(i, _)| i.url.as_str()).collect();
        let categories: Vec<&str> = indicators.iter().map(|(i, _)| i.category.as_str()).collect();
        let data_values: Vec<Option<String>> = indicators
            .iter()
            .map(|(_, d)| serde_json::to_string(d).ok())
            .collect();

        let schema = Arc::new(ArrowSchema::new(vec![
            ArrowField::new("id", ArrowDataType::Utf8, false),
            ArrowField::new("name", ArrowDataType::Utf8, false),
            ArrowField::new("url", ArrowDataType::Utf8, false),
            ArrowField::new("category", ArrowDataType::Utf8, false),
            ArrowField::new("data", ArrowDataType::Utf8, true),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(ids)),
                Arc::new(StringArray::from(names)),
                Arc::new(StringArray::from(urls)),
                Arc::new(StringArray::from(categories)),
                Arc::new(StringArray::from(data_values)),
            ],
        )?;

        Ok(batch)
    }
}
