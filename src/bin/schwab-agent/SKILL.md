# schwab-agent CLI

Structured JSON CLI for Charles Schwab API. All command output is raw JSON data payloads except `completions`, which prints a shell completion script. Set env vars or config once, then most commands need zero flags.

> **Disclaimer:** This project is unofficial and is not affiliated with, endorsed by, or connected to Charles Schwab, TD Ameritrade, or thinkorswim in any way.

## Setup

```bash
export SCHWAB_CLIENT_ID="..."
export SCHWAB_CLIENT_SECRET="..."
# Token path defaults to $XDG_CONFIG_HOME/schwab-agent-rs/token.json for compatibility with existing installs
# Override with a non-empty SCHWAB_TOKEN_PATH if needed
# Callback URL defaults to https://127.0.0.1:8182
```

Credentials can also live in `~/.config/schwab-agent/config.json`. Environment variables take precedence over config file values.

## Release Notes

The binary is published from the `schwab` crate as `schwab-agent`. Install it with `cargo install schwab --bin schwab-agent --locked`. Releases are automated on push to main: release-plz creates release PRs and tags from Conventional Commits, then cargo-dist builds cross-platform binaries, creates GitHub Releases, and publishes `schwab` to crates.io through Trusted Publishing.

Generate shell completions with `schwab-agent completions <shell>`, where `<shell>` is one of `bash`, `elvish`, `fish`, `powershell`, or `zsh`. This command writes the raw completion script to stdout instead of JSON so shells can source it directly; write failures emit a short stderr diagnostic and exit non-zero.

## Mutable Operation Guard

Commands that submit, replace, repeat-place, or cancel orders require `"i-also-like-to-live-dangerously": true` in `~/.config/schwab-agent/config.json`. Without it, these commands return error code `config.mutable_disabled` (exit code 10). Read-only commands (build, preview, get) are not gated. `order repeat --save-preview` only previews and saves a digest, so it remains available without the mutable guard; direct repeat placement and `--preview-first` are gated.

## Auth

```bash
schwab-agent auth status          # check token state
schwab-agent auth login-url       # get OAuth URL (open in browser)
schwab-agent auth exchange --redirect-url "CALLBACK_URL_WITH_CODE"
schwab-agent auth refresh         # refresh expired token
schwab-agent auth login           # interactive: opens browser, waits for callback
```

`auth login` keeps listening through browser certificate-warning probes and other incomplete localhost requests. It stops when Schwab sends the full OAuth callback, returns an OAuth error, hits a state mismatch, or the login timeout expires.

If you get `auth.token_missing`, run `login-url` then `exchange`. If `auth.expired`, run `refresh`.

## Market Data

```bash
schwab-agent market quote AAPL              # single quote
schwab-agent market quote AAPL MSFT GOOG    # multiple quotes
schwab-agent market quote AAPL --fields sym,last,pct,vol
schwab-agent market quote AAPL --all-fields
schwab-agent market history SPY             # price history (defaults are fine)
schwab-agent market history SPY --fields ts,close,vol
schwab-agent market history SPY --all-fields
```

Quote output defaults to token-efficient rows: `columns`, `rows`, and `rowCount`. Default columns are `req`, `sym`, `bid`, `ask`, `last`, `mark`, `chg`, `pct`, `vol`, and `err` so per-symbol quote errors stay visible in compact output. Use `--fields` for specific output columns, using compact names or full aliases such as `requested_symbol`, `symbol`, `net_change`, `net_percent_change`, `volume`, and `error`. Use `--all-fields` for full detailed quote objects. Use `--api-fields quote,reference` only to limit Schwab API field groups.

History output also defaults to token-efficient rows with `symbol`, `columns`, `rows`, and `rowCount`. Default candle columns are `ts`, `open`, `high`, `low`, `close`, and `vol`, which are enough for most trading decisions and TA handoffs. Use `--fields` for specific candle columns, using compact names or aliases such as `timestamp`, `datetime`, `datetimeISO8601`, `iso`, `o`, `h`, `l`, `c`, and `volume`. Use `--all-fields` for the full Schwab price history object, including previous-close metadata and raw candle objects.

