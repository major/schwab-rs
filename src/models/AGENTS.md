# src/models/ - API Response Types

> [!CAUTION]
> **After every code change, update this file, root `AGENTS.md`, `src/AGENTS.md`, and `README.md` before considering the work done.** Verify claims (signatures, variant lists, line counts, examples) against the actual code - do not copy from memory or prior versions.

## Module Structure

- `mod.rs` - `Number` type alias, `Quotes`/`Quote` compatibility aliases, re-exports all submodules
- `enums.rs` - ~70 enums shared across market data and trader APIs
- `market_data.rs` - quote responses, option chains, candles, instruments, market hours, screeners
- `streaming/` - streaming `StreamEvent`/`StreamData` types plus level-one equities, options, futures, futures options, forex, chart equity, and chart futures field/data models
- `trader.rs` - accounts, orders, transactions, user preferences

Everything is re-exported via `pub use` in `mod.rs`, then again via `models::*` at crate root.

## Streaming Models

- `StreamEvent` variants: `Data`, `Response`, `Heartbeat`, `Disconnected`, `Reconnecting`, `Reconnected`
- `StreamData` variants: `LevelOneEquities`, `LevelOneOptions`, `LevelOneFutures`, `LevelOneFuturesOptions`, `LevelOneForex`, `ChartEquities`, `ChartFutures`
- Field selector enums live beside their data structs and are re-exported from `models::streaming`: `EquityField`, `OptionField`, `FuturesField`, `FuturesOptionField`, `ForexField`, `ChartEquityField`, `ChartFuturesField`
- Streaming data structs parse numeric string keys with crate-internal `from_value()` helpers instead of serde derives because the WebSocket protocol uses field indexes as JSON keys
- `from_value()` helpers initialize named metadata in struct literals first, then fill numeric index fields, which keeps partial subscription parsing readable and satisfies `clippy::field_reassign_with_default`
- Streaming numeric fields use `Number`, and all streaming data fields remain `Option<T>` because subscriptions may request partial field sets
- Streaming fixture files under `tests/fixtures/streaming_*.json` cover LOGIN responses, heartbeat notifications, and level-one equity/option numeric-key payloads; keep fixture numbers compatible with both default `Number` and the `decimal` feature

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

All numeric fields in model structs use `Number`, never raw `f64` or `Decimal`. This ensures the `decimal` feature flag works transparently. Both variants must compile and pass tests.

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

### Tagged enums (discriminated unions)

Used when the API includes a type discriminator field:

- `SecuritiesAccount` - `#[serde(tag = "type")]` with `"MARGIN"` / `"CASH"` variants

### Recursive types

`Order` contains `Vec<Option<Order>>` for child orders. `TransactionInstrument` uses `Box<Option<...>>` to break infinite size. Use `Box` when a type would otherwise be infinitely sized.

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
10. Verify compilation with both `cargo test` and `cargo test --features decimal`

## Keeping Documentation Current

When model types change (new structs, renamed fields, added enum variants), update this file to reflect the current state. Also update the root `AGENTS.md` and `src/AGENTS.md` if the change affects project-wide conventions or module architecture. Keep `README.md`, `.coderabbit.yaml`, and `.github/copilot-instructions.md` in sync with the actual code.
