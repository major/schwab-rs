# src/bin/schwab-agent - JSON CLI

> **KEEP DOCS IN SYNC.** Any change to commands, args, output format, error codes, features, or workflows MUST be reflected in this file, `SKILL.md`, root `AGENTS.md`, `src/AGENTS.md`, `src/models/AGENTS.md`, and `README.md` as part of the same PR. Stale docs are worse than no docs.

## What This Is

Rust CLI binary (`schwab-agent`) embedded in the `schwab` crate to provide agent-oriented structured JSON output for Charles Schwab API workflows. Not public library API - it is CLI porcelain over the local crate modules plus direct HTTP workarounds for Schwab response shapes.

### Architecture Boundary: library vs CLI

The root library is a low-level API crate with typed request builders, transport, auth, and deserialization. Library modules must not read hidden config, write user-facing output, or exit the process. CLI-only data munging, response normalization, account selector resolution, mutable-operation guards, preview digest handling, and compact JSON output belong in `src/bin/schwab-agent/`. When the Schwab API returns unexpected formats, keep CLI workarounds in `raw.rs` using `Provider::token()` for auth and direct HTTP requests via reqwest unless the root library contract intentionally changes.

- Package: `schwab`, binary: `schwab-agent`
- Edition 2024, MSRV 1.96
- Install published binary with `cargo install schwab --bin schwab-agent --locked`
- Default feature: `cli` enables this binary and CLI-only dependencies. The library still builds with `default-features = false` without compiling `schwab-agent`.
- Feature flag: `decimal` swaps the shared `Number` alias to `rust_decimal::Decimal`

## Source Layout

```text
src/bin/schwab-agent/
  main.rs          - Binary entry point, module tree, run_from_env(), CLI dispatch, JSON output
  cli.rs           - clap derive CLI definition with subcommands and global args
  completions.rs   - Shell completion script generation for the clap command tree
  output.rs        - ErrorBody struct for structured error JSON output
  shared.rs        - Shared types: SessionChoice, DurationChoice, to_number() helper
  config.rs        - Agent config: load shared config, mutable-operation guard
  raw.rs           - Raw Schwab API requests with response normalization (account endpoint envelopes, false to null)
  error/
    mod.rs         - AppError enum (thiserror) with stable codes, exit codes, categories, hints
    tests.rs       - Error module tests
  auth/
    mod.rs         - Auth commands: status, login, login-url, exchange, refresh
    tests.rs       - Auth module tests
  market/
    mod.rs         - Market commands: history, quote. opt_field! macro, summarize_quote(), compact quote/history rows
    tests.rs       - Market module tests
  verify.rs        - Post-action verification: OrderActionResult, verify_order(), action_value()
  order/
    mod.rs         - Order command dispatch, inline tests
    equity.rs      - Equity order actions: buy, sell, sell-short, buy-to-cover
    option.rs      - Option order actions: buy-to-open, sell-to-open, buy-to-close, sell-to-close
    workflow.rs    - Order execution modes: dry-run, place, save-preview, preview-first
    replace.rs     - Replace an existing order with a new equity or option payload
    preview.rs     - SHA-256 tamper-evident preview with 15-min TTL (shared by equity + order)
    lifecycle.rs   - Order lifecycle commands: get, cancel, repeat with post-action verification
  account/
    mod.rs         - Account command: summary when no selector is provided, resolver when selector is provided, balance renderer
    tests.rs       - Account module tests
  options/
    mod.rs         - Module root: re-exports subcommand modules
    types.rs       - Shared types: FieldDef, FlatContract, flatten_chain, sort_contracts, select_fields, validate_fields, compute_dte, filter predicates
    tests.rs       - Options module tests
    expirations.rs - Expirations command: list available expiration dates
    chain.rs       - Chain command: option chain with server+client filtering
    contract.rs    - Contract lookup: single contract with curated flat output
    screen.rs      - Screen command: chain screening with liquidity/pricing filters
  ta/
    mod.rs           - Module root: re-exports dashboard handler
    types.rs         - Output types: DashboardOutput, ExpectedMoveOutput, AnalyzeOutput, signal types
    indicators.rs    - 8 hand-rolled TA indicators: SMA, EMA, RSI, MACD, ATR, BBands, Stochastic, ADX
    custom.rs        - Custom indicators: VWAP, Historical Volatility
    interval.rs      - Interval enum, Schwab API parameter mapping
    candles.rs       - Candle data extraction and validation helpers
    dashboard.rs     - Dashboard command handler with category-grouped output
    expected_move.rs - Expected move from ATM option straddle pricing
    tests.rs         - TA module tests
  analyze/
    mod.rs           - Multi-symbol analyze command with partial-failure support
    tests.rs         - Analyze module tests
SKILL.md            - Detailed LLM-facing command contract; root `SKILL.md` points here for discoverability
```