Optional history flags: `--period-type`, `--period`, `--frequency-type`, `--frequency`, `--from`, `--to`, `--extended-hours`.

## Account

Discover and resolve accounts before placing orders.

Recommended workflow: `account` -> choose `account_hash` or nickname -> pass to `--account` in order commands.

```bash
schwab-agent account                                    # list accounts with balances
schwab-agent account --positions                        # include holdings as compact objects
schwab-agent account Trading                            # resolve nickname to canonical hash
schwab-agent account Trading --positions                # selected account summary with holdings
schwab-agent account ABCDEF1234567890                   # verify a known hash
```

Position output with `--positions` returns compact position objects with all curated fields Schwab provides: `symbol`, `cusip`, `instrument_id`, `description`, `asset_type`, `long_quantity`, `short_quantity`, `average_price`, `market_value`, `current_day_profit_loss`, and `current_day_profit_loss_percentage`. Missing Schwab fields are omitted from each position object; `cusip` and `instrument_id` are included when available so positions without symbols still have actionable instrument identifiers. Add `--positions` to a selector when you need holdings for one account; omit position flags when you only need canonical hash resolution.

The `--account` flag on order commands accepts either the canonical account hash or a unique nickname. Raw account numbers are not supported.


## Equity Orders

Buy/sell shares of stock. Recommended LLM workflow: pass `--account HASH --save-preview` to get a digest, then `order place-from-preview --account HASH --digest DIGEST`.

The `-a`/`--account` flag controls execution mode:
- No `--account`: dry-run (prints order JSON, no API call)
- `--account HASH`: places directly
- `--account HASH --save-preview`: previews and saves digest
- `--account HASH --preview-first`: previews then places automatically

Non-fatal Schwab preview warnings do not block digest creation. When present, `--save-preview` and `preview-raw --save-preview` include sanitized `warnings` entries with severity, message, and validation rule fields; the saved digest still covers only the order payload and preview metadata. Saved previews use `$XDG_STATE_HOME/schwab-agent/previews/` when set, otherwise the platform state or local data directory.

Prefer limit orders when practical: pass `--price` for limit orders. Omit `--price` only when a market order is explicitly desired.

```bash
# Dry-run (no account = no API call)
schwab-agent order equity buy AAPL -q 10
schwab-agent order equity sell AAPL -q 10

# Limit order dry-run
schwab-agent order equity buy AAPL -q 10 --price 180.00

# Stop order dry-run
schwab-agent order equity buy AAPL -q 10 --stop 170.00

# Stop-limit dry-run
schwab-agent order equity buy AAPL -q 10 --price 169.00 --stop 170.00

# Short selling
schwab-agent order equity sell-short AAPL -q 10 --price 200.00
schwab-agent order equity buy-to-cover AAPL -q 10 --price 180.00

# Preview and save digest (recommended LLM workflow)
schwab-agent order equity buy AAPL -q 100 --price 180.00 -a HASH --save-preview

# Place from saved preview (15-min TTL)
schwab-agent order place-from-preview -a HASH -d DIGEST_HEX

# Direct place (for explicit human requests only)
schwab-agent order equity buy AAPL -q 100 --price 180.00 -a HASH
```

## Option Orders

Single-leg option orders using OCC symbols. Recommended LLM workflow: pass `--account HASH --save-preview` to get a digest, then `order place-from-preview`.

The same `-a`/`--account` execution modes apply as for equity orders.

Prefer limit orders when practical: pass `--price`. Omit `--price` only when a market order is explicitly desired.

