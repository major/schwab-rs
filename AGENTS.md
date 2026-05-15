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

Release: `release-plz` runs on manual `workflow_dispatch` only. Two independent jobs:

- `release-pr`: opens/updates a PR with version bump and changelog entries (from Conventional Commits)
- `release`: publishes to crates.io, creates git tag, and GitHub release when a version bump lands on main

The workflow uses crates.io Trusted Publishing with GitHub Actions OIDC (`id-token: write`) instead of `CARGO_REGISTRY_TOKEN`. The Trusted Publisher is configured with workflow filename `release-plz.yml`. crates.io requires the first release of a brand-new crate to be published manually with a token that has `publish-new` scope before Trusted Publishing works.

The `release-pr` job uses the `RELEASE_PLZ_TOKEN` repository secret instead of the default `GITHUB_TOKEN` so release PR branch pushes trigger normal CI workflows. The `release` publish job can keep using `GITHUB_TOKEN` because it publishes after a version bump lands on `main` and does not need to trigger another workflow.

Configuration lives in `release-plz.toml` (semver checking, changelog, git release/tag settings).

### Release Workflow

Manual trigger flow (Actions > Release-plz > Run workflow):

1. Push commits to `main` using Conventional Commits (`feat:`, `fix:`, etc.)
2. When ready to release, trigger the workflow manually from GitHub Actions
3. `release-pr` opens a PR with the version bump, `Cargo.lock` update, and `CHANGELOG.md` entries
4. Review and merge the release PR
5. Trigger the workflow again to publish
6. `release` detects the version bump, runs `cargo publish`, creates git tag and GitHub release
7. Verify at `https://crates.io/crates/schwab`

### Manual Release Fallback

If `release-pr` is unavailable, version bumps can be done manually:

1. Bump `version` in `Cargo.toml`
2. Run `cargo update --workspace` to sync `Cargo.lock`
3. Commit **both** `Cargo.toml` and `Cargo.lock` together (dirty `Cargo.lock` causes `cargo publish` to fail)
4. Push to `main`
5. Trigger the release-plz workflow manually, or run `cargo publish` locally

## Review Instructions

Detailed file-specific review instructions live in `.github/instructions/`. The project-wide review policy is in `.github/copilot-instructions.md`. Clippy allow attributes require a specific lint name and explanation comment.

## Keeping Documentation Current

When the code or project structure changes, keep these files updated to match:

- `AGENTS.md` (this file), `src/AGENTS.md`, `src/models/AGENTS.md` - AI agent context
- `CHANGELOG.md` - managed by release-plz automatically via release PRs
- `release-plz.toml` - release-plz configuration (semver check, changelog, git release settings)
- `README.md` - user-facing usage docs and feature descriptions
- `.coderabbit.yaml` - automated review configuration
- `.github/copilot-instructions.md` and `.github/instructions/*.instructions.md` - review policies

Stale documentation misleads both human reviewers and AI agents. Update these files as part of any PR that changes public API surface, conventions, build commands, CI workflows, or security requirements.

## Subdirectory Guides

- [`src/AGENTS.md`](src/AGENTS.md) - module architecture, API patterns, error handling, testing
- [`src/models/AGENTS.md`](src/models/AGENTS.md) - type design, serde patterns, Number alias
