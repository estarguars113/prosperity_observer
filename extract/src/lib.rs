//! Prosperity Lakehouse
//!
//! A high-performance Delta Lake implementation for prosperity indicator data,
//! following Rust best practices.

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

pub mod config;
pub mod error;
pub mod indicators;
pub mod lakehouse;

pub use config::LakehouseConfig;
pub use error::LakehouseError;
pub use indicators::{Indicator, IndicatorFetcher};
pub use lakehouse::ProsperityLakehouse;
