# schwab-rs

Rust client library for the [Schwab API](https://developer.schwab.com/).

Wraps the Schwab Market Data and Trader REST APIs with typed methods and models so callers don't need to build URLs or parse raw JSON.

> [!IMPORTANT]
> `schwab-rs` is an unofficial project. It is not affiliated with, endorsed by, or sponsored by Charles Schwab & Co., Inc., Schwab brokerage services, or thinkorswim.

## Features

- **Market Data** - quotes, option chains, expiration chains, instruments, market hours, movers, price history
- **Trader** - accounts, orders (place/replace/cancel/preview), transactions, user preferences
- **Order builder** - typed equity helpers, single-leg option helpers, OCO, and first-triggers-second order composition
- **Typed order statuses** - known lifecycle states such as `WORKING`, `FILLED`, `CANCELED`, and `REJECTED` deserialize to typed variants, with an `Unknown` fallback for future Schwab values
- **Streaming** - WebSocket session engine for account activity, level-one equities, options, futures, futures options, forex, chart equity, chart futures, screener equity, and screener option with broadcast events and automatic reconnect
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

## Order builder

`OrderBuilder` creates serializable order payloads for `place_order`, `replace_order`, and `preview_order`. The common equity constructors choose the buy/sell instruction for you, and the option constructors choose buy-to-open, sell-to-open, buy-to-close, or sell-to-close for single-leg option orders. Lower-level `equity_*` and `option_*` constructors remain available when you need to pass an explicit instruction. Each public helper's rustdoc documents its arguments, default fields, serialized payload effects, and an example so downstream CLIs can generate command help from the API docs.

```rust,no_run
use schwab::{Client, Config, Duration, OrderBuilder};

# async fn example() -> schwab::Result<()> {
let client = Client::new(Config::new().bearer_token("your-token"));
let quantity = "10".parse().unwrap();
let price = "150.00".parse().unwrap();

let order = OrderBuilder::limit_buy("AAPL", quantity, price)
    .duration(Duration::GoodTillCancel);

let response = client.place_order("account-hash", &order).await?;
println!("order id: {:?}", response.order_id);
# Ok(())
# }
```

Single-leg option helpers use the Schwab option symbol you pass and set `assetType` to `OPTION`:

```rust
use schwab::{Number, OrderBuilder};

let quantity: Number = "1".parse().unwrap();
let price: Number = "2.50".parse().unwrap();

let open = OrderBuilder::option_buy_to_open_market("AAPL  260116C00150000", quantity);
let close = OrderBuilder::option_sell_to_close_limit("AAPL  260116C00150000", quantity, price);
```

Compose orders before submission when the Schwab payload needs nested strategies. Use `one_cancels_other` for an OCO exit order, or `first_triggers_second` when the second order should stay pending until the first fills:

```rust
use schwab::{Number, OrderBuilder};

let quantity: Number = "1".parse().unwrap();
let limit_price: Number = "140.00".parse().unwrap();
let stop_price: Number = "120.00".parse().unwrap();

let oco = OrderBuilder::one_cancels_other(
    OrderBuilder::limit_sell("AAPL", quantity, limit_price),
    OrderBuilder::stop_sell("AAPL", quantity, stop_price),
);

let buy_with_stop_loss = OrderBuilder::first_triggers_second(
    OrderBuilder::market_buy("AAPL", quantity),
    OrderBuilder::stop_sell("AAPL", quantity, stop_price),
);

let bracket = OrderBuilder::first_triggers_second(
    OrderBuilder::market_buy("AAPL", quantity),
    OrderBuilder::one_cancels_other(
        OrderBuilder::limit_sell("AAPL", quantity, limit_price),
        OrderBuilder::stop_sell("AAPL", quantity, stop_price),
    ),
);
```

## Streaming

Streaming support is built around `StreamingSession`, which owns a background WebSocket task and broadcasts typed `StreamEvent` values to any number of receivers. The session supports account activity, level-one equities, options, futures, futures options, forex, chart equity, chart futures, screener equity, and screener option subscriptions. All subscription methods share the same input trimming, field-index serialization, validation, active-subscription recording, and SUBS command delivery path. It sends LOGOUT on `disconnect()`, records active subscriptions, and replays them after reconnecting.

The streaming protocol parser accepts command response IDs as either JSON strings or numbers and maps data messages into typed account activity, level-one, chart, and screener payloads.

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
    &["AAPL", "MSFT"],
    &[EquityField::LastPrice, EquityField::BidPrice, EquityField::AskPrice],
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
