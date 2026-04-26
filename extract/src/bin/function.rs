//! Azure Functions custom handler entry-point.
//!
//! This binary runs an HTTP server on the port dictated by the
//! `FUNCTIONS_CUSTOMHANDLER_PORT` environment variable.  Azure Functions
//! forwards timer-trigger invocations to it as HTTP POST requests.

use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::{
    body::Bytes,
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use serde_json::Value;
use tracing::{error, info};

use prosperity_lakehouse::{
    flatten_records, Indicator, IndicatorFetcher, LakehouseConfig, ProsperityLakehouse, StorageMode,
};

#[derive(Debug, Serialize)]
struct FunctionResponse {
    outputs: Value,
    logs: Vec<String>,
    return_value: String,
}

/// Shared application state (empty for now, but extensible).
#[derive(Clone)]
struct AppState;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("prosperity_lakehouse=info")
        .with_target(false)
        .init();

    let port: u16 = std::env::var("FUNCTIONS_CUSTOMHANDLER_PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .context("Invalid FUNCTIONS_CUSTOMHANDLER_PORT")?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/ProsperityExtract", post(timer_handler))
        .with_state(AppState);

    info!(%addr, "Azure custom handler starting");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .await
        .context("axum server error")?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn timer_handler(
    State(_state): State<AppState>,
    body: Bytes,
) -> Json<FunctionResponse> {
    info!(body = %String::from_utf8_lossy(&body), "Received timer invocation");

    let result = run_extraction().await;

    let return_value = match result {
        Ok(_) => {
            info!("Extraction completed successfully");
            "success".to_string()
        }
        Err(ref e) => {
            error!(error = %e, "Extraction failed");
            format!("error: {}", e)
        }
    };

    Json(FunctionResponse {
        outputs: serde_json::json!({}),
        logs: vec![format!("Extraction result: {}", return_value)],
        return_value,
    })
}

async fn run_extraction() -> Result<()> {
    let config = LakehouseConfig::from_mode(StorageMode::Azure);

    let lakehouse = ProsperityLakehouse::new(config)
        .await
        .context("Failed to initialize lakehouse")?;

    let indicators = Indicator::load_all()
        .await
        .context("Failed to load indicators from configuration file")?;

    let fetcher = IndicatorFetcher::default();
    let outcome = fetcher.fetch_all_indicators(indicators).await;

    if !outcome.failed.is_empty() {
        return Err(anyhow::anyhow!(
            "{} of {} indicators failed to fetch",
            outcome.failed.len(),
            outcome.successful.len() + outcome.failed.len()
        ));
    }

    let records = flatten_records(&outcome.successful);
    lakehouse
        .write_records(&records)
        .await
        .context("Failed to write data to lakehouse")?;

    Ok(())
}
