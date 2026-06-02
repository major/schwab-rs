# src/models/ - API Response Types

> [!CAUTION]
> **After every code change, update this file, root `AGENTS.md`, `src/AGENTS.md`, and `README.md` before considering the work done.** Verify claims (signatures, variant lists, line counts, examples) against the actual code - do not copy from memory or prior versions.

## Module Structure

- `mod.rs` - `Number` type alias, `Quotes`/`Quote` compatibility aliases, re-exports all submodules
- `enums.rs` - ~70 enums shared across market data and trader APIs
- `market_data.rs` - quote responses, option chains, candles, instruments, market hours, screeners
- `streaming/` - streaming `StreamEvent`/`StreamData` types plus account activity, level-one equities, options, futures, futures options, forex, chart equity, chart futures, screener equity, and screener option field/data models
- `trader.rs` - accounts, recursive order request/response models, preview results, transactions, user preferences; `Order` response values are also the source type for `OrderBuilder::try_from_order(&Order)` repeat-order conversion, including validation of common response-level `quantity` against supported single-leg payloads

Everything is re-exported via `pub use` in `mod.rs`, then again via `models::*` at crate root.

The `schwab-agent` binary under `src/bin/schwab-agent/` consumes these same model types for CLI output and order conversion. CLI formatting may add compact row-shaped views, sanitized setup status objects, machine-readable discovery schema objects, local draft order JSON for `--dry-run` or `--preview` (choose one local draft flag per command), persist preview payloads outside the model layer under `XDG_STATE_HOME` or platform state directories with owner-only file permissions, normalize human-readable market history dates before building API query options, validate user-supplied numeric filters before model conversion, and enforce normalized contract-type filters in CLI rows, but it must not change the model-layer `Number`, serde, `Option<T>`, or enum conventions described here.

## Streaming Models

- `StreamEvent` variants: `Data`, `Response`, `Heartbeat`, `Disconnected`, `Reconnecting`, `Reconnected`
- `StreamData` variants: `AccountActivity`, `LevelOneEquities`, `LevelOneOptions`, `LevelOneFutures`, `LevelOneFuturesOptions`, `LevelOneForex`, `ChartEquities`, `ChartFutures`, `ScreenerEquities`, `ScreenerOptions`
- Field selector enums live beside their data structs and are re-exported from `models::streaming`: `AccountActivityField`, `EquityField`, `OptionField`, `FuturesField`, `FuturesOptionField`, `ForexField`, `ChartEquityField`, `ChartFuturesField`, `ScreenerEquityField`, `ScreenerOptionField`
- Streaming subscription code converts those field selector enums to protocol field indexes centrally in `StreamingSession::subscribe_service`; model modules still own each enum's `index()` values and `all()` slices
- `ScreenerItem` struct is defined in `screener_equity.rs` and shared with `screener_option.rs`; both screener data types use `Option<Vec<ScreenerItem>>` for the nested items array (field index 4)
- Streaming data structs parse numeric string keys with crate-internal `from_value()` helpers instead of serde derives because the WebSocket protocol uses field indexes as JSON keys. Account activity also parses named `seq` and `key` metadata.
- `from_value()` helpers initialize named metadata in struct literals first, then fill numeric index fields, which keeps partial subscription parsing readable and satisfies `clippy::field_reassign_with_default`
- Streaming numeric fields use `Number`, and all streaming data fields remain `Option<T>` because subscriptions may request partial field sets
- Streaming fixture files under `tests/fixtures/streaming_*.json` cover LOGIN responses, heartbeat notifications, account activity payloads, level-one equity/option numeric-key payloads, chart OHLCV data, and screener data with nested items arrays; keep fixture numbers compatible with both default `Number` and the `decimal` feature

## Number Type Alias

```rust
cfg_select! {
    feature = "decimal" => {
        pub type Number = rust_decimal::Decimal;
    }
    _ => {
        pub type Number = f64;
    }
}
```

All numeric fields in model structs use `Number`, never raw `f64` or `Decimal`. This ensures the `decimal` feature flag works transparently. Both variants must compile and pass tests. `OrderBuilder` uses `Number` for equity share quantities, option contract quantities, limit prices, and stop prices, including nested trigger/OCO bracket examples; its rustdoc examples parse numeric literals from strings so they compile under both default and `decimal` features.

## Repository Automation Notes

- Numeric model changes must pass tests with default `Number = f64`, with the `decimal` feature enabled, with `--lib --no-default-features`, and with `--lib --no-default-features --features decimal`. Routine checks must not enable `test_online`.
- CI coverage and `make patch-coverage` enforce a 90% line threshold with nightly `cargo llvm-cov` and the `coverage_nightly` cfg, use offline tests only, and must never enable `test_online`
- CLI smoke tests in `tests/cli_smoke.rs` run only with the `cli` feature and may assert serialized model-derived order JSON from hermetic implicit and explicit dry-run commands, sanitized config/doctor status output, machine-readable discovery schema output, opt-in JSON usage errors, command-specific help examples, valid option `--type` help values, raw shell completion output, and stderr diagnostics from `schwab-agent completions`; keep numeric output compatible with both default `Number` and `decimal` builds.
- CLI workflow docs should direct symbol-specific order conflict checks to `order get --symbol <SYMBOL>` while preserving broad unfiltered open-order checks when agents need to inspect all active orders.
- `cargo machete` runs in CI and through `make machete`; model dependency changes may require updating imports or dependencies together
- Generated `lcov.info` is ignored by git and CodeRabbit, and CI pins the installed coverage and machete tool versions with install-action fallback disabled
- The repository root `SKILL.md` is a pointer to the detailed `src/bin/schwab-agent/SKILL.md` CLI contract; model guidance remains in this file.
- Keep model fixtures and copied API reference text ASCII unless the Schwab wire format explicitly requires Unicode; hidden or decorative Unicode trips Renovate warnings and should be replaced with plain ASCII equivalents.
- Renovate dependency policy is inherited from the org-level shared config; validate with `npx --yes --package renovate renovate-config-validator`, and model changes should not add a repo-local Renovate config unless a repo-specific override is required.

