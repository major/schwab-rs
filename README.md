# schwab-rs

Rust client library and `schwab-agent` structured JSON CLI for the [Schwab API](https://developer.schwab.com/).

Wraps the Schwab Market Data and Trader REST APIs with typed methods and models so callers don't need to build URLs or parse raw JSON. The same crate also ships `schwab-agent`, an agent-oriented CLI for auth, market data, account discovery, option workflows, technical analysis, and guarded order actions.

> [!IMPORTANT]
> `schwab-rs` is an unofficial project. It is not affiliated with, endorsed by, or sponsored by Charles Schwab & Co., Inc., Schwab brokerage services, or thinkorswim.

## Features

- **Market Data** - quotes, option chains, expiration chains, instruments, market hours, movers, price history
- **Trader** - accounts, orders (place/replace/cancel/preview), transactions, user preferences
- **Order builder** - typed equity helpers, single-leg option helpers, OCO, and first-triggers-second order composition
- **Repeat orders** - convert supported historical `Order` responses into `OrderBuilder` payloads for reuse
- **Typed order statuses** - known lifecycle states such as `WORKING`, `FILLED`, `CANCELED`, and `REJECTED` deserialize to typed variants, with an `Unknown` fallback for future Schwab values
- **Streaming** - WebSocket session engine for account activity, level-one equities, options, futures, futures options, forex, chart equity, chart futures, screener equity, and screener option with broadcast events and automatic reconnect
- **OAuth2 auth** - PKCE authorization code flow, file-backed token storage, automatic refresh via `Provider`
- **schwab-agent CLI** - structured JSON command-line workflows for auth, quotes, history, accounts, orders, options, technical analysis, and multi-symbol analysis
- **Async** - built on `tokio` and `reqwest` with `rustls` for TLS

## Quick start

Add `schwab` from crates.io:

```toml
[dependencies]
schwab = "0.3"
```

The default feature set includes the bundled `schwab-agent` CLI. Library-only consumers that do not need the binary can avoid CLI-only dependencies with:

```toml
[dependencies]
schwab = { version = "0.3", default-features = false }
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

## schwab-agent CLI

Install the bundled JSON CLI from the published crate:

```bash
cargo install schwab --bin schwab-agent --locked
```

`schwab-agent` prints raw JSON payloads on success and structured JSON errors with stable `code`, `message`, `category`, `retryable`, and `hint` fields on failure. The `completions` command is the only raw stdout exception because shells need a completion script; write failures emit a short stderr diagnostic and exit non-zero. Credentials come from `SCHWAB_CLIENT_ID`, `SCHWAB_CLIENT_SECRET`, and optional `SCHWAB_CALLBACK_URL`, or from `~/.config/schwab-agent/config.json`. The token path can be overridden with a non-empty `SCHWAB_TOKEN_PATH`; the default remains `$XDG_CONFIG_HOME/schwab-agent-rs/token.json` for compatibility with existing agent installs, falling back to the platform config directory when `XDG_CONFIG_HOME` is unset.

For the full LLM-facing command contract, workflows, and safety rules, see [`SKILL.md`](SKILL.md), which points to the detailed binary guide under `src/bin/schwab-agent/`.

```bash
schwab-agent auth login-url
schwab-agent auth exchange --redirect-url "CALLBACK_URL_WITH_CODE"
schwab-agent auth refresh

schwab-agent market quote AAPL MSFT --fields sym,last,pct,vol
schwab-agent market quote AAPL --all-fields
schwab-agent market history SPY --from 2026-01-01 --to 2026-01-31 --fields ts,close,vol

schwab-agent account --positions
schwab-agent order get --symbol AAPL
schwab-agent option expirations AAPL
schwab-agent option chain AAPL --type call --dte 30 --fields strike,delta,bid,ask,volume,oi
schwab-agent option screen AAPL --type call --min-bid 1.00 --max-spread-pct 10
schwab-agent ta dashboard SPY
schwab-agent analyze AAPL MSFT --points 5
schwab-agent completions bash > schwab-agent.bash
```

Command-specific `--help` output includes copyable examples for the main market, option, technical analysis, and analyze workflows so agents can discover usage without leaving the terminal. `market history --from` and `--to` accept `YYYY-MM-DD`, RFC3339, or epoch milliseconds. Date-only values use inclusive UTC calendar-day boundaries. Option chain and screen help list the valid `--type` values: `call`, `put`, and `all`. Option screen numeric filters reject non-finite values such as `NaN` and infinity before making API calls, and screen output serializes numeric values through the crate's active `Number` type so default and `decimal` builds stay consistent.

Mutable order commands are disabled unless `~/.config/schwab-agent/config.json` contains `"i-also-like-to-live-dangerously": true`. Before drafting or placing a symbol-specific order, use `schwab-agent order get --symbol AAPL` to inspect active open orders for that public ticker, adding `--account HASH` when the account scope is known. Use unfiltered `order get` or `order get --account HASH` when you need a broader conflict check across symbols or strategies. The recommended agent workflow is preview-first: save an order preview to get a digest, then submit the saved payload by digest after review. Saved previews use owner-only file permissions, but they are tamper-evident rather than encrypted. Mutable actions resolve account nicknames to canonical Schwab hashes and perform best-effort post-action order verification.

```bash
schwab-agent order equity buy AAPL -q 10 --price 180.00 --account HASH --save-preview
schwab-agent order place-from-preview --account HASH --digest DIGEST_HEX
```

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

`OrderBuilder` can also rebuild supported historical orders returned by the Trader API. Conversion keeps request fields, validates common response-level `quantity` against supported single-leg payloads, omits response metadata such as order IDs and status, and fails with `Error::OrderConversion` when an order is partial or uses unsupported order or leg shapes.

```rust,no_run
use schwab::{Client, Config, OrderBuilder, OrderListOptions};

# async fn example() -> schwab::Result<()> {
let client = Client::new(Config::new().bearer_token("your-token"));
let options = OrderListOptions::new("2026-01-01T00:00:00Z", "2026-01-31T23:59:59Z");
let orders = client.get_orders("123456789", options).await?;

if let Some(order) = orders.first() {
    let repeat = OrderBuilder::try_from_order(order)?;
    client.place_order("123456789", &repeat).await?;
}
# Ok(())
# }
```

Supported repeat-order strategies are `SINGLE`, `TRIGGER`, and `OCO` with equity or option legs.

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
| `cli` | Yes | Enables the bundled `schwab-agent` binary and CLI-only dependencies, including clap parsing, shell completions, browser opening, config directory lookup, and preview digests. |
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

## Development

Use the Makefile for the same checks CI expects:

```bash
make check
make coverage
make patch-coverage
make audit
make machete
```

`make check` runs formatting, clippy, tests, and rustdoc checks. Clippy and tests run with default features, with `--features decimal`, with `--lib --no-default-features`, and with `--lib --no-default-features --features decimal` so the `Number` alias stays valid and library consumers can build without CLI dependencies.

Offline tests include `cli` feature-gated compiled-binary smoke checks for `schwab-agent` help output, shell completions, clap usage errors, structured JSON error output, and hermetic dry-run order payloads. Live Schwab API tests remain gated behind `test_online`.

`make coverage` runs offline tests through nightly `cargo llvm-cov` with the `coverage_nightly` cfg enabled and enforces 90% line coverage. It does not enable `test_online`, because live Schwab API tests require explicit credentials and must never run in CI.

`make patch-coverage` generates `lcov.info` and runs `diff-cover` against `PATCH_COVERAGE_BASE` (default `main`) with `PATCH_COVERAGE_FAIL_UNDER` (default `100`). Set `DIFF_COVER` if you use `uvx diff-cover` or another wrapper.

`make machete` runs `cargo machete` to catch unused dependencies before CI does.

Generated `lcov.info` is ignored by git and CodeRabbit. CI pins the installed `cargo-llvm-cov` and `cargo-machete` versions, disables install-action fallback, gates Codecov upload with a non-secret presence flag, and scopes Codecov upload secrets only to the upload step.

Keep source, docs, fixtures, and copied API reference text ASCII unless the Schwab wire format explicitly requires Unicode. Decorative separators, mojibake, and non-breaking spaces can trigger Renovate hidden-Unicode warnings, so use plain ASCII equivalents.

## Release automation

release-plz runs through `.github/workflows/release-plz.yml`. It keeps a release PR current from Conventional Commits and the `cliff.toml` changelog configuration, refuses dirty working trees, and does not update dependencies because Renovate owns dependency bumps.

When the release PR is merged, release-plz creates the version tag. That tag triggers `.github/workflows/release.yml`, where cargo-dist builds `schwab-agent` binary artifacts, creates the GitHub Release, and publishes `schwab` to crates.io with GitHub Actions OIDC Trusted Publishing. The crates.io Trusted Publisher is configured for workflow file `release.yml`; never add `CARGO_REGISTRY_TOKEN` or another long-lived crates.io token.

## API stability

`schwab-rs` is pre-1.0. Public APIs may change while the crate tracks Schwab API behavior and fills out coverage for Market Data and Trader endpoints. Pin an exact crate version for production use.

## Minimum supported Rust version

This crate requires Rust **1.96** or later and uses Edition 2024.

## License

Apache-2.0. See [LICENSE](LICENSE) for details.