## Command Groups

- **auth** - Token management (status, login, login-url, exchange, refresh)
- **market** - Market data (history, quote)
- **account** - Account discovery, balances, positions, and resolution
- **order** - Unified order workflow: equity and option placement, lifecycle (get, cancel, replace, repeat), raw JSON
- **option** - Option chain data (expirations, chain, screen, contract)
- **ta** - Technical analysis (dashboard, expected-move)
- **analyze** - Multi-symbol analysis with partial-failure support
- **completions** - Raw shell completion scripts for bash, elvish, fish, powershell, and zsh

Command-specific `--help` for `market quote`, `market history`, `option chain`, `option screen`, `ta dashboard`, and `analyze` includes copyable examples. Keep these examples sanitized with public tickers and placeholders only, and keep `option chain --help` plus `option screen --help` listing valid `--type` values (`call`, `put`, `all`).

### Auth Callback Listener

`auth login` owns its local HTTPS callback listener in `src/bin/schwab-agent/auth/mod.rs` instead of using the one-shot listener from `schwab::auth::start_login()`. The listener must keep accepting requests through browser certificate-warning probes, wrong paths, missing query parameters, and other incomplete localhost requests. It should stop only after a complete Schwab OAuth callback with `code` and matching `state`, an OAuth error callback, a state mismatch, a bind/listener error, or the login timeout. Manual/headless flows still use `auth login-url` plus `auth exchange`.

### Equity Order Actions (4 total)

buy, sell, sell-short, buy-to-cover

Use `order equity ACTION` for stock orders. Each action hardcodes the Schwab `Instruction` to prevent accidental trade reversal. Supports order types: market (default), limit, stop, stop-limit.

`order preview-raw` and `order place-raw` accept arbitrary JSON payloads for complex order types (bracket, OCO, triggered orders) that use recursive `childOrderStrategies`.

### Option Order Actions (4 total)

buy-to-open, sell-to-open, buy-to-close, sell-to-close

Use `order option ACTION OCC_SYMBOL` for single-leg option orders. Each action hardcodes the Schwab `Instruction` to prevent accidental trade reversal. The OCC symbol must be the full 21-character format (e.g., `AAPL  250117C00150000`). For multi-leg orders, use `order place-raw` with a raw JSON payload.

### Order Workflow

The `-a`/`--account` flag controls execution mode:

- No `--account`: dry-run. Prints the OrderBuilder JSON locally, no API call.
- `--account HASH`: places the order directly.
- `--account HASH --save-preview`: previews only, saves the preview file, returns the SHA-256 digest.
- `--account HASH --preview-first`: previews first, then places automatically if the preview succeeds.

Recommended LLM workflow: pass `--save-preview` to get a digest, then `order place-from-preview --account HASH --digest DIGEST`. This submits the exact saved preview payload after the SHA-256 digest, 15-minute TTL, and account checks pass. Previews are stored in `$XDG_STATE_HOME/schwab-agent/previews/` when `XDG_STATE_HOME` is set, otherwise in the platform state or local data directory.

Preview responses can include non-fatal Schwab validation warnings. `--save-preview` and `preview-raw --save-preview` keep these as sanitized `warnings` entries in the command output while still saving a usable digest. The saved preview file continues to store only the submitted order payload and metadata.

Agents should prefer limit-style pricing whenever practical: pass `--price` so orders use `LIMIT`. Omitting `--price` intentionally creates a market order and should be reserved for cases where market execution is explicitly desired.

### Mutable Operation Guard

Commands that submit, replace, repeat-place, or cancel orders check `~/.config/schwab-agent/config.json` for `"i-also-like-to-live-dangerously": true` before executing. The config file is shared with the Go CLI.

- Missing config file or missing key = mutable operations disabled (safe default)
- Guard function: `config::require_mutable_enabled()` returns `AppError::MutableDisabled` (exit code 10, error code `config.mutable_disabled`)
- Guard is called inside the order dispatch handlers, before mutable API calls
- Read-only commands (dry-run mode, get, repeat `--save-preview`) are NOT gated

### Post-Action Verification

All mutable order actions (place, place-from-preview, place-raw, replace, repeat, cancel) immediately follow up with a GET to retrieve the order status. This is critical for LLM agents because Schwab's place/replace response only returns a Location header and order ID, not the actual order state.

The verification module (`src/bin/schwab-agent/verify.rs`) provides:

