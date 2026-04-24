//! Indicator fetching module with retry logic and graceful failure handling

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tokio_retry::{Retry, strategy::FixedInterval};
use tracing::{debug, error, info, warn, instrument};

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
    /// Fetch single indicator with retry logic
    /// Returns Ok(None) if indicator fails after all retries
    #[tracing::instrument(skip(self))]
    pub async fn fetch_indicator(&self, indicator: &Indicator) -> Result<Option<serde_json::Value>> {
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

            let data = response.json()
                .await
                .context("Failed to parse JSON response")?;

            Ok(data)
        }).await;

        match result {
            Ok(data) => {
                info!(indicator_id = indicator.id, "Successfully fetched indicator");
                Ok(Some(data))
            }
            Err(e) => {
                error!(indicator_id = indicator.id, error = %e, "Indicator failed after {} retries, SKIPPING", self.max_retries);
                warn!(indicator_id = indicator.id, "Continuing with remaining indicators");
                Ok(None)
            }
        }
    }

    /// Fetch all indicators concurrently, skipping failed ones
    #[tracing::instrument(skip(self))]
    pub async fn fetch_all_indicators(&self, indicators: Vec<Indicator>) -> Vec<(Indicator, serde_json::Value)> {
        let total = indicators.len();
        info!("Starting fetch for {} indicators", total);

        let futures = indicators.into_iter().map(|indicator| async move {
            match self.fetch_indicator(&indicator).await {
                Ok(Some(data)) => Some((indicator, data)),
                _ => None,
            }
        });

        let results = futures::future::join_all(futures).await;

        let successful: Vec<_> = results.into_iter().flatten().collect();

        info!("✅ Completed: {} successful / {} total indicators", successful.len(), total);

        successful
    }
}