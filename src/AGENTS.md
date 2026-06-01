# src/ - Library Core

> [!CAUTION]
> **After every code change, update this file, root `AGENTS.md`, `src/models/AGENTS.md`, and `README.md` before considering the work done.** Verify claims (signatures, variant lists, line counts, examples) against the actual code - do not copy from memory or prior versions.

## Module Architecture

The public library remains rooted in `src/lib.rs`. The `schwab-agent` CLI is a separate binary target under `src/bin/schwab-agent/`, gated by the default `cli` feature so library consumers can build with `default-features = false`; it can use CLI config files, environment variables, structured JSON output, process exit codes, owner-only saved preview files, compatibility token paths, and CLI-specific validation such as rejecting non-finite option screen filters and enforcing normalized contract-type filters, but those behaviors must not leak into public library modules.

```text
lib.rs
  pub mod auth          # OAuth2 PKCE, token storage, provider (standalone module)
  mod client            # Client struct, HTTP plumbing (private, re-exported)
  mod config            # Config builder (private, re-exported)
  mod error             # Error/Result types (private, re-exported)
  mod market_data_api   # impl Client: 11 market data methods
  mod trader_api        # impl Client: 13 trader methods
  mod models            # response/request types (private, re-exported via models::*)
  mod options           # query param builders (private, re-exported individually)
  mod order_builder     # OrderBuilder construction and conversion (private, re-exported)
  mod query             # internal query string helpers
  mod stream_session    # WebSocket protocol, transport, StreamingSession engine
  mod streaming_api     # Client::stream entry point, credentials bootstrap
  mod test_support      # #[cfg(test)] helpers
bin/schwab-agent/       # Binary-private JSON CLI modules and tests
```

Root re-exports: `Client`, `Config`, `Error`, `Result`, `models::*`, all option builder types, `OrderBuilder`, `StreamingSession`.

## Adding API Methods

Market data and trader endpoints follow the same pattern:

1. Add an `async fn` on `Client` in the appropriate `*_api.rs` file
2. Use `&self` receiver with `#[instrument(skip_all)]`
3. Build the URL with `self.endpoint_url(ApiBase::MarketData or Trader, &["path", "segments"])`
4. Use `self.send_json(Method::GET, url, &query, None)` for GET returning JSON, or `self.send_empty(Method::DELETE, url)` for simple POST/PUT/DELETE
5. Return `Result<T>` where `T` is a model type from `models/`

Example pattern from existing methods:

```rust
#[instrument(skip_all)]
pub async fn get_quotes<S>(&self, symbols: impl IntoIterator<Item = S>) -> Result<Quotes>
where
    S: AsRef<str>,
{
    let symbols = comma_separated_symbols(symbols)?;
    let url = self.endpoint_url(ApiBase::MarketData, &["quotes"])?;
    let query = vec![("symbols", symbols)];
    self.send_json(Method::GET, url, &query, None).await
}
```

For methods with optional query parameters, build a `Vec<(&str, String)>` using helpers from `query.rs` (`push_optional`, `required_text`, etc.).

For POST/PUT with a JSON body, use `self.send_empty_with_location(method, url, &body)` which serializes the body and returns an `OrderResponse`.

6. Add a `# Errors` doc section listing specific `Error` variants returned by the method
7. Use `crate::Error::VariantName` in rustdoc links (private modules cannot resolve bare `Error`)
8. Add a `# Examples` section with a `no_run` block showing typical usage (use `# async fn example() -> schwab::Result<()> {` boilerplate)

## Client Internals (`client.rs`)

- `ApiBase` enum selects between market data and trader base URLs
- `endpoint_url(api_base: ApiBase, path_segments: &[&str])` - builds a URL from the base URL and path segments
- `send_json(method: Method, url: Url, query: &[(&str, String)], body: Option<Value>)` - send request, deserialize response as JSON
- `send_empty(method: Method, url: Url)` - send request, return `()` on success
- `send_empty_with_location(method: Method, url: Url, body: &B)` - send request, return `OrderResponse` wrapping the `Location` header
- All methods attach bearer token via `Authorization` header
- HTTP errors map to `Error::HttpStatus { status, body }` with body truncated/redacted in Debug

## Config (`config.rs`)

- `Config::new()` creates a config with default API base URLs
- `.bearer_token("...")` sets the bearer token via builder method
- `.base_url()` and `.trader_base_url()` override the defaults
- `normalize_base_url()` ensures trailing slash removal and valid URL parsing
- `Debug` impl redacts `bearer_token` field

## Error Handling (`error.rs`)

