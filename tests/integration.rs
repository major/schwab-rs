//! Live integration tests that hit the real Schwab API.
//!
//! Gated behind the `test_online` feature flag.  Run with:
//!
//! ```bash
//! cargo test --features test_online
//! ```
//!
//! Required environment variables:
//!
//! - `SCHWAB_BEARER_TOKEN` - a valid OAuth2 bearer token

#![cfg(feature = "test_online")]

use schwab::{Client, Config};

fn live_client() -> Client {
    let token =
        std::env::var("SCHWAB_BEARER_TOKEN").expect("SCHWAB_BEARER_TOKEN must be set for online tests");
    Client::new(Config::new().bearer_token(token))
}

#[tokio::test]
async fn get_quotes_returns_equity_data() {
    let client = live_client();
    let quotes = client.get_quotes(&["AAPL"]).await.unwrap();
    assert!(
        !quotes.is_empty(),
        "expected at least one quote for AAPL"
    );
}

#[tokio::test]
async fn get_account_numbers_succeeds() {
    let client = live_client();
    let accounts = client.get_account_numbers().await.unwrap();
    assert!(
        !accounts.is_empty(),
        "expected at least one account number"
    );
}