```bash
# Dry-run
schwab-agent order option buy-to-open "AAPL  250117C00150000" -q 1 --price 5.00

# Preview and save digest
schwab-agent order option buy-to-open "AAPL  250117C00150000" -q 1 --price 5.00 -a HASH --save-preview

# Place from saved preview
schwab-agent order place-from-preview -a HASH -d DIGEST_HEX

# Direct place
schwab-agent order option sell-to-open "SPY   250620P00550000" -q 1 --price 4.50 -a HASH

# Close positions
schwab-agent order option buy-to-close "AAPL  250117C00150000" -q 1 --price 2.00 -a HASH
schwab-agent order option sell-to-close "SPY   250620P00550000" -q 1 --price 3.00 -a HASH
```

For multi-leg orders (spreads, straddles, condors), use `order place-raw` with a raw JSON payload.

### Complex Orders (Bracket, OCO, Trigger)

Use `order preview-raw` and `order place-raw` to submit arbitrary JSON payloads for order types not covered by the porcelain commands. This is the path for bracket orders, OCO (one-cancels-other), and triggered orders.

#### Bracket Order (Buy + Stop Loss + Profit Target)

A bracket order is a `TRIGGER` parent with two `OCO` child orders. When the parent fills, both children activate; when one child fills, the other cancels.

```bash
schwab-agent order preview-raw --account HASH --save-preview --json '{
  "orderType": "LIMIT",
  "session": "NORMAL",
  "duration": "DAY",
  "orderStrategyType": "TRIGGER",
  "price": "180.00",
  "orderLegCollection": [
    {
      "instruction": "BUY",
      "quantity": 100,
      "instrument": {"symbol": "AAPL", "assetType": "EQUITY"}
    }
  ],
  "childOrderStrategies": [
    {
      "orderStrategyType": "OCO",
      "childOrderStrategies": [
        {
          "orderType": "LIMIT",
          "session": "NORMAL",
          "duration": "GOOD_TILL_CANCEL",
          "orderStrategyType": "SINGLE",
          "price": "200.00",
          "orderLegCollection": [
            {
              "instruction": "SELL",
              "quantity": 100,
              "instrument": {"symbol": "AAPL", "assetType": "EQUITY"}
            }
          ]
        },
        {
          "orderType": "STOP",
          "session": "NORMAL",
          "duration": "GOOD_TILL_CANCEL",
          "orderStrategyType": "SINGLE",
          "stopPrice": "170.00",
          "orderLegCollection": [
            {
              "instruction": "SELL",
              "quantity": 100,
              "instrument": {"symbol": "AAPL", "assetType": "EQUITY"}
            }
          ]
        }
      ]
    }
  ]
}'
```

#### OCO Order (Stop Loss OR Profit Target)

An OCO order places two orders where filling one cancels the other. Use this when you already hold shares and want to set both a stop loss and a profit target.

```bash
schwab-agent order place-raw --account HASH --json '{
  "orderStrategyType": "OCO",
  "childOrderStrategies": [
    {
      "orderType": "LIMIT",
      "session": "NORMAL",
      "duration": "GOOD_TILL_CANCEL",
      "orderStrategyType": "SINGLE",
      "price": "200.00",
      "orderLegCollection": [
        {
          "instruction": "SELL",
          "quantity": 100,
          "instrument": {"symbol": "AAPL", "assetType": "EQUITY"}
        }
      ]
    },
    {
      "orderType": "STOP",
      "session": "NORMAL",
      "duration": "GOOD_TILL_CANCEL",
      "orderStrategyType": "SINGLE",
      "stopPrice": "170.00",
      "orderLegCollection": [
        {
          "instruction": "SELL",
          "quantity": 100,
          "instrument": {"symbol": "AAPL", "assetType": "EQUITY"}
        }
      ]
    }
  ]
}'
```

#### Triggered Order (Buy, Then Stop Loss)

A `TRIGGER` parent fires its child orders when the parent fills. Use this when you want a stop loss activated automatically after a buy.

