# Schwab API reference files

This directory keeps the Schwab OpenAPI files beside the Rust crate so the generated or handwritten client code can be reviewed against the API contract.

| File | Purpose |
|---|---|
| `market_data.openapi.json` | Market Data API contract, including quotes, price history, instruments, market hours, movers, and option chains. |
| `trader_api.openapi.json` | Trader API contract, including accounts, orders, transactions, and user preferences. |

These files were originally sourced from the `schwab-go` project's `docs/` directory. Keep this copy in sync when Schwab publishes updated specs.
