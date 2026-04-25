//! Binary entry point for the Prosperity Lakehouse.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::{info, instrument};

use prosperity_lakehouse::{
    flatten_records, Indicator, IndicatorFetcher, LakehouseConfig, ProsperityLakehouse, StorageMode,
};

#[derive(Parser)]
#[command(name = "prosperity-lakehouse")]
#[command(about = "Fetch prosperity indicators and save them to a partitioned lakehouse")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Save files locally partitioned by indicator/country/year
    Local,
    /// Save files to AWS S3 partitioned by indicator/country/year
    Aws,
    /// Save files to Azure Blob Storage partitioned by indicator/country/year
    Azure,
}

impl From<Commands> for StorageMode {
    fn from(cmd: Commands) -> Self {
        match cmd {
            Commands::Local => StorageMode::Local,
            Commands::Aws => StorageMode::Aws,
            Commands::Azure => StorageMode::Azure,
        }
    }
}

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("prosperity_lakehouse=info")
        .with_target(false)
        .init();

    info!("Starting Prosperity Lakehouse");

    let cli = Cli::parse();
    let config = LakehouseConfig::from_mode(cli.command.into());

    info!(mode = ?config.mode, "Storage mode selected");

    let lakehouse = ProsperityLakehouse::new(config)
        .await
        .context("Failed to initialize lakehouse")?;

    let indicators = Indicator::load_all()
        .await
        .context("Failed to load indicators from configuration file")?;

    let fetcher = IndicatorFetcher::default();
    let outcome = fetcher.fetch_all_indicators(indicators).await;

    // Print failure summary if any indicators failed
    if !outcome.failed.is_empty() {
        eprintln!("\n❌ FAILED INDICATORS:");
        eprintln!("{:<30} {:<25} {}", "ID", "Name", "Error");
        eprintln!("{}", "-".repeat(120));
        for (indicator, error) in &outcome.failed {
            eprintln!(
                "{:<30} {:<25} {}",
                indicator.id,
                indicator.name,
                error.chars().take(60).collect::<String>()
            );
        }
        eprintln!();

        return Err(anyhow::anyhow!(
            "{} of {} indicators failed to fetch — no data written to lakehouse",
            outcome.failed.len(),
            outcome.successful.len() + outcome.failed.len()
        ));
    }

    info!(
        count = outcome.successful.len(),
        "Processing successfully fetched indicators"
    );

    let records = flatten_records(&outcome.successful);

    info!(record_count = records.len(), "Flattened indicator records");

    lakehouse
        .write_records(&records)
        .await
        .context("Failed to write data to lakehouse")?;

    info!("All operations completed successfully");

    Ok(())
}