- `OrderActionResult` struct with the existing `order_id`, `location`, and submitted `order` fields, plus `verification_state` ("verified" or "unverified"), optional `verification_failures`, and the follow-up GET payload in `verified_order`
- `verify_order()` does a best-effort GET after any mutable action; on failure it returns `unverified` with failure details instead of propagating the error
- `action_value()` serializes the `OrderActionResult` directly to `Value` (verification failures are already in the struct)

### Order Lifecycle Commands

`order get`, `order replace`, `order repeat`, and `order cancel` manage existing orders.

- **get**: Discovery mode without `--order` queries orders through `raw::fetch_order_list()` so unexpected order activity values do not abort decoding. With no arguments, it returns active orders across all linked accounts. With `--account`, it returns active orders for that account only. Active means the returned `status` is in `ACTIVE_ORDER_STATUSES`; any other returned status is treated as inactive. `--symbol SYMBOL` keeps only orders whose `orderLegCollection` includes a matching instrument symbol, matches case-insensitively, includes multi-leg orders when any leg matches, and returns a successful empty `orders` array when no active orders match. Prefer `order get --symbol AAPL` for symbol-specific open-order checks and add `--account HASH` when the account scope is known; keep unfiltered `order get` or `order get --account HASH` for broader conflict checks across symbols or strategies. `--include-inactive` keeps inactive orders instead of filtering them out. `--from` and `--to` accept `YYYY-MM-DD` or RFC3339; date-only values are inclusive UTC calendar days. Specific-order mode is `order get --account HASH_OR_NICKNAME --order ORDER_ID`, resolves the account, and fetches the one order by ID. Unknown activity enum values are preserved in `orders` and summarized in an optional sanitized `warnings` array.
- **replace**: Replace an existing order by positive order ID. Requires `--account` and `--order-id`, then an `equity` or `option` subcommand with the new order payload. Includes post-replace verification via GET.
- **repeat**: Repeat an existing order by positive order ID. Requires `--account`; the order ID can be passed positionally or with `--order-id`. Fetches the source order, converts it with `schwab::OrderBuilder::try_from_order`, and runs the rebuilt payload through the normal order workflow. Supports `SINGLE`, `TRIGGER`, and `OCO` orders with equity or option legs when Schwab can convert them. Use `--save-preview` for the recommended digest workflow. Unsupported historical shapes return `order.validation_failed`; use `order place-raw` when the complex payload needs manual adaptation.
- **cancel**: Cancel by order ID, requires `--account`. The order ID can be passed positionally or with `--order-id`. Includes post-cancel verification via GET and only reports `verified` once the fetched status is `CANCELED`.

### Option Data Subcommands (4 total)

expirations, chain, screen, contract

Row-based output (columns + rows arrays) for expirations, chain, and screen. Flat object output for contract. All include underlying symbol context.

`option screen` validates all user-supplied numeric filters as finite values before applying them or making API calls. Do not allow `NaN` or infinity to become filter descriptions or empty-result success responses.

### Account Position Output

`account --positions` returns compact position objects with all curated fields Schwab provides: `symbol`, `cusip`, `instrument_id`, `description`, `asset_type`, `long_quantity`, `short_quantity`, `average_price`, `market_value`, `current_day_profit_loss`, and `current_day_profit_loss_percentage`. Missing Schwab fields are omitted from each position object; `cusip` and `instrument_id` are included when available so positions without symbols still have actionable instrument identifiers. A selector alone resolves a nickname or hash to the canonical account hash; a selector plus `--positions` returns a filtered account summary for the matching account.

### Market Quote and History Output

`market quote` is token-optimized by default and returns row-based output with `columns`, `rows`, and `rowCount`. Default columns are `req`, `sym`, `bid`, `ask`, `last`, `mark`, `chg`, `pct`, `vol`, and `err` so per-symbol quote errors stay visible in compact output. Use `--fields` to select output columns by compact names or full aliases such as `requested_symbol`, `symbol`, `net_change`, `net_percent_change`, `volume`, and `error`. Use `--all-fields` to return full detailed quote objects. Use `--api-fields quote,reference` to limit Schwab quote field groups requested from the API.

`market history` is token-optimized by default and returns row-based output with `symbol`, `columns`, `rows`, and `rowCount`. Default columns are `ts`, `open`, `high`, `low`, `close`, and `vol`. Use `--fields` to select candle columns by compact names or aliases such as `timestamp`, `datetime`, `datetimeISO8601`, `iso`, `o`, `h`, `l`, `c`, and `volume`. Use `--all-fields` to return the full Schwab price history object, including previous-close metadata and raw candle objects. `--from` and `--to` accept `YYYY-MM-DD`, RFC3339, or epoch milliseconds; date-only values are inclusive UTC calendar days and invalid values return `market.validation_failed` before auth or API calls.

