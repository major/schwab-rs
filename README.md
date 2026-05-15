# schwab-rs

Rust client library for the [Schwab API](https://developer.schwab.com/).

Wraps the Schwab Market Data and Trader REST APIs with typed methods and models so callers don't need to build URLs or parse raw JSON.

> [!IMPORTANT]
> `schwab-rs` is an unofficial project. It is not affiliated with, endorsed by, or sponsored by Charles Schwab & Co., Inc., Schwab brokerage services, or thinkorswim.

## Features

- **Market Data** - quotes, option chains, expiration chains, instruments, market hours, movers, price history
- **Trader** - accounts, orders (place/replace/cancel/preview), transactions, user preferences
- **Streaming** - WebSocket session engine for level-one equities, options, futures, futures options, and forex with broadcast events and automatic reconnect
- **OAuth2 auth** - PKCE authorization code flow, file-backed token storage, automatic refresh via `Provider`
- **Async** - built on `tokio` and `reqwest` with `rustls` for TLS

## Quick start

Add `schwab` from crates.io:

```toml
[dependencies]
schwab = "0.1"
```

```rust
use schwab::{Client, Config, QuoteOptions};

#[tokio::main]
async fn main() -> schwab::Result<()> {
    let config = Config::new()
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

## Authentication

Schwab requires OAuth2 with a browser approval step. For local development, set the app credentials in environment variables and run the auth example:

```bash
SCHWAB_CLIENT_ID='your-app-key' \
SCHWAB_CLIENT_SECRET='your-app-secret' \
SCHWAB_CALLBACK_URL='https://127.0.0.1:8182/callback' \
SCHWAB_TOKEN_PATH='schwab-token.json' \
cargo run --example auth
```

The auth example writes a token file that `Provider::from_token_file` can refresh and turn into a ready-to-use `Client`. See [`docs/auth.md`](docs/auth.md), [`examples/auth.rs`](examples/auth.rs), and [`examples/quotes.rs`](examples/quotes.rs) for the full flow.

Do not commit Schwab client secrets, authorization codes, access tokens, refresh tokens, token files, or account data. Prefer environment variables or a secret manager for credentials, and see [`SECURITY.md`](SECURITY.md) for reporting and token-handling guidance.

## Streaming

Streaming support is built around `StreamingSession`, which owns a background WebSocket task and broadcasts typed `StreamEvent` values to any number of receivers. The session supports level-one equities, options, futures, futures options, and forex subscriptions. It sends LOGOUT on `disconnect()`, records active subscriptions, and replays them after reconnecting.

The streaming protocol parser accepts command response IDs as either JSON strings or numbers and maps level-one data messages into typed equity, option, futures, futures option, and forex payloads.

WebSocket transport failures surface as `Error::WebSocket`, while HTTP response bodies remain redacted in debug output.

Reconnect behavior uses 10 attempts with exponential backoff starting at 1 second, doubling to a 30 second cap, plus 0-500ms jitter. A LOGIN_DENIED response with code 3 stops reconnecting so callers can create a new session with fresh credentials.

Note: v1 does not refresh the bearer token after reconnect. If the token expires during a long-running session, create a new session with a fresh token.

```rust,no_run
use schwab::{Client, Config, EquityField, StreamEvent, StreamData};

# async fn example() -> schwab::Result<()> {
let client = Client::new(Config::new().bearer_token("your-token"));
let mut session = client.stream().await?;
let mut rx = session.subscribe();

session.subscribe_equities(
    ["AAPL", "MSFT"],
    [EquityField::LastPrice, EquityField::BidPrice, EquityField::AskPrice],
).await?;

while let Ok(event) = rx.recv().await {
    match event {
        StreamEvent::Data(StreamData::LevelOneEquities(updates)) => {
            for update in updates {
                println!("{:?}", update);
            }
        }
        StreamEvent::Heartbeat(ts) => println!("heartbeat {ts}"),
        StreamEvent::Disconnected { .. } => break,
        _ => {}
    }
}
session.disconnect().await?;
# Ok(())
# }
```

## Feature flags

| Feature | Default | Purpose |
|---|---:|---|
| `decimal` | No | Enables `rust_decimal` support for models where decimal precision is preferable to floating-point values. |
| `test_online` | No | Enables live integration tests that call the Schwab API. Use only with explicit credentials and never in untrusted CI. |

Enable optional features with Cargo:

```bash
cargo test --features decimal
```

Run live tests only when you intentionally want network access:

```bash
cargo test --features test_online
```

## API stability

`schwab-rs` is pre-1.0. Public APIs may change while the crate tracks Schwab API behavior and fills out coverage for Market Data and Trader endpoints. Pin an exact crate version for production use.

## Minimum supported Rust version

This crate requires Rust **1.95** or later and uses Edition 2024.

## License

Apache-2.0. See [LICENSE](LICENSE) for details.
