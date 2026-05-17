# schwab-rs

> [!CAUTION]
> **After every code change, update `AGENTS.md`, `src/AGENTS.md`, `src/models/AGENTS.md`, and `README.md` before considering the work done.** Stale docs mislead reviewers and AI agents. Treat doc updates as part of the change, not a follow-up. Verify claims (signatures, variant lists, line counts, examples) against the actual code - do not copy from memory or prior versions.

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
make coverage       # cargo llvm-cov test --fail-under-lines 90
```

MSRV: 1.95. Edition: 2024. Always test with both default and `decimal` feature.

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
  streaming_api.rs    # Client::stream entry point (planned glue for streaming sessions)
  options.rs          # query parameter builder types
  order_builder.rs    # equity order construction
  query.rs            # query string helpers
  stream_session/     # WebSocket protocol, transport, StreamingSession engine, inline mock-transport tests
  test_support.rs     # test-only helpers (n(), fixture())
  models/             # see src/models/AGENTS.md
```

## Conventions

- Public API: `Client` + typed async methods returning `schwab::Result<T>`
- Root re-exports include `StreamingSession` plus streaming event, data, and field selector model types through `models::*`
- All public async methods: `&self` receiver, `#[instrument(skip_all)]` tracing attribute
- Two API bases: `MarketData` (`/marketdata/v1`) and `Trader` (`/trader/v1`) via `ApiBase` enum
- Streaming engine: `StreamingSession` owns a background WebSocket task, prioritizes queued outbound commands over reads in the loop, exposes `subscribe()` for broadcast `StreamEvent`s, supports account activity plus market data subscriptions, sends LOGOUT through `disconnect()`, replays subscriptions after reconnect, closes old transport before reconnecting, stores subscriptions before sending to avoid races with reconnect, and routes all subscription methods through shared symbol trimming, field-index serialization, validation, storage, and SUBS command delivery
- `Error::WebSocket` stores the tungstenite error in a `Box` so the crate-wide `Result<T>` stays small enough for `clippy::result_large_err`
- Streaming reconnect policy: 10 attempts, 1s exponential backoff doubled to a 30s cap, 0-500ms jitter, code 3 LOGIN_DENIED stops reconnecting
- `Config` builder: `Config::new()` with `.bearer_token("...")` and optional `.base_url()`/`.trader_base_url()` overrides
- All response model fields are `Option<T>` (Schwab API returns partial data)
- All enums in `enums.rs` are `#[non_exhaustive]` with `#[serde(rename_all = "...")]`
- Untagged/tagged dispatch enums (`QuoteResponseObject`, `SecuritiesAccount`, `AccountsInstrument`, `TransactionInstrument`) omit `#[non_exhaustive]` since serde cannot deserialize unknown variants for these
- Clippy: `-D clippy::all -A clippy::needless_borrow -A clippy::large_enum_variant`
- `#![deny(missing_docs)]` in `lib.rs` - all public items require doc comments (compile error if missing)
- Doc comments: short one-line summaries, action-verb start ("Get", "Place", "Parse")
- `# Errors` section required on all public `Result`-returning methods
- Model types use `#[allow(missing_docs)]` since struct fields and enum variants mirror JSON field names
- Rustdoc links in private modules must use `crate::Error` paths; `pub mod auth` can use bare `Error`
- US English spelling enforced
- Rustdoc examples required on all public structs, enums, functions, and async API methods
- Async network methods use ` ```no_run ` fences with `# async fn example() -> schwab::Result<()> {` boilerplate
- Sync builders and pure-logic items use plain ` ``` ` fences with compile-time assertions where possible
- Examples must compile under both default and `decimal` features (use `"1.0".parse().unwrap()` for `Number` literals)

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
- `msrv` (Rust 1.95, Ubuntu)
- `docs` (stable, Ubuntu)
- `audit` (daily cron + on Cargo.toml/Cargo.lock changes)

Release: `release-plz` runs automatically on every push to `main` via `.github/workflows/cd.yml`. A single job handles both release PRs and publishing in one step.

This repository exclusively uses crates.io Trusted Publishing with GitHub Actions OIDC (`id-token: write`) for all crate publishing. Never add `CARGO_REGISTRY_TOKEN` or any other long-lived registry token. The Trusted Publisher on crates.io is configured with workflow filename `cd.yml`. The workflow uses `RELEASE_PLZ_TOKEN` for GitHub operations (checkout, PR creation, tagging).

Changelog generation uses git-cliff via `cliff.toml` with Conventional Commits grouping (features, bug fixes, docs, performance, refactor, styling, testing, miscellaneous, security, reverts). The template produces version comparison links and commit SHA links.

Configuration lives in `release-plz.toml` (semver checking, changelog via cliff.toml, git release/tag settings, dependency updates, PR labels) and `cliff.toml` (git-cliff changelog template and commit parsing rules).

### Release Workflow

Automatic flow on push to `main`:

1. Push commits to `main` using Conventional Commits (`feat:`, `fix:`, etc.)
2. `cd.yml` triggers automatically and runs release-plz
3. release-plz opens/updates a PR with the version bump, `Cargo.lock` update, and `CHANGELOG.md` entries (generated by git-cliff)
4. Review and merge the release PR
5. The merge triggers release-plz again, which detects the version bump, runs `cargo publish`, creates git tag and GitHub release
6. Verify at `https://crates.io/crates/schwab`

### Manual Release Fallback

If release-plz is unavailable, version bumps can be done manually:

1. Bump `version` in `Cargo.toml`
2. Run `cargo update --workspace` to sync `Cargo.lock`
3. Commit **both** `Cargo.toml` and `Cargo.lock` together (dirty `Cargo.lock` causes `cargo publish` to fail)
4. Push to `main`
5. Run `cargo publish` locally

## Review Instructions

Detailed file-specific review instructions live in `.github/instructions/`. The project-wide review policy is in `.github/copilot-instructions.md`. Clippy allow attributes require a specific lint name and explanation comment.

## Keeping Documentation Current

When the code or project structure changes, keep these files updated to match:

- `AGENTS.md` (this file), `src/AGENTS.md`, `src/models/AGENTS.md` - AI agent context
- `CHANGELOG.md` - managed by release-plz automatically via release PRs
- `release-plz.toml` - release-plz configuration (semver check, changelog, git release settings)
- `cliff.toml` - git-cliff changelog template and commit parsing rules
- `README.md` - user-facing usage docs and feature descriptions
- `.coderabbit.yaml` - automated review configuration
- `.github/copilot-instructions.md` and `.github/instructions/*.instructions.md` - review policies

Stale documentation misleads both human reviewers and AI agents. Update these files as part of any PR that changes public API surface, conventions, build commands, CI workflows, or security requirements.

## Subdirectory Guides

- [`src/AGENTS.md`](src/AGENTS.md) - module architecture, API patterns, error handling, testing
- [`src/models/AGENTS.md`](src/models/AGENTS.md) - type design, serde patterns, Number alias