- `Error` enum uses `thiserror` derive
- Variants: `EmptyBaseUrl`, `InvalidBaseUrl`, `EmptySymbols`, `MissingRequiredParameter`, `OrderConversion`, `InvalidAuthConfig`, `AuthRequired`, `AuthExpired`, `AuthCallback`, `Io`, `Encode`, `Json`, `HttpStatus`, `Request`, `Decode`, `WebSocket`, `StreamLogin`, `StreamProtocol`
- `WebSocket` boxes the tungstenite source error so crate-wide `Result<T>` does not trip `clippy::result_large_err`
- Manual `Debug` impl on `Error`: redacts `body` field in `HttpStatus` and `Decode` to `<redacted>`
- `Result<T>` is `std::result::Result<T, Error>`
- Never expose raw HTTP response bodies in error messages or debug output

## Auth Module (`auth.rs`)

Largest module (~1900 lines). Handles the full OAuth2 PKCE flow:

- `AuthConfig` - credentials and OAuth settings
- `authorize_url()` - build the Schwab authorization URL with PKCE challenge
- `login()` / `start_login()` - spawn a local HTTPS callback server (self-signed TLS), open browser, exchange code
- `TokenStore` trait - async read/write interface for token persistence
- `FileTokenStore` - file-based with strict permissions (dir 0o700, file 0o600)
- `MemoryTokenStore` - in-memory for testing
- `Provider` - wraps a `Client` + `TokenStore`, auto-refreshes expired tokens
- Callback server restricted to `https://127.0.0.1` only

## Options Builders (`options.rs`)

Builder types for query parameter construction. Each has a `new()` constructor and chained setter methods:

- `QuoteOptions` - fields, indicative
- `OptionChainOptions` - symbol, contract type, strike count, strategy, range, dates, volatility, etc.
- `MoverOptions` - index, sort, frequency
- `PriceHistoryOptions` - symbol, period, frequency, date range, extended hours
- `OrderListOptions` - date range, max results, status, order filtering
- `TransactionListOptions` - account, date range, transaction type, symbol

All builders produce a `Vec<(&str, String)>` consumed by query string assembly.

Trader order responses use `OrderStatus::Unknown` as the serde fallback for undocumented order lifecycle values, while known statuses such as `WORKING`, `CANCELED`, and `REJECTED` deserialize to typed variants.

## Order Builder (`order_builder.rs`)

`OrderBuilder` constructs serializable equity and single-leg option order payloads for `place_order`, `replace_order`, and `preview_order`. Factory methods:

- `market_buy()`, `market_sell()` - market orders
- `limit_buy()`, `limit_sell()` - limit orders
- `stop_buy()`, `stop_sell()` - stop orders
- `stop_limit_buy()`, `stop_limit_sell()` - stop-limit orders
- `equity_market()`, `equity_limit()`, `equity_stop()`, `equity_stop_limit()` - lower-level constructors for explicit `Instruction` values such as short sales
- `option_buy_to_open_market()`, `option_buy_to_open_limit()`, `option_sell_to_open_market()`, `option_sell_to_open_limit()`, `option_buy_to_close_market()`, `option_buy_to_close_limit()`, `option_sell_to_close_market()`, `option_sell_to_close_limit()` - single-leg option helpers for common open/close flows
- `option_market()`, `option_limit()` - lower-level constructors for explicit option `Instruction` values
- `one_cancels_other()`, `first_triggers_second()` - compose nested `childOrderStrategies` before submission

The builder itself implements `Serialize`; pass `&order` directly to trader methods. Single-order constructors set `NORMAL` session, `DAY` duration, and `SINGLE` strategy by default. Equity helpers set `assetType=EQUITY`; option helpers set `assetType=OPTION` and trust the caller-provided Schwab option symbol without parsing or rewriting it. OCO parent payloads omit order type, session, duration, and legs so they do not invent simple-order fields. TRIGGER examples use `first_triggers_second()` for supported parent/child flows such as buying shares first and sending a stop-loss sell only after the buy fills; bracket examples use an entry order as TRIGGER parent with an OCO child containing profit-taking limit and stop-loss sell orders. Multi-leg option spread examples are intentionally excluded until the builder models spread fields. Every public method has structured rustdoc sections covering arguments, defaults, payload effects, cautions for lower-level constructors, and examples so downstream CLIs can use rustdoc as command-help source material. Does NOT silently add fields or mutate the payload beyond what the caller specified.

`OrderBuilder::try_from_order(&Order)`, `TryFrom<&Order>`, and `TryFrom<Order>` convert historical `Order` response models back into submit-ready builder payloads for supported `SINGLE`, `TRIGGER`, and `OCO` strategies. Conversion keeps request-relevant fields, recursively converts child strategies, supports equity and option legs, omits response metadata such as order IDs and status, validates common top-level `quantity` against supported single-leg payloads, validates order-type-specific required prices, and returns `Error::OrderConversion` instead of guessing when required submit fields are missing or order/leg shapes are unsupported.

## Query Helpers (`query.rs`)