```bash
schwab-agent order place-raw --account HASH --json '{
  "orderType": "LIMIT",
  "session": "NORMAL",
  "duration": "DAY",
  "orderStrategyType": "TRIGGER",
  "price": "180.00",
  "orderLegCollection": [
    {
      "instruction": "BUY",
      "quantity": 100,
      "instrument": {"symbol": "AAPL", "assetType": "EQUITY"}
    }
  ],
  "childOrderStrategies": [
    {
      "orderType": "STOP",
      "session": "NORMAL",
      "duration": "GOOD_TILL_CANCEL",
      "orderStrategyType": "SINGLE",
      "stopPrice": "170.00",
      "orderLegCollection": [
        {
          "instruction": "SELL",
          "quantity": 100,
          "instrument": {"symbol": "AAPL", "assetType": "EQUITY"}
        }
      ]
    }
  ]
}'
```

#### Key Fields for Complex Orders

- `orderStrategyType`: `"SINGLE"` (leaf), `"TRIGGER"` (parent fires children on fill), `"OCO"` (one-cancels-other)
- `childOrderStrategies`: Array of child orders (recursive structure)
- `instruction`: `"BUY"`, `"SELL"`, `"BUY_TO_COVER"`, `"SELL_SHORT"`
- `orderType`: `"MARKET"`, `"LIMIT"`, `"STOP"`, `"STOP_LIMIT"`, `"TRAILING_STOP"`
- Prices are strings in raw JSON (e.g., `"180.00"` not `180.00`)

## Order Lifecycle

```bash
schwab-agent order get                                                                    # active orders across all linked accounts
schwab-agent order get --account HASH                                                     # active orders for one account
schwab-agent order get --symbol IBM                                                       # active orders whose legs include IBM
schwab-agent order get --include-inactive --from 2025-01-01 --to 2025-01-31
schwab-agent order get --account HASH --order 12345678                                    # single order by ID
schwab-agent order replace -a HASH --order-id 12345678 equity buy AAPL -q 10 --price 148.00  # replace with equity order
schwab-agent order replace -a HASH --order-id 12345678 option buy-to-open "AAPL  250117C00150000" -q 1 --price 4.50
schwab-agent order repeat -a HASH 12345678 --save-preview                                 # rebuild existing order + save digest
schwab-agent order repeat -a HASH --order-id 12345678 --preview-first                     # rebuild, preview, then place
schwab-agent order cancel --account HASH 12345678                                         # cancel + verify
schwab-agent order cancel --account HASH --order-id 12345678                              # equivalent named-flag form
```

Get discovery flags: `--account` (optional hash or nickname), `--symbol`, `--from`/`--to` (`YYYY-MM-DD` or RFC3339), `--recent`, and `--include-inactive`. Without `--account`, `order get` returns active orders across all linked accounts. With `--account`, it returns active orders for that account. With `--symbol SYMBOL`, it keeps only orders whose `orderLegCollection` includes a matching instrument symbol; matching is case-insensitive, multi-leg orders are included when any leg matches, and no matches returns a successful empty `orders` array. Active means the returned `status` exactly matches one of the strings in the `active_statuses` output field; any other status is treated as inactive and kept only with `--include-inactive`. Date-only ranges are inclusive UTC calendar days, so `--from 2026-05-28 --to 2026-05-31` includes both end dates and the dates between them. Output: `{"orders": [...], "count": N, "include_inactive": false, "active_statuses": [...]}` plus optional sanitized `warnings` when Schwab returns unrecognized order activity enum values. Canceled order activities are preserved and do not make discovery fail. Specific-order mode is `order get --account HASH --order ORDER_ID`; do not combine `--order` with discovery filters.

Repeat workflow: `order repeat --account HASH ORDER_ID --save-preview` is the safest default. It fetches the historical order, rebuilds a new order payload, and saves a preview digest. `order repeat` supports Schwab-convertible `SINGLE`, `TRIGGER`, and `OCO` orders with equity or option legs. It drops response-only fields such as original order ID, status, timestamps, account number, and fill history. Unsupported shapes return `order.validation_failed`; switch to `order place-raw` if you need to hand-edit a complex payload.

## Post-Action Verification

