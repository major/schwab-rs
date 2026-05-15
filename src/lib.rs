#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

//! Rust client library for the Schwab API.
//!
//! `schwab-rs` is an unofficial project. It is not affiliated with,
//! endorsed by, or sponsored by Charles Schwab & Co., Inc., Schwab brokerage
//! services, or thinkorswim.
//!
//! The OpenAPI files in `docs/` describe the Schwab HTTP API. This crate exposes
//! small client methods like `get_quotes(...)` instead of forcing callers to
//! build raw URLs and parse raw JSON.
//!
//! # Quotes example
//!
//! ```no_run
//! # async fn example() -> schwab::Result<()> {
//! use schwab::{Client, Config, QuoteOptions};
//!
//! let config = Config::new().base_url("http://127.0.0.1:8080/marketdata/v1")?;
//! let client = Client::new(config);
//!
//! let quotes = client
//!     .get_quotes_with_options(
//!         ["AAPL", "MSFT"],
//!         QuoteOptions::new().fields("quote,reference"),
//!     )
//!     .await?;
//!
//! if let Some(quote) = quotes.get("AAPL") {
//!     println!("AAPL quote: {:?}", quote);
//! }
//! # Ok(())
//! # }
//! ```

pub mod auth;

mod client;
mod config;
mod error;
mod market_data_api;
mod models;
mod options;
mod order_builder;
mod query;
#[allow(hidden_glob_reexports)]
mod streaming;
mod streaming_api;
mod trader_api;

pub use client::Client;
pub use config::Config;
pub use error::{Error, Result};
pub use models::*;
pub use options::{
    MoverOptions, OptionChainOptions, OrderListOptions, PriceHistoryOptions, QuoteOptions,
    TransactionListOptions,
};
pub use order_builder::OrderBuilder;
pub use streaming::StreamingSession;

#[cfg(test)]
mod test_support;
