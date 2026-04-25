//! Indicator fetching module with retry logic and graceful failure handling

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tokio_retry::{Retry, strategy::FixedInterval};
use tracing::{debug, error, info, instrument};

/// World Bank Indicator definition.
#[derive(Debug, Clone, Deserialize)]
pub struct Indicator {
    /// Unique identifier for the indicator.
    pub id: String,
    /// Human-readable name of the indicator.
    pub name: String,
    /// URL endpoint to fetch the indicator data.
    pub url: String,
    /// Category the indicator belongs to.
    pub category: String,
}

impl Indicator {
    /// Load all indicators from external JSON configuration file
    #[instrument]
    pub async fn load_all() -> Result<Vec<Self>> {
        let content = tokio::fs::read_to_string("indicators.json")
            .await
            .context("Failed to read indicators.json file")?;

        let indicators: Vec<Indicator> = serde_json::from_str(&content)
            .context("Failed to parse indicators JSON")?;

        info!("Loaded {} indicators from configuration file", indicators.len());

        Ok(indicators)
    }
}

/// A single flattened indicator record extracted from World Bank API response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndicatorRecord {
    /// Indicator code (e.g., "SP.POP.TOTL").
    pub indicator_id: String,
    /// Human-readable indicator name.
    pub indicator_name: String,
    /// ISO3 country code (e.g., "AFE").
    pub country_id: String,
    /// Human-readable country name.
    pub country_name: String,
    /// Year of observation.
    pub year: String,
    /// Numeric value (null if missing).
    pub value: Option<f64>,
}

/// Flatten fetched indicator responses into individual records.
pub fn flatten_records(
    fetched: &[(Indicator, serde_json::Value)],
) -> Vec<IndicatorRecord> {
    let mut records = Vec::new();

    for (_indicator, response) in fetched {
        let data_array = response
            .as_array()
            .and_then(|arr| arr.get(1))
            .and_then(|v| v.as_array());

        if let Some(items) = data_array {
            for item in items {
                let Some(indicator_id) = item
                    .get("indicator")
                    .and_then(|i| i.get("id"))
                    .and_then(|v| v.as_str())
                else {
                    continue;
                };
                let indicator_name = item
                    .get("indicator")
                    .and_then(|i| i.get("value"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let country_id = item
                    .get("countryiso3code")
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        item.get("country")
                            .and_then(|c| c.get("id"))
                            .and_then(|v| v.as_str())
                    })
                    .unwrap_or("");

                let country_name = item
                    .get("country")
                    .and_then(|c| c.get("value"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let year = item
                    .get("date")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let value = item.get("value").and_then(|v| {
                    if v.is_null() {
                        None
                    } else {
                        v.as_f64()
                    }
                });

                records.push(IndicatorRecord {
                    indicator_id: indicator_id.to_string(),
                    indicator_name: indicator_name.to_string(),
                    country_id: country_id.to_string(),
                    country_name: country_name.to_string(),
                    year: year.to_string(),
                    value,
                });
            }
        }
    }

    records
}

/// Outcome of fetching all indicators.
#[derive(Debug)]
pub struct FetchOutcome {
    /// Successfully fetched indicator data.
    pub successful: Vec<(Indicator, serde_json::Value)>,
    /// Failed indicators with their error messages.
    pub failed: Vec<(Indicator, String)>,
}

/// Result of a single indicator fetch attempt.
#[derive(Debug)]
pub enum FetchResult {
    /// Indicator data was successfully retrieved.
    Success(serde_json::Value),
    /// Indicator fetch failed after all retries.
    Failure(String),
}

/// Indicator fetcher with retry policy
#[derive(Debug, Clone)]
pub struct IndicatorFetcher {
    client: Client,
    max_retries: usize,
    retry_delay: Duration,
}

impl Default for IndicatorFetcher {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("ProsperityLakehouse/0.0.1")
                .build()
                .expect("Failed to create HTTP client"),
            max_retries: 2,
            retry_delay: Duration::from_secs(1),
        }
    }
}

impl IndicatorFetcher {
    /// Fetch single indicator with retry logic.
    #[tracing::instrument(skip(self))]
    pub async fn fetch_indicator(&self, indicator: &Indicator) -> Result<FetchResult> {
        let retry_strategy = FixedInterval::new(self.retry_delay)
            .take(self.max_retries);

        let result: Result<serde_json::Value> = Retry::spawn(retry_strategy, || async {
            debug!(indicator_id = indicator.id, "Fetching indicator");

            let response = self.client
                .get(&indicator.url)
                .send()
                .await
                .and_then(|res| res.error_for_status())
                .context("HTTP request failed")?;

            let data: serde_json::Value = response.json()
                .await
                .context("Failed to parse JSON response")?;

            // World Bank API returns HTTP 200 with an error body for invalid indicators.
            // Detect this: [{"message":[{"id":"120","key":"Invalid value",...}]}]
            if let Some(first) = data.as_array().and_then(|arr| arr.first()) {
                if first.get("message").is_some() {
                    let error_detail = serde_json::to_string(&data).unwrap_or_default();
                    return Err(anyhow::anyhow!("World Bank API error: {}", error_detail));
                }
            }

            Ok(data)
        }).await;

        match result {
            Ok(data) => {
                info!(indicator_id = indicator.id, "Successfully fetched indicator");
                Ok(FetchResult::Success(data))
            }
            Err(e) => {
                let msg = format!("{}", e);
                error!(indicator_id = indicator.id, error = %e, "Indicator failed after {} retries", self.max_retries);
                Ok(FetchResult::Failure(msg))
            }
        }
    }

    /// Fetch all indicators concurrently, collecting both successes and failures.
    #[tracing::instrument(skip(self))]
    pub async fn fetch_all_indicators(&self, indicators: Vec<Indicator>) -> FetchOutcome {
        let total = indicators.len();
        info!("Starting fetch for {} indicators", total);

        let futures = indicators.into_iter().map(|indicator| async move {
            match self.fetch_indicator(&indicator).await {
                Ok(FetchResult::Success(data)) => Ok((indicator, data)),
                Ok(FetchResult::Failure(msg)) => Err((indicator, msg)),
                Err(e) => Err((indicator, format!("Unexpected error: {}", e))),
            }
        });

        let results = futures::future::join_all(futures).await;

        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for result in results {
            match result {
                Ok(pair) => successful.push(pair),
                Err(pair) => failed.push(pair),
            }
        }

        let success_count = successful.len();
        let fail_count = failed.len();

        if fail_count == 0 {
            info!("✅ All {} indicators fetched successfully", success_count);
        } else {
            info!("⚠️  Completed: {} successful / {} failed / {} total indicators", success_count, fail_count, total);
        }

        FetchOutcome { successful, failed }
    }
}