All mutable actions (place, place-from-preview, place-raw, replace, repeat, cancel) auto-verify by GETting the order after the action. Schwab only returns a Location header on placement and replacement, so this GET is what gives the LLM actual order state.

Response fields: `action` ("place"/"replace"/"cancel"), `order_id`, `location`, `order` (submitted payload), `verification_state` ("verified"/"unverified"), and `verified_order` (full order from GET when available). Optional: `verification_failures` (when unverified), `digest`/`original_command` (for place-from-preview). Unverified failures are included in the response; the order may still have succeeded. Cancel verification is only `verified` when the fetched order status is `CANCELED`.

### Duration Aliases

`day` (default), `good-till-cancel`/`gtc`, `fill-or-kill`/`fok`, `immediate-or-cancel`/`ioc`

## Option Data

Read-only option chain commands for research and strategy selection. No orders are placed. Recommended workflow: `expirations` to pick a date, `chain` to scan strikes, `contract` for a single contract's full detail. Use `screen` when you need multi-criteria filtering across expirations and strikes.

### Expirations

```bash
schwab-agent option expirations AAPL
```

Returns a row-based list of available expiration dates for the underlying. Use the dates here as input to `--expiration` in `chain`, `screen`, and `contract`.

### Chain

```bash
# Full chain (all expirations, all strikes)
schwab-agent option chain AAPL

# Calls near 30 DTE with selected fields
schwab-agent option chain AAPL --type call --dte 30 --fields strike,delta,bid,ask,volume,oi

# Puts in a strike range with delta filter
schwab-agent option chain AMD --type put --strike-min 140 --strike-max 160 --delta-min -0.30 --delta-max -0.15

# Exact expiration, specific strike count around ATM
schwab-agent option chain SPY --expiration 2025-06-20 --strike-count 10
```

Chain flags:

| Flag | Description |
|---|---|
| `--type call\|put\|all` | Contract type filter (default: all) |
| `--dte N` | Nearest expiration by days to expiration |
| `--expiration YYYY-MM-DD` | Exact expiration date |
| `--delta-min N` | Minimum delta filter |
| `--delta-max N` | Maximum delta filter |
| `--fields LIST` | Comma-separated field list |
| `--strike-count N` | Strikes around at-the-money |
| `--strike N` | Exact strike price |
| `--strike-min N` | Minimum strike price |
| `--strike-max N` | Maximum strike price |
| `--strike-range RANGE` | Schwab strike range filter |

Output is row-based: `{ "columns": [...], "rows": [[...], ...], "rowCount": N }`.

### Screen

Screen adds liquidity and pricing filters on top of all chain flags. Use it when you want to narrow results by volume, open interest, spread quality, or premium range.

```bash
# Liquid calls with tight spreads, 20-45 DTE
schwab-agent option screen AAPL --type call --dte-min 20 --dte-max 45 --min-volume 100 --min-oi 500 --max-spread-pct 10

# Premium range filter with result limit
schwab-agent option screen SPY --type put --min-premium 1.00 --max-premium 5.00 --limit 20
```

Screen-only flags (all chain flags also apply):

| Flag | Description |
|---|---|
| `--dte-min N` | Minimum days to expiration |
| `--dte-max N` | Maximum days to expiration |
| `--min-bid N` | Minimum bid price |
| `--max-ask N` | Maximum ask price |
| `--min-volume N` | Minimum volume |
| `--min-oi N` | Minimum open interest |
| `--max-spread-pct N` | Maximum spread percent |
| `--min-premium N` | Minimum premium |
| `--max-premium N` | Maximum premium |
| `--sort FIELD` | Sort field |
| `--limit N` | Maximum number of results |

Output adds `totalScanned` and `filtersApplied` alongside the row-based data. Numeric filters must be finite values; `NaN` and infinity are validation errors, not empty-result filters.

### Contract

Look up a single contract by expiration, strike, and type. Returns a flat object (no columns/rows).

