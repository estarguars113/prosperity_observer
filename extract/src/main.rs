//! Binary entry point for the Prosperity Lakehouse.

use anyhow::{Context, Result};
use tracing::{info, instrument};

use prosperity_lakehouse::{Indicator, IndicatorFetcher, LakehouseConfig, ProsperityLakehouse};

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("prosperity_lakehouse=info,deltalake=warn")
        .with_target(false)
        .init();

    info!("Starting Prosperity Lakehouse");

    // Load configuration
    let config = LakehouseConfig::default();

    // Initialize lakehouse
    let lakehouse = ProsperityLakehouse::new(config)
        .await
        .context("Failed to initialize lakehouse")?;

    // Load indicators from external JSON configuration file
    let indicators = Indicator::load_all()
        .await
        .context("Failed to load indicators from configuration file")?;

    // Fetch all indicators with retry logic, automatically skip failed endpoints
    let fetcher = IndicatorFetcher::default();
    let successful_indicators = fetcher.fetch_all_indicators(indicators).await;

    info!(
        count = successful_indicators.len(),
        "Processing successfully fetched indicators"
    );

    // Convert to Arrow RecordBatch and write to Delta table
    let record_batch = ProsperityLakehouse::indicators_to_record_batch(&successful_indicators)
        .context("Failed to convert indicators to record batch")?;

    lakehouse
        .write_batch("transactions", record_batch)
        .await
        .context("Failed to write data to Delta lakehouse")?;

    info!("All operations completed successfully");
    info!("Delta files stored locally in ./lakehouse/transactions/");

    Ok(())
}
