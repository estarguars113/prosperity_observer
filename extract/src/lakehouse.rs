//! Core lakehouse logic for managing partitioned Delta tables.

use std::sync::Arc;

use anyhow::{Context, Result};
use deltalake::arrow::array::{Float64Array, StringArray};
use deltalake::arrow::datatypes::{DataType as ArrowDataType, Field as ArrowField, Schema as ArrowSchema};
use deltalake::arrow::record_batch::RecordBatch;
use deltalake::kernel::{DataType as DeltaDataType, PrimitiveType, StructField};
use deltalake::protocol::SaveMode;
use deltalake::{open_table, DeltaOps, DeltaTable};
use tokio::fs;
use tracing::{info, instrument};

use crate::config::{LakehouseConfig, StorageMode};
use crate::error::LakehouseError;
use crate::indicators::IndicatorRecord;

/// High-level lakehouse client for reading and writing partitioned Delta tables.
#[derive(Debug, Clone)]
pub struct ProsperityLakehouse {
    config: LakehouseConfig,
}

impl ProsperityLakehouse {
    /// Create a new lakehouse instance.
    #[instrument(skip(config))]
    pub async fn new(config: LakehouseConfig) -> Result<Self> {
        if config.mode == StorageMode::Local {
            fs::create_dir_all(&config.local_path)
                .await
                .context("Failed to create lakehouse storage directory")?;
        }

        info!(mode = ?config.mode, "Lakehouse initialized");

        Ok(Self { config })
    }

    /// Build the Delta table URI based on the storage mode.
    fn table_uri(&self) -> String {
        match self.config.mode {
            StorageMode::Local => self
                .config
                .local_path
                .join(&self.config.table_name)
                .to_string_lossy()
                .to_string(),
            StorageMode::Aws => format!(
                "s3://{}/{}",
                self.config
                    .aws_bucket
                    .as_ref()
                    .expect("AWS_S3_BUCKET must be set for AWS mode"),
                self.config.table_name
            ),
            StorageMode::Azure => format!(
                "az://{}/{}",
                self.config
                    .azure_container
                    .as_ref()
                    .expect("AZURE_CONTAINER_NAME must be set for Azure mode"),
                self.config.table_name
            ),
        }
    }

    /// Get or create a Delta table with partitioning by indicator_id/country_id/year.
    ///
    /// If the table already exists it is loaded as-is; otherwise a new partitioned
    /// table is created.
    #[instrument(skip(self))]
    pub async fn get_or_create_table(&self) -> Result<DeltaTable, LakehouseError> {
        let uri = self.table_uri();

        info!(uri = %uri, "Accessing Delta table");

        // Try to load an existing table first.
        match open_table(&uri).await {
            Ok(table) => {
                info!("Loaded existing Delta table");
                Ok(table)
            }
            Err(_) => {
                info!("Creating new partitioned Delta table");
                let table = DeltaOps::try_from_uri(uri)
                    .await?
                    .create()
                    .with_columns([
                        StructField::new(
                            "indicator_id".to_string(),
                            DeltaDataType::Primitive(PrimitiveType::String),
                            false,
                        ),
                        StructField::new(
                            "indicator_name".to_string(),
                            DeltaDataType::Primitive(PrimitiveType::String),
                            false,
                        ),
                        StructField::new(
                            "country_id".to_string(),
                            DeltaDataType::Primitive(PrimitiveType::String),
                            false,
                        ),
                        StructField::new(
                            "country_name".to_string(),
                            DeltaDataType::Primitive(PrimitiveType::String),
                            false,
                        ),
                        StructField::new(
                            "year".to_string(),
                            DeltaDataType::Primitive(PrimitiveType::String),
                            false,
                        ),
                        StructField::new(
                            "value".to_string(),
                            DeltaDataType::Primitive(PrimitiveType::Double),
                            true,
                        ),
                    ])
                    .with_partition_columns(["indicator_id", "country_id", "year"])
                    .with_save_mode(SaveMode::Ignore)
                    .await?;

                Ok(table)
            }
        }
    }

    /// Write indicator records to the Delta table.
    #[instrument(skip(self, records))]
    pub async fn write_records(&self, records: &[IndicatorRecord]) -> Result<(), LakehouseError> {
        if records.is_empty() {
            info!("No records to write");
            return Ok(());
        }

        let batch = records_to_batch(records)?;
        let table = self.get_or_create_table().await?;

        let version = DeltaOps::from(table)
            .write(vec![batch])
            .await?
            .version();

        info!(version, "Successfully wrote records to Delta table");

        Ok(())
    }
}

/// Convert indicator records into an Arrow [`RecordBatch`].
fn records_to_batch(records: &[IndicatorRecord]) -> Result<RecordBatch, LakehouseError> {
    let indicator_ids: Vec<&str> = records.iter().map(|r| r.indicator_id.as_str()).collect();
    let indicator_names: Vec<&str> = records.iter().map(|r| r.indicator_name.as_str()).collect();
    let country_ids: Vec<&str> = records.iter().map(|r| r.country_id.as_str()).collect();
    let country_names: Vec<&str> = records.iter().map(|r| r.country_name.as_str()).collect();
    let years: Vec<&str> = records.iter().map(|r| r.year.as_str()).collect();
    let values: Vec<Option<f64>> = records.iter().map(|r| r.value).collect();

    let schema = Arc::new(ArrowSchema::new(vec![
        ArrowField::new("indicator_id", ArrowDataType::Utf8, false),
        ArrowField::new("indicator_name", ArrowDataType::Utf8, false),
        ArrowField::new("country_id", ArrowDataType::Utf8, false),
        ArrowField::new("country_name", ArrowDataType::Utf8, false),
        ArrowField::new("year", ArrowDataType::Utf8, false),
        ArrowField::new("value", ArrowDataType::Float64, true),
    ]));

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(indicator_ids)),
            Arc::new(StringArray::from(indicator_names)),
            Arc::new(StringArray::from(country_ids)),
            Arc::new(StringArray::from(country_names)),
            Arc::new(StringArray::from(years)),
            Arc::new(Float64Array::from(values)),
        ],
    )?;

    Ok(batch)
}