Internal functions for building query parameter vectors:

- `required_text()` - adds a required string param
- `comma_separated_symbols()` / `comma_separated_symbols_required()` - join symbol lists
- `push_optional()` - conditionally add optional params

## Streaming Module (`stream_session/`)

- `StreamingSession` is non-generic for callers; `StreamingSession::new<T: WsTransport + 'static>` is crate-internal for mockable construction
- `WsTransport` has a source-local `MockTransport` in `src/stream_session/mod.rs` tests; it scripts `next()` responses with `VecDeque<Result<Option<String>>>` and records `send()` payloads for LOGIN/SUBS assertions
- `subscribe()` returns a `tokio::sync::broadcast::Receiver<StreamEvent>` with a 1024-event buffer
- `disconnect()` sends LOGOUT through the background task, closes the transport, and stops the loop
- Subscription methods cover account activity, level-one equities, options, futures, futures options, forex, chart equity, chart futures, screener equity, and screener option
- All subscription methods route through the private `subscribe_service` helper, which trims symbol/key inputs, serializes each service-specific field enum to field indexes, validates non-empty symbols and fields, stores the active subscription, and sends the SUBS command
- The message loop uses a biased `tokio::select!` with command handling before reads so ready SUBS commands are acknowledged before an already-ready close/read branch can move into reconnect handling
- The session stores one active subscription per service and replays stored SUBS commands after reconnect
- Subscriptions are stored before sending the SUBS command so a concurrent reconnect can replay them; on send failure the stored subscription is rolled back
- Symbol inputs are trimmed and whitespace-only entries filtered before validation
- `replay_subscriptions` propagates errors so a failed replay is treated as a reconnect failure and triggers the next attempt
- The message loop closes the old transport before entering the reconnect path
- Malformed or missing heartbeat timestamps are logged and skipped rather than broadcast as epoch values
- Login response validation requires an explicit `Some(0)` success code; a missing code is treated as a protocol error
- Reconnect policy: 10 attempts, 1s exponential backoff doubled to a 30s cap, 0-500ms jitter, code 3 LOGIN_DENIED stops reconnecting
- Message dispatch maps supported service names (account activity, level-one, chart, and screener) into the matching `StreamData` variants and skips unknown services with a warning
- `WsTransport` trait methods return `Send` futures so the session loop can run inside `tokio::spawn`
- `StreamParameters` uses a manual `Debug` impl that shows `"<redacted>"` for the `authorization` field so bearer tokens never appear in debug output; `StreamRequest` and `StreamRequestItem` derive `Debug` and inherit the redaction transitively; `SessionCredentials` also has a manual `Debug` impl that shows `"<redacted>"` for `bearer_token`; all redaction strings use `"<redacted>"` to match the convention in `Error`'s manual `Debug` impl
- `build_view()` passes `customer_id` and `correl_id` into the request item like all other protocol builders; `OptionField`, `FuturesField`, and `FuturesOptionField` are all `#[non_exhaustive]`

## Test Patterns

- Inline `#[cfg(test)] mod tests` blocks in each source file
- `mockito` for HTTP mocking: create a mock server, set expectations, verify request shape
- Streaming tests use inline `MockTransport` plus golden fixtures under `tests/fixtures/streaming_*.json`; keep them offline and deterministic
- Tests verify: HTTP method, path, query params, headers, request body, response deserialization
- Use Rust 1.96's standard `assert_matches!` macro for pattern assertions in tests
- `test_support.rs` provides:
  - `n(value)` - convert numeric literals to `Number` (works for both f64 and Decimal)
  - `fixture(name)` - load JSON from `tests/fixtures/{name}.json`
- Live integration tests in `tests/integration.rs`, gated behind `#[cfg(feature = "test_online")]`
- Always run tests with default features, `--features decimal`, `--lib --no-default-features`, and `--lib --no-default-features --features decimal`. Do not use `--all-features` for routine offline checks because that enables `test_online`.
- CI and local coverage use nightly `cargo llvm-cov` with the `coverage_nightly` cfg, a 90% line threshold, offline tests only, and must not enable `test_online`
- `make patch-coverage` writes `lcov.info` and uses `diff-cover` against `PATCH_COVERAGE_BASE` (default `main`) so changed lines stay tested
- `make machete` and the CI `machete` job run `cargo machete` for unused dependency checks. CI pins the installed `cargo-llvm-cov` and `cargo-machete` versions and disables install-action fallback.
- Generated `lcov.info` is ignored by git and CodeRabbit path filters and should not be reviewed as source

## Keeping Documentation Current

When modules, public API methods, or internal patterns change, update this file, root `AGENTS.md`, `src/models/AGENTS.md`, and `README.md`. Also keep `.coderabbit.yaml` and `.github/copilot-instructions.md` in sync with the actual code.
