# src/ - Library Core

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
  mod test_support      # #[cfg(test)] helpers
```

Root re-exports: `Client`, `Config`, `Error`, `Result`, `models::*`, all option builder types, `OrderBuilder`.

## Adding API Methods

Market data and trader endpoints follow the same pattern:

1. Add an `async fn` on `Client` in the appropriate `*_api.rs` file
2. Use `&self` receiver with `#[instrument(skip_all)]`
3. Build the URL with `self.endpoint_url(ApiBase::MarketData or Trader, "path")`
4. Use `self.send_json(&url)` for GET returning JSON, or `self.send_empty(...)` for POST/PUT/DELETE
5. Return `Result<T>` where `T` is a model type from `models/`

Example pattern from existing methods:

```rust
#[instrument(skip_all)]
pub async fn get_quote(&self, symbol: &str) -> Result<QuoteResponseObject> {
    let url = self.endpoint_url(ApiBase::MarketData, &format!("{symbol}/quotes"));
    self.send_json(&url).await
}
```

For methods with query parameters, build a `Vec<(&str, String)>` using helpers from `query.rs`, then append with `Url::query_pairs_mut().extend_pairs()`.

For POST/PUT with a JSON body, use `self.send_empty()` or `self.send_empty_with_location()` which accept a `reqwest::RequestBuilder`.

## Client Internals (`client.rs`)

- `ApiBase` enum selects between market data and trader base URLs
- `endpoint_url()` joins the base URL with a relative path
- `send_json()` - GET request, deserialize response as JSON
- `send_empty()` - send request, return `()` on success
- `send_empty_with_location()` - send request, extract and return the `Location` header value
- All methods attach bearer token via `Authorization` header
- HTTP errors map to `Error::HttpStatus { code, body }` with body truncated/redacted in Debug

## Config (`config.rs`)

- `Config::new(bearer_token)` sets defaults for both API base URLs
- `base_url()` and `trader_base_url()` override the defaults
- `normalize_base_url()` ensures trailing slash removal and valid URL parsing
- `Debug` impl redacts `bearer_token` field

## Error Handling (`error.rs`)

- `Error` enum uses `thiserror` derive
- Variants: `HttpStatus`, `Reqwest`, `SerdeJson`, `UrlParse`, `Header`, `Io`, `Auth`, `Token`, `Tls`, `Redirect`
- Manual `Debug` impl on `Error`: redacts `body` field in `HttpStatus` to `[REDACTED]`
- `Result<T>` is `std::result::Result<T, Error>`
- Never expose raw HTTP response bodies in error messages or debug output

## Auth Module (`auth.rs`)

Largest module (1400+ lines). Handles the full OAuth2 PKCE flow:

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

## Test Patterns

- Inline `#[cfg(test)] mod tests` blocks in each source file
- `mockito` for HTTP mocking: create a mock server, set expectations, verify request shape
- Tests verify: HTTP method, path, query params, headers, request body, response deserialization
- `test_support.rs` provides:
  - `n(value)` - convert numeric literals to `Number` (works for both f64 and Decimal)
  - `fixture(name)` - load JSON from `tests/fixtures/{name}.json`
- Live integration tests in `tests/integration.rs`, gated behind `#[cfg(feature = "test_online")]`
- Always run tests with both default features and `--features decimal`

## Keeping Documentation Current

When modules, public API methods, or internal patterns change, update this file and the root `AGENTS.md`. Also keep `README.md`, `.coderabbit.yaml`, and `.github/copilot-instructions.md` in sync with the actual code.
