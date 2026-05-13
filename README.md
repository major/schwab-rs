# schwab-rs

Rust client library for the [Schwab API](https://developer.schwab.com/).

Wraps the Schwab Market Data and Trader REST APIs with typed methods and models so callers don't need to build URLs or parse raw JSON.

## Features

- **Market Data** - quotes, option chains, expiration chains, instruments, market hours, movers, price history
- **Trader** - accounts, orders (place/replace/cancel/preview), transactions, user preferences
- **OAuth2 auth** - PKCE authorization code flow, file-backed token storage, automatic refresh via `Provider`
- **Async** - built on `tokio` and `reqwest` with `rustls` for TLS

## Quick start

```rust
use schwab::{Client, Config, QuoteOptions};

#[tokio::main]
async fn main() -> schwab::Result<()> {
    let config = Config::new()
        .base_url("https://api.schwabapi.com/marketdata/v1")?
        .bearer_token("your-token");
    let client = Client::new(config);

    let quotes = client
        .get_quotes_with_options(
            ["AAPL", "MSFT"],
            QuoteOptions::new().fields("quote,reference"),
        )
        .await?;

    if let Some(quote) = quotes.get("AAPL") {
        println!("{quote:?}");
    }
    Ok(())
}
```

See `examples/auth.rs` for the full OAuth2 login flow.

## Minimum supported Rust version

Edition 2024, which requires Rust **1.85** or later.

## License

Apache-2.0. See [LICENSE](LICENSE) for details.