## Type Design Rules

### All fields are `Option<T>`

Schwab's API returns partial data. Every response struct field must be `Option<T>`. No exceptions.

### Enums in `enums.rs` are `#[non_exhaustive]`

Schwab may add new variants at any time. Every enum in `enums.rs` gets `#[non_exhaustive]` so downstream code is forced to handle unknown variants. Untagged/tagged dispatch enums (`QuoteResponseObject`, `SecuritiesAccount`, `AccountsInstrument`, `TransactionInstrument`) omit `#[non_exhaustive]` since serde cannot deserialize unknown variants for these.

### Derive conventions

- Structs: `Clone, Debug, Deserialize, PartialEq`
- Enums: `Clone, Debug, Deserialize, Serialize, PartialEq, Eq` (when all variants are unit-like)
- Add `Serialize` to enums that appear in request payloads or query parameters
- Use `Eq` only when all fields support it (no `Number`/`f64` fields)
- `OrderStatus::Unknown` is the serde fallback for undocumented Schwab order status strings; keep known order lifecycle statuses as explicit variants when Schwab documents or returns them.
- `OrderType::Unknown` is response-only and must not be converted into `OrderTypeRequest`; repeat-order conversion returns `Error::OrderConversion` for it.

## Serde Patterns

### Field renaming

Most structs use `#[serde(rename_all = "camelCase")]` to match the Schwab JSON API.

### Enum renaming

Varies by enum. Common patterns:
- `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]` - most market data enums
- `#[serde(rename_all = "camelCase")]` - some trader enums
- Individual `#[serde(rename = "...")]` on specific variants when the API uses inconsistent casing

### Untagged enums (polymorphic responses)

Used when the API returns different shapes without a discriminator field:

- `QuoteResponseObject` - dispatches to per-asset response types (equity, option, forex, etc.)
- `AccountsInstrument` - dispatches by instrument shape
- `TransactionInstrument` - dispatches by instrument shape, uses `Box<Option<T>>` for recursive types

Untagged enums try variants in declaration order. Put the most specific (most fields) variants first.

`AccountsInstrument` can deserialize a minimal equity-shaped payload into the option variant because both variants have many optional fields and option appears first. Code that needs a submit asset type, such as order-to-builder conversion, must trust `OrderLegCollection::order_leg_type` or the nested instrument `asset_type` field instead of the enum variant alone.

### Tagged enums (discriminated unions)

Used when the API includes a type discriminator field:

- `SecuritiesAccount` - `#[serde(tag = "type")]` with `"MARGIN"` / `"CASH"` variants

### Recursive types

`OrderRequest` and `Order` contain `Option<Vec<...>>` child-order collections for nested OCO/TRIGGER strategies. `TransactionInstrument` uses `Box<Option<...>>` to break infinite size. Use `Box` when a type would otherwise be infinitely sized.

## Doc Comment Conventions

- Every public type (struct/enum) requires a `///` doc comment (enforced by `#![deny(missing_docs)]`)
- Types use `#[allow(missing_docs)]` since their fields/variants mirror JSON field names and are self-documenting
- Place the doc comment before the derive block, and `#[allow(missing_docs)]` immediately before `pub struct`/`pub enum`

## Adding New Types

1. Determine which file the type belongs in (`enums.rs`, `market_data.rs`, or `trader.rs`)
2. Add a `///` doc comment describing the type
3. Use `Option<T>` for every field
4. Add `#[non_exhaustive]` to enums
5. Add `#[allow(missing_docs)]` before the type to suppress field/variant doc requirements
6. Match the serde rename pattern of neighboring types
7. Use `Number` for all numeric fields, never `f64` or `Decimal` directly
8. Add the type to the corresponding OpenAPI spec reference in `docs/` if applicable
9. Test deserialization with a fixture in `tests/fixtures/`; streaming fixtures should include numeric string keys plus metadata such as `key` and `delayed`
10. Use Rust 1.96's standard `assert_matches!` macro for pattern assertions in tests
11. Verify compilation with both `cargo test` and `cargo test --features decimal`

## Keeping Documentation Current

When model types change (new structs, renamed fields, added enum variants), update this file to reflect the current state. Also update the root `AGENTS.md` and `src/AGENTS.md` if the change affects project-wide conventions or module architecture. Keep `README.md`, `.coderabbit.yaml`, and `.github/copilot-instructions.md` in sync with the actual code.