```bash
schwab-agent option contract AAPL --expiration 2025-06-20 --strike 200 --call
schwab-agent option contract SPY --expiration 2025-06-20 --strike 550 --put
```

All three flags are required: `--expiration YYYY-MM-DD`, `--strike N`, and one of `--call` or `--put`.

## Technical Analysis

Read-only TA commands. No orders are placed.

### Dashboard

Runs all indicators for a symbol and returns category-grouped output: trend, momentum, volatility, and volume. Includes derived fields (ATR percent, relative volume, distance from SMAs) and signal interpretations.

```bash
schwab-agent ta dashboard AAPL                          # daily dashboard, 20 points
schwab-agent ta dashboard SPY --interval weekly --points 10
```

Dashboard flags:

| Flag | Description |
|---|---|
| `--interval INTERVAL` | Candle interval: daily (default), weekly, 1min, 5min, 15min, 30min |
| `--points N` | Number of data points per indicator series (default: 20) |

### Expected Move

Computes expected move from the ATM straddle price in the option chain. Output includes straddle price, expected move (price and percent), upper/lower ranges, and implied volatility from ATM options.

```bash
schwab-agent ta expected-move AAPL                      # 30-day expected move
schwab-agent ta expected-move SPY --dte 45
```

Expected-move flags:

| Flag | Description |
|---|---|
| `--dte N` | Target days to expiration for the option chain (default: 30) |

## Analyze

Multi-symbol analysis combining quote and TA dashboard per symbol. Partial failures include per-symbol error fields (`quote_error`, `analysis_error`) alongside successful results.

```bash
schwab-agent analyze AAPL                    # single symbol
schwab-agent analyze AAPL MSFT GOOG          # multiple symbols
schwab-agent analyze AAPL --interval weekly --points 10
```

Analyze flags:

| Flag | Description |
|---|---|
| `--interval INTERVAL` | Candle interval (same values as ta dashboard) |
| `--points N` | Number of data points per indicator series (default: 1) |

## Output Format

Commands output raw JSON data payloads directly (no wrapper envelope). Errors output a structured JSON object:

```json
{"code": "auth.token_missing", "message": "...", "category": "auth", "retryable": false, "hint": "..."}
```

On error (non-zero exit code), read `hint` for recovery steps. Check `retryable` before retrying.

### Error Codes

| Code | Meaning | Recovery |
|---|---|---|
| `auth.config_missing` | No client ID/secret | Add to `~/.config/schwab-agent/config.json` or set `SCHWAB_CLIENT_ID`/`SCHWAB_CLIENT_SECRET` |
| `auth.token_missing` | No token file | Run `auth login-url` then `auth exchange` |
| `auth.expired` | Token expired | Run `auth refresh` |
| `auth.required` | Auth needed | Run full auth flow |
| `schwab.http_status` | API HTTP error | Check message for status code |
| `input.empty_symbols` | No symbols given | Provide at least one symbol |
| `account.validation_failed` | Account input validation error | Read the error message and hint for details (unknown account selector, ambiguous nickname) |
| `account.response_shape` | Schwab account response shape is not recognized | Update schwab-agent or report the sanitized shape metadata from the message |
| `market.validation_failed` | Invalid market-data params | Use a listed `--fields` value or read the error hint |
| `order.validation_failed` | Bad order params | Check strike/expiration values |
| `order.preview_failed` | Preview issue | Re-run preview (may have expired) |
| `options.symbol_not_found` | Symbol has no options | Verify symbol is optionable |
| `options.validation_failed` | Invalid option params | Check expiration/strike values |
| `ta.insufficient_data` | Not enough candle data | Try a shorter interval or fewer points |
| `ta.invalid_interval` | Unrecognized interval | Use: daily, weekly, 1min, 5min, 15min, 30min |
| `config.mutable_disabled` | Mutable ops disabled | Set `"i-also-like-to-live-dangerously": true` in config |
| `ta.calculation_error` | Indicator math failed | Check input data quality |
