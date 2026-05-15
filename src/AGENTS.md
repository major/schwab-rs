# src/ - Library Core

> [!CAUTION]
> **After every code change, update this file, root `AGENTS.md`, `src/models/AGENTS.md`, and `README.md` before considering the work done.** Verify claims (signatures, variant lists, line counts, examples) against the actual code - do not copy from memory or prior versions.

## Module Architecture

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
  mod order_builder     # OrderBuilder (private, re-exported)
  mod query             # internal query string helpers
  mod streaming         # WebSocket protocol, transport, StreamingSession engine
  mod streaming_api     # Client::stream entry point (planned glue)
  mod test_support      # #[cfg(test)] helpers
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
- Variants: `EmptyBaseUrl`, `InvalidBaseUrl`, `EmptySymbols`, `MissingRequiredParameter`, `InvalidAuthConfig`, `AuthRequired`, `AuthExpired`, `AuthCallback`, `Io`, `Encode`, `Json`, `HttpStatus`, `Request`, `Decode`, `WebSocket`, `StreamLogin`, `StreamProtocol`
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

## Order Builder (`order_builder.rs`)

`OrderBuilder` constructs equity order JSON payloads. Factory methods:

- `market_buy()`, `market_sell()` - market orders
- `limit_buy()`, `limit_sell()` - limit orders
- `stop_buy()`, `stop_sell()` - stop orders
- `stop_limit_buy()`, `stop_limit_sell()` - stop-limit orders

Produces a `serde_json::Value` via `build()`. Does NOT silently add fields or mutate the payload beyond what the caller specified.

## Query Helpers (`query.rs`)

Internal functions for building query parameter vectors:

- `required_text()` - adds a required string param
- `comma_separated_symbols()` / `comma_separated_symbols_required()` - join symbol lists
- `push_optional()` - conditionally add optional params

## Streaming Module (`streaming/`)

- `StreamingSession` is non-generic for callers; `StreamingSession::new<T: WsTransport + 'static>` is crate-internal for mockable construction
- `WsTransport` has a source-local `MockTransport` in `src/streaming/mod.rs` tests; it scripts `next()` responses with `VecDeque<Result<Option<String>>>` and records `send()` payloads for LOGIN/SUBS assertions
- `subscribe()` returns a `tokio::sync::broadcast::Receiver<StreamEvent>` with a 1024-event buffer
- `disconnect()` sends LOGOUT through the background task, closes the transport, and stops the loop
- Level-one subscription methods cover equities, options, futures, futures options, and forex only
- The message loop uses a biased `tokio::select!` with command handling before reads so ready SUBS commands are acknowledged before an already-ready close/read branch can move into reconnect handling
- The session stores one active subscription per service and replays stored SUBS commands after reconnect
- Reconnect policy: 10 attempts, 1s exponential backoff doubled to a 30s cap, 0-500ms jitter, code 3 LOGIN_DENIED stops reconnecting
- Message dispatch maps supported level-one service names into the matching `StreamData` variants and skips unknown services with a warning
- `WsTransport` trait methods return `Send` futures so the session loop can run inside `tokio::spawn`

## Test Patterns

- Inline `#[cfg(test)] mod tests` blocks in each source file
- `mockito` for HTTP mocking: create a mock server, set expectations, verify request shape
- Streaming tests use inline `MockTransport` plus golden fixtures under `tests/fixtures/streaming_*.json`; keep them offline and deterministic
- Tests verify: HTTP method, path, query params, headers, request body, response deserialization
- `test_support.rs` provides:
  - `n(value)` - convert numeric literals to `Number` (works for both f64 and Decimal)
  - `fixture(name)` - load JSON from `tests/fixtures/{name}.json`
- Live integration tests in `tests/integration.rs`, gated behind `#[cfg(feature = "test_online")]`
- Always run tests with both default features and `--features decimal`

## Keeping Documentation Current

When modules, public API methods, or internal patterns change, update this file, root `AGENTS.md`, `src/models/AGENTS.md`, and `README.md`. Also keep `.coderabbit.yaml` and `.github/copilot-instructions.md` in sync with the actual code.