Recommended LLM workflow: `expirations` (pick date) -> `chain` (with filters) -> `contract` (for detail). Use `screen` for multi-criteria filtering with liquidity and pricing constraints.

## Auth Configuration

Credentials are read from environment variables (`SCHWAB_CLIENT_ID`, `SCHWAB_CLIENT_SECRET`, `SCHWAB_CALLBACK_URL`) first, then from `~/.config/schwab-agent/config.json`. The callback URL defaults to `https://127.0.0.1:8182` when unset.

Token path env var: `SCHWAB_TOKEN_PATH`. Empty values are ignored. Default: `$XDG_CONFIG_HOME/schwab-agent-rs/token.json` for compatibility with existing agent installs, falling back to the platform config directory when `XDG_CONFIG_HOME` is unset.

## Output Format

Commands output raw JSON data payloads directly (no wrapper). Errors output an `ErrorBody` JSON object with `code`, `message`, `category`, `retryable`, and `hint` fields. `completions` is the only raw stdout exception because shell completion scripts must not be JSON-wrapped; completion generation write failures emit a short stderr diagnostic and exit non-zero.

### Error Codes and Exit Codes

- 3 = auth errors
- 4 = HTTP status errors
- 10 = input/validation/config errors (includes account.validation_failed, market.validation_failed, ta.insufficient_data, ta.invalid_interval, config.mutable_disabled)
- 11 = preview errors
- 20 = IO/JSON/config errors (includes account.response_shape, ta.calculation_error)

## Key Dependencies

- `clap` (derive) - CLI parsing
- `clap_complete` - Shell completion script generation
- `schwab` - Schwab API client
- `reqwest` - Direct HTTP requests for raw API workarounds
- `serde` / `serde_json` - Serialization
- `serde_with` - `skip_serializing_none` for clean JSON
- `thiserror` - Error derivation
- `tokio` - Async runtime
- `time` - Date/time handling
- `rcgen` / `rustls` - Local HTTPS callback listener
- `sha2` - Preview digest
- `tempfile` (dev) - Test fixtures

## Build and Test

Use `make check` for the full suite. Individual targets:

```bash
make fmt          # cargo fmt --all --check
make fmt-fix      # cargo fmt --all
make clippy       # Runs default, decimal, library no-default, library no-default+decimal
                  # Flags: -D clippy::all -A clippy::needless_borrow -A clippy::large_enum_variant
make test         # Runs default, decimal, library no-default, library no-default+decimal
make doc          # Checks default docs and library no-default docs for broken intra-doc links
make coverage     # nightly cargo llvm-cov test --fail-under-lines 90 with coverage_nightly cfg
make patch-coverage # lcov + diff-cover, 100% changed-line threshold against main
make audit        # cargo audit
make check        # fmt + clippy + test + doc (aggregate)
```

Always run default, `decimal`, library no-default, and library no-default `decimal` feature configurations. CI does the same without enabling `test_online`.

## Conventions

### Code Style

- Every module uses `#[cfg(test)] mod tests;` - separate test files for auth, error, market, account; inline tests for lib, cli, output, preview, order/mod, order/equity, order/option, order/replace, order/workflow, verify, lifecycle, raw
- `tests/cli_smoke.rs` runs only with the `cli` feature and uses `assert_cmd` and `predicates` to spawn the compiled `schwab-agent` binary for offline help output, shell completions, clap usage errors, structured JSON error output, and hermetic dry-run order JSON checks
- Docstrings on all public items and many private items
- `#[must_use]` on pure functions
- `serde_with::skip_serializing_none` for clean JSON output
- Tests use standard `assert!`/`assert_eq!` macros, `#[tokio::test]` for async tests
- Conventional commit messages

### Patterns to Follow

- New commands go in their command group module and get wired through `cli.rs` and `lib.rs`
- All command output is raw JSON data payloads; errors use `ErrorBody` struct
- Errors use `AppError` variants with stable error codes, exit codes, categories, and hints
- Equity and option order actions hardcode instruction (safety invariant)
- Mutable order actions (place, replace, repeat, cancel) use `verify::verify_order()` for post-action verification

### Testing

- Coverage threshold: 90% line coverage. Coverage runs use nightly `cargo llvm-cov` with the `coverage_nightly` cfg so explicitly marked browser, network, and dispatch paths are excluded from the denominator.
- Patch coverage threshold: 100% changed-line coverage via `make patch-coverage`. Override the base with `PATCH_COVERAGE_BASE=<branch>` or run with `DIFF_COVER='uvx diff-cover'` when `diff-cover` is not installed locally.

