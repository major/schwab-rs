# schwab-rs

Rust client library for the Charles Schwab brokerage API. Library crate only, no binaries.

## Build and Test

```bash
make check          # runs: fmt clippy test doc
make fmt            # cargo fmt --all --check
make fmt-fix        # cargo fmt --all
make clippy         # clippy twice: default features + --features decimal
make test           # cargo test twice: default features + --features decimal
make doc            # RUSTDOCFLAGS with deny flags, cargo doc --no-deps
make audit          # cargo audit
```

MSRV: 1.85. Edition: 2024. Always test with both default and `decimal` feature.

## Feature Flags

- `decimal`: swaps `Number` type alias from `f64` to `rust_decimal::Decimal`. All numeric model fields use `Number`, so both variants must compile and pass tests.
- `test_online`: gates live integration tests against the real Schwab API. Never run in CI.

## Project Layout

```text
src/
  lib.rs              # crate root, public module tree, re-exports
  auth.rs             # OAuth2 PKCE flow, token storage, provider
  client.rs           # HTTP client, endpoint routing
  config.rs           # Config builder, URL normalization
  error.rs            # Error enum (thiserror), redacted Debug
  market_data_api.rs  # 11 market data endpoint methods
  trader_api.rs       # 13 trader endpoint methods
  options.rs          # query parameter builder types
  order_builder.rs    # equity order construction
  query.rs            # query string helpers
  test_support.rs     # test-only helpers (n(), fixture())
  models/             # see src/models/AGENTS.md
```

## Conventions

- Public API: `Client` + typed async methods returning `schwab::Result<T>`
- All public async methods: `&self` receiver, `#[instrument(skip_all)]` tracing attribute
- Two API bases: `MarketData` (`/marketdata/v1`) and `Trader` (`/trader/v1`) via `ApiBase` enum
- `Config` builder: `Config::new(bearer_token)` with optional base URL overrides
- All response model fields are `Option<T>` (Schwab API returns partial data)
- All enums are `#[non_exhaustive]` with `#[serde(rename_all = "...")]`
- Clippy: `-D clippy::all -A missing_docs -A clippy::needless_borrow -A clippy::large_enum_variant`
- Doc comments: short one-line summaries, action-verb start ("Get", "Place", "Parse")
- US English spelling enforced

## Security (Non-Negotiable)

- Bearer tokens and HTTP response bodies MUST be redacted in Debug impls
- `auth.rs` callback URL restricted to `https://127.0.0.1` only
- Token files: directory 0o700, file 0o600 permissions
- Library must never call `process::exit`, write user-facing output, or read hidden config
- Flag any credential/token exposure in logs, errors, tests, or docs
- Order methods must not invent safety shortcuts or silently mutate payloads

## CI

Runs on Ubuntu, macOS, Windows:
- `fmt` (nightly rustfmt)
- `clippy` (stable, 3 OS)
- `test` (stable, 3 OS)
- `msrv` (Rust 1.85, Ubuntu)
- `docs` (stable, Ubuntu)
- `audit` (daily cron + on Cargo.toml/Cargo.lock changes)

Release: `release-plz` (manual trigger). `publish = false` in Cargo.toml.

## Review Instructions

Detailed file-specific review instructions live in `.github/instructions/`. The project-wide review policy is in `.github/copilot-instructions.md`. Clippy allow attributes require a specific lint name and explanation comment.

## Keeping Documentation Current

When the code or project structure changes, keep these files updated to match:

- `AGENTS.md` (this file), `src/AGENTS.md`, `src/models/AGENTS.md` - AI agent context
- `README.md` - user-facing usage docs and feature descriptions
- `.coderabbit.yaml` - automated review configuration
- `.github/copilot-instructions.md` and `.github/instructions/*.instructions.md` - review policies

Stale documentation misleads both human reviewers and AI agents. Update these files as part of any PR that changes public API surface, conventions, build commands, CI workflows, or security requirements.

## Subdirectory Guides

- [`src/AGENTS.md`](src/AGENTS.md) - module architecture, API patterns, error handling, testing
- [`src/models/AGENTS.md`](src/models/AGENTS.md) - type design, serde patterns, Number alias