## CI

### ci.yml

- fmt (nightly rustfmt), clippy (stable), test (stable), MSRV (1.96, `--locked`), coverage upload to Codecov, docs (stable)
- Uses pinned action SHAs

### audit.yml

- `cargo audit` on push/PR when Cargo files change, plus daily cron

### release-plz.yml and release.yml

Release automation uses three chained components triggered by git events:

1. **git-cliff** (`cliff.toml`, referenced by `changelog_config = "cliff.toml"` in `release-plz.toml`) - Generates changelogs from Conventional Commits.
2. **release-plz** (`release-plz.yml` + `release-plz.toml`) - Runs on push to main. The release-pr job creates or updates the release PR, and the release job creates the git tag after the version bump lands. It does not publish to crates.io and does not create GitHub Releases.
3. **cargo-dist** (`release.yml` + `dist-workspace.toml`) - Triggered by version tags. Builds cross-platform `schwab-agent` binaries, creates the GitHub Release, and publishes the `schwab` crate to crates.io through Trusted Publishing. Never add `CARGO_REGISTRY_TOKEN`.

The `release-plz` job uses `RELEASE_PLZ_TOKEN` so release PR branch pushes trigger normal CI workflows.

`release.yml` is auto-generated by cargo-dist. Do not edit manually. Run `dist generate --ci github` to regenerate after changing `dist-workspace.toml`.

#### Release Workflow

Automatic flow on push to main:

1. Push commits to `main` using Conventional Commits (`feat:`, `fix:`, etc.)
2. `release-plz.yml` runs automatically, release-plz creates/updates a release PR with the version bump, `Cargo.lock` update, and `CHANGELOG.md` entries
3. Review and merge the release PR
4. Merge triggers `release-plz.yml` again, release-plz detects the version bump and creates a git tag
5. Git tag push triggers `release.yml`, cargo-dist builds binaries, creates the GitHub Release, and publishes `schwab`
6. Verify at `https://crates.io/crates/schwab`

#### Manual Release Fallback

If release-plz is unavailable, version bumps can be done manually:

1. Bump `version` in `Cargo.toml`
2. Run `cargo update --workspace` to sync `Cargo.lock`
3. Commit both `Cargo.toml` and `Cargo.lock` together (dirty `Cargo.lock` causes `cargo publish` to fail)
4. Push to `main`
5. Let `release-plz.yml` create the tag and `release.yml` publish through cargo-dist, or run `cargo publish` locally only if the release automation is unavailable

## Security

Keep account hashes, tokens, and credentials out of logs, errors, tests, and docs. The preview system stores submitted order payloads on disk with owner-only permissions and uses cryptographic digests for tamper detection; it is not encrypted, so do not treat saved previews as secret-free artifacts.

## Tooling Config

- **CodeRabbit** (`.coderabbit.yaml`): auto-review disabled (manual trigger via `@coderabbitai review`). References `**/AGENTS.md` as code guideline source.
- **Codecov** (`codecov.yml`): project and patch coverage gates. CI uploads `lcov.info`; local PR checks use `make patch-coverage` with `diff-cover`.
- **git-cliff** (`cliff.toml` for CLI, `changelog_config = "cliff.toml"` in `release-plz.toml` for CI): changelog generation from Conventional Commits with emoji-prefixed groups.
- **release-plz** (`release-plz.toml`, `.github/workflows/release-plz.yml`): push-to-main release PR and tag workflow. Changelog config is referenced from the `[workspace]` section.
- **cargo-dist** (`dist-workspace.toml`, `.github/workflows/release.yml`): tag-triggered cross-platform binary builds, GitHub Releases, and crates.io publish through Trusted Publishing.

## Files to Keep Updated

When the project changes (new commands, strategies, args, error codes, CI config, etc.), update:

- **`README.md`** - project overview and usage for GitHub
- **`AGENTS.md`** - this file
- **`SKILL.md`** - LLM-facing CLI usage guide
- **`cliff.toml`** - git-cliff changelog configuration
- **`release-plz.toml`** - release-plz configuration (semver check, changelog, release PRs, git tags)
- **`dist-workspace.toml`** - cargo-dist configuration (targets, installers, CI)
- **`.github/workflows/release-plz.yml`** - push-to-main release-plz workflow
- **`.github/workflows/release.yml`** - auto-generated cargo-dist workflow (do not edit manually)
- **`.github/instructions/*.instructions.md`** - review instructions for workflow-specific policies
- **`.coderabbit.yaml`** - path instructions and review guidelines
