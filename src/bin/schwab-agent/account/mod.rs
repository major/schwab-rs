//! Account discovery, balance summary, position, and selector resolution handlers.

use serde::Serialize;
use serde_json::{Value, to_value};

use schwab::{
    Account, AccountNumberHash, AccountsInstrument, CashBalance, CashInitialBalance, MarginBalance,
    MarginInitialBalance, SecuritiesAccount, UserPreference, UserPreferenceAccount,
};

use crate::auth;
use crate::cli::{AccountArgs, Cli};
use crate::error::AppError;

/// Dispatches the account command and returns its JSON value.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn handle(_cli: &Cli, args: &AccountArgs) -> Result<Value, AppError> {
    if let Some(selector) = &args.selector
        && !args.requests_summary()
    {
        let provider = auth::provider()?;
        let token = provider.token().await?;
        let data = resolve_account(&token, selector).await?;
        return Ok(to_value(data)?);
    }

    let provider = auth::provider()?;
    let token = provider.token().await?;
    let data = run_summary(&token, args.include_positions(), args.selector.as_deref()).await?;
    Ok(to_value(data)?)
}

/// A normalized brokerage account row for summary output.
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct AccountRow {
    pub account_hash: String,
    pub nickname: Option<String>,
    pub display_account_id: Option<String>,
    pub primary_account: Option<bool>,
    pub account_type: Option<String>,
    pub is_closing_only_restricted: Option<bool>,
    pub is_day_trader: Option<bool>,
    pub balances: Option<AccountBalances>,
    pub positions: Option<Value>,
}

/// Account balance summary, tagged by account kind.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum AccountBalances {
    Margin(MarginBalanceSummary),
    Cash(CashBalanceSummary),
}

/// Status for margin-safe true cash discovery.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrueCashStatus {
    Verified,
    Unavailable,
    NotApplicable,
}

/// Margin account balance summary.
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct MarginBalanceSummary {
    pub true_cash: Option<schwab::Number>,
    pub true_cash_status: TrueCashStatus,
    pub cash_balance: Option<schwab::Number>,
    pub cash_available_for_trading: Option<schwab::Number>,
    pub cash_available_for_withdrawal: Option<schwab::Number>,
    pub buying_power: Option<schwab::Number>,
    pub stock_buying_power: Option<schwab::Number>,
    pub option_buying_power: Option<schwab::Number>,
    pub equity: Option<schwab::Number>,
}

/// Cash account balance summary.
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct CashBalanceSummary {
    pub true_cash: Option<schwab::Number>,
    pub true_cash_status: TrueCashStatus,
    pub cash_balance: Option<schwab::Number>,
    pub cash_available_for_trading: Option<schwab::Number>,
    pub cash_available_for_withdrawal: Option<schwab::Number>,
    pub total_cash: Option<schwab::Number>,
}

/// Account summary payload.
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct AccountSummaryData {
    pub accounts: Vec<AccountRow>,
}

#[derive(Debug)]
struct AccountFields {
    account_number: Option<String>,
    variant_type: &'static str,
    is_closing_only_restricted: Option<bool>,
    is_day_trader: Option<bool>,
    balances: Option<AccountBalances>,
    positions: Option<Value>,
}

/// Account resolution payload.
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct AccountResolveData {
    pub account_hash: String,
    pub matched_by: String,
    pub nickname: Option<String>,
    pub display_account_id: Option<String>,
    pub primary_account: Option<bool>,
    pub account_type: Option<String>,
}

/// Builds a compact [`AccountRow`] from a hash entry and optional user preference account.
///
/// The `account_hash` field comes from `AccountNumberHash.hash_value`.
/// Raw account numbers are never included in the output.
#[must_use]
pub fn build_account_row(hash_value: String, pref: Option<&UserPreferenceAccount>) -> AccountRow {
    AccountRow {
        account_hash: hash_value,
        nickname: pref
            .and_then(|p| p.nick_name.clone())
            .filter(|n| !n.is_empty()),
        display_account_id: pref.and_then(|p| p.display_acct_id.clone()),
        primary_account: pref.and_then(|p| p.primary_account),
        account_type: pref.and_then(|p| p.r#type.clone()),
        is_closing_only_restricted: None,
        is_day_trader: None,
        balances: None,
        positions: None,
    }
}

fn preference_accounts(preferences: Vec<UserPreference>) -> Vec<UserPreferenceAccount> {
    preferences
        .into_iter()
        .filter_map(|preference| preference.accounts)
        .flatten()
        .collect()
}

/// Fetches accounts, account hashes, and user preferences, then renders compact
/// account rows with balance summaries.
///
/// Uses a raw HTTP request to normalize Schwab API quirks (object-wrapped
/// arrays, boolean `false` in numeric fields) before deserialization.
///
/// When `with_positions` is true, account data is fetched with position details
/// included. Otherwise, positions are omitted from the output.
///
/// # Errors
///
/// Returns an `AppError` when any Schwab API call fails.
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn run_summary(
    bearer_token: &str,
    with_positions: bool,
    selector: Option<&str>,
) -> Result<AccountSummaryData, AppError> {
    let http = reqwest::Client::new();
    let fields = with_positions.then_some("positions");
    let (hashes, prefs, accounts, selected_hash) = if let Some(selector) = selector {
        let (hashes, preferences) = tokio::try_join!(
            crate::raw::fetch_account_numbers_with_client(&http, bearer_token),
            crate::raw::fetch_user_preference_with_client(&http, bearer_token),
        )?;
        let prefs = preference_accounts(preferences);
        let selected_hash = resolve_account_from_data(&hashes, &prefs, selector)?.account_hash;
        let accounts = crate::raw::fetch_accounts_with_client(&http, bearer_token, fields).await?;

        (hashes, prefs, accounts, Some(selected_hash))
    } else {
        let (hashes, preferences, accounts) = tokio::try_join!(
            crate::raw::fetch_account_numbers_with_client(&http, bearer_token),
            crate::raw::fetch_user_preference_with_client(&http, bearer_token),
            crate::raw::fetch_accounts_with_client(&http, bearer_token, fields),
        )?;
        let prefs = preference_accounts(preferences);

        (hashes, prefs, accounts, None)
    };

    let mut summary = render_summary_from_data(&accounts, &hashes, &prefs, with_positions);
    if let Some(account_hash) = selected_hash.as_deref() {
        retain_account_summary(&mut summary, account_hash);
        ensure_selected_account_rendered(&summary, account_hash)?;
    }

    Ok(summary)
}

/// Keeps only the account row matching `account_hash`.
pub(crate) fn retain_account_summary(summary: &mut AccountSummaryData, account_hash: &str) {
    summary
        .accounts
        .retain(|account| account.account_hash == account_hash);
}

/// Validates that a selected account survived summary rendering.
///
/// # Errors
///
/// Returns [`AppError::AccountValidation`] when Schwab account details could not
/// be joined back to the resolved account hash.
pub(crate) fn ensure_selected_account_rendered(
    summary: &AccountSummaryData,
    account_hash: &str,
) -> Result<(), AppError> {
    if summary.accounts.is_empty() {
        return Err(AppError::AccountValidation(format!(
            "account '{account_hash}' resolved but no account summary data was available"
        )));
    }

    Ok(())
}

/// Pure helper that builds an [`AccountSummaryData`] from pre-fetched API data.
///
/// Joins accounts to hashes via `account_number`, enriches with user preferences,
/// and extracts balance summaries based on account type (margin vs cash).
///
/// Position output uses compact objects with all curated fields.
#[must_use]
pub(crate) fn render_summary_from_data(
    accounts: &[Account],
    hashes: &[AccountNumberHash],
    prefs: &[UserPreferenceAccount],
    with_positions: bool,
) -> AccountSummaryData {
    let rows = accounts
        .iter()
        .filter_map(|account| {
            let fields = extract_account_fields(account, with_positions)?;
            let AccountFields {
                account_number,
                variant_type,
                is_closing_only_restricted,
                is_day_trader,
                balances,
                positions,
            } = fields;
            let hash_value = find_hash_value(account_number.as_deref(), hashes)?;
            let pref = matching_preference(account_number.as_deref(), prefs);
            let mut row = build_account_row(hash_value, pref);
            if row.nickname.is_none() {
                row.nickname = row
                    .account_type
                    .clone()
                    .or_else(|| Some(variant_type.to_string()));
            }
            row.is_closing_only_restricted = is_closing_only_restricted;
            row.is_day_trader = is_day_trader;
            row.balances = balances;
            row.positions = positions;
            Some(row)
        })
        .collect();

    AccountSummaryData { accounts: rows }
}

/// Extracts the account number, balance summary, and optional positions from an
/// [`Account`] by dispatching on the securities account variant.
///
/// When position output is requested, positions use compact objects via
/// [`compact_position`].
///
/// Returns `None` when the account has no `securities_account` field.
#[must_use]
fn extract_account_fields(account: &Account, with_positions: bool) -> Option<AccountFields> {
    match account.securities_account.as_ref()? {
        SecuritiesAccount::Margin(margin) => {
            let balances = account_balances_margin(
                margin.current_balances.as_ref(),
                margin.initial_balances.as_ref(),
            );
            let positions = with_positions
                .then(|| format_positions(&margin.positions))
                .flatten();
            Some(AccountFields {
                account_number: margin.account_number.clone(),
                variant_type: "MARGIN",
                is_closing_only_restricted: margin.is_closing_only_restricted,
                is_day_trader: margin.is_day_trader,
                balances,
                positions,
            })
        }
        SecuritiesAccount::Cash(cash) => {
            let balances = account_balances_cash(
                cash.current_balances.as_ref(),
                cash.initial_balances.as_ref(),
            );
            let positions = with_positions
                .then(|| format_positions(&cash.positions))
                .flatten();
            Some(AccountFields {
                account_number: cash.account_number.clone(),
                variant_type: "CASH",
                is_closing_only_restricted: cash.is_closing_only_restricted,
                is_day_trader: cash.is_day_trader,
                balances,
                positions,
            })
        }
    }
}

fn account_balances_margin(
    current: Option<&MarginBalance>,
    initial: Option<&MarginInitialBalance>,
) -> Option<AccountBalances> {
    (current.is_some() || initial.is_some())
        .then(|| AccountBalances::Margin(margin_balance_summary(current, initial)))
}

fn account_balances_cash(
    current: Option<&CashBalance>,
    initial: Option<&CashInitialBalance>,
) -> Option<AccountBalances> {
    (current.is_some() || initial.is_some())
        .then(|| AccountBalances::Cash(cash_balance_summary(current, initial)))
}

/// Maps margin balance snapshots to a compact [`MarginBalanceSummary`].
#[must_use]
fn margin_balance_summary(
    balance: Option<&MarginBalance>,
    initial: Option<&MarginInitialBalance>,
) -> MarginBalanceSummary {
    let cash_balance = balance.and_then(|balance| balance.cash_balance);
    let true_cash = cash_balance
        .or_else(|| initial.and_then(|balance| balance.total_cash))
        .or_else(|| initial.and_then(|balance| balance.cash_balance));
    MarginBalanceSummary {
        true_cash,
        true_cash_status: true_cash_status(true_cash),
        cash_balance,
        cash_available_for_trading: balance.and_then(|balance| balance.available_funds),
        cash_available_for_withdrawal: balance
            .and_then(|balance| balance.available_funds_non_marginable_trade),
        buying_power: balance.and_then(|balance| balance.buying_power),
        stock_buying_power: balance.and_then(|balance| balance.stock_buying_power),
        option_buying_power: balance.and_then(|balance| balance.option_buying_power),
        equity: balance.and_then(|balance| balance.equity),
    }
}

/// Maps cash balance snapshots to a compact [`CashBalanceSummary`].
#[must_use]
fn cash_balance_summary(
    balance: Option<&CashBalance>,
    initial: Option<&CashInitialBalance>,
) -> CashBalanceSummary {
    let true_cash = balance
        .and_then(|balance| balance.cash_balance.or(balance.total_cash))
        .or_else(|| initial.and_then(|balance| balance.cash_balance));
    CashBalanceSummary {
        true_cash,
        true_cash_status: true_cash_status(true_cash),
        cash_balance: balance.and_then(|balance| balance.cash_balance),
        cash_available_for_trading: balance.and_then(|balance| balance.cash_available_for_trading),
        cash_available_for_withdrawal: balance
            .and_then(|balance| balance.cash_available_for_withdrawal),
        total_cash: balance.and_then(|balance| balance.total_cash),
    }
}

fn true_cash_status(true_cash: Option<schwab::Number>) -> TrueCashStatus {
    if true_cash.is_some() {
        TrueCashStatus::Verified
    } else {
        TrueCashStatus::Unavailable
    }
}

/// Formats positions as compact objects with all curated fields.
///
/// Returns `None` when positions are absent, preserving the distinction between
/// "not requested" and "empty list".
#[must_use]
fn format_positions(positions: &Option<Vec<schwab::Position>>) -> Option<Value> {
    let pos = positions.as_ref()?;
    Some(Value::Array(pos.iter().map(compact_position).collect()))
}

/// Builds a compact JSON value from a single position, including only fields
/// useful for an account summary.
#[must_use]
fn compact_position(position: &schwab::Position) -> Value {
    let mut map = serde_json::Map::new();

    if let Some(instrument) = position.instrument.as_ref().map(instrument_summary) {
        if let Some(symbol) = instrument.symbol {
            map.insert("symbol".to_string(), serde_json::json!(symbol));
        }
        if let Some(cusip) = instrument.cusip {
            map.insert("cusip".to_string(), serde_json::json!(cusip));
        }
        if let Some(instrument_id) = instrument.instrument_id {
            map.insert(
                "instrument_id".to_string(),
                serde_json::json!(instrument_id),
            );
        }
        if let Some(description) = instrument.description {
            map.insert("description".to_string(), serde_json::json!(description));
        }
        if let Some(asset_type) = instrument.asset_type {
            map.insert("asset_type".to_string(), serde_json::json!(asset_type));
        }
    }

    if let Some(qty) = position.long_quantity {
        map.insert("long_quantity".to_string(), serde_json::json!(qty));
    }
    if let Some(qty) = position.short_quantity {
        map.insert("short_quantity".to_string(), serde_json::json!(qty));
    }
    if let Some(price) = position.average_price {
        map.insert("average_price".to_string(), serde_json::json!(price));
    }
    if let Some(value) = position.market_value {
        map.insert("market_value".to_string(), serde_json::json!(value));
    }
    if let Some(pnl) = position.current_day_profit_loss {
        map.insert(
            "current_day_profit_loss".to_string(),
            serde_json::json!(pnl),
        );
    }
    if let Some(pnl_pct) = position.current_day_profit_loss_percentage {
        map.insert(
            "current_day_profit_loss_percentage".to_string(),
            serde_json::json!(pnl_pct),
        );
    }

    Value::Object(map)
}

struct InstrumentSummary {
    symbol: Option<String>,
    cusip: Option<String>,
    instrument_id: Option<i64>,
    description: Option<String>,
    asset_type: Option<String>,
}

/// Normalizes Schwab account instrument variants into identifier fields for a
/// compact position row.
#[must_use]
fn instrument_summary(instrument: &AccountsInstrument) -> InstrumentSummary {
    macro_rules! extract {
        ($value:expr) => {
            InstrumentSummary {
                symbol: $value.symbol.clone(),
                cusip: $value.cusip.clone(),
                instrument_id: $value.instrument_id,
                description: $value.description.clone(),
                asset_type: $value
                    .asset_type
                    .as_ref()
                    .map(|asset_type| format!("{asset_type:?}")),
            }
        };
    }

    match instrument {
        AccountsInstrument::Option(value) => extract!(value),
        AccountsInstrument::FixedIncome(value) => extract!(value),
        AccountsInstrument::CashEquivalent(value) => extract!(value),
        AccountsInstrument::Equity(value) => extract!(value),
        AccountsInstrument::MutualFund(value) => extract!(value),
    }
}

/// Finds the hash value for an account number from the account numbers list.
#[must_use]
fn find_hash_value(account_number: Option<&str>, hashes: &[AccountNumberHash]) -> Option<String> {
    let account_number = account_number?;
    hashes
        .iter()
        .find(|h| h.account_number.as_deref() == Some(account_number))
        .and_then(|h| h.hash_value.clone())
}

/// Resolves the default account hash from pre-fetched data.
///
/// Pure helper: prefers the account marked as `primary_account == true`,
/// falls back to the first account in the hash list.
///
/// # Errors
///
/// Returns `AppError::AccountValidation` when no accounts are available.
#[cfg(test)]
pub(crate) fn resolve_default_account_hash_from_data(
    hashes: &[AccountNumberHash],
    prefs: &[UserPreferenceAccount],
) -> Result<String, AppError> {
    let rows = joined_account_rows(hashes, prefs);

    // Primary account wins; first account is the fallback.
    if let Some(row) = rows.iter().find(|r| r.primary_account == Some(true)) {
        return Ok(row.account_hash.clone());
    }

    rows.into_iter()
        .next()
        .map(|r| r.account_hash)
        .ok_or_else(|| AppError::AccountValidation("no accounts found".to_string()))
}

/// Resolves an account selector to the canonical Schwab account hash.
///
/// Exact hash matches take precedence over exact nickname matches. Raw account
/// numbers are used only as the join key between Schwab API responses and are
/// never returned in the result or validation messages.
///
/// # Errors
///
/// Returns an `AppError` when the selector does not match any account or when a
/// nickname selector matches more than one account. Schwab API failures also
/// return an `AppError`.
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn resolve_account(
    bearer_token: &str,
    selector: &str,
) -> Result<AccountResolveData, AppError> {
    let http = reqwest::Client::new();
    let (hashes, preferences) = tokio::try_join!(
        crate::raw::fetch_account_numbers_with_client(&http, bearer_token),
        crate::raw::fetch_user_preference_with_client(&http, bearer_token),
    )?;
    let prefs = preference_accounts(preferences);

    resolve_account_from_data(&hashes, &prefs, selector)
}

/// Resolves a selector from pre-fetched account hash and preference data.
///
/// This pure helper keeps the matching rules unit-testable without requiring a
/// live Schwab client or credentials.
pub(crate) fn resolve_account_from_data(
    hashes: &[AccountNumberHash],
    prefs: &[UserPreferenceAccount],
    selector: &str,
) -> Result<AccountResolveData, AppError> {
    let rows = joined_account_rows(hashes, prefs);

    if let Some(row) = rows.iter().find(|row| row.account_hash == selector) {
        return Ok(account_resolve_data(row, "hash"));
    }

    let nickname_matches = rows
        .iter()
        .filter(|row| row.nickname.as_deref() == Some(selector))
        .collect::<Vec<_>>();

    match nickname_matches.as_slice() {
        [row] => Ok(account_resolve_data(row, "nickname")),
        [] => Err(AppError::AccountValidation(format!(
            "no account found matching '{selector}'"
        ))),
        matches => Err(AppError::AccountValidation(format!(
            "ambiguous account nickname '{selector}' matched: {}",
            matches
                .iter()
                .map(|row| compact_account_label(row))
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

/// Joins hash records to preference records on raw account number.
///
/// Raw account numbers are borrowed only for this comparison. Rows without a
/// hash value are skipped because they cannot be used as canonical selectors.
#[must_use]
fn joined_account_rows(
    hashes: &[AccountNumberHash],
    prefs: &[UserPreferenceAccount],
) -> Vec<AccountRow> {
    hashes
        .iter()
        .filter_map(|hash| {
            let hash_value = hash.hash_value.clone()?;
            let pref = matching_preference(hash.account_number.as_deref(), prefs);
            Some(build_account_row(hash_value, pref))
        })
        .collect()
}

/// Finds the preference account that shares the hash entry account number.
#[must_use]
fn matching_preference<'a>(
    account_number: Option<&str>,
    prefs: &'a [UserPreferenceAccount],
) -> Option<&'a UserPreferenceAccount> {
    let account_number = account_number?;
    prefs
        .iter()
        .find(|pref| pref.account_number.as_deref() == Some(account_number))
}

/// Converts a joined account row into resolver output with match metadata.
#[must_use]
fn account_resolve_data(row: &AccountRow, matched_by: &str) -> AccountResolveData {
    AccountResolveData {
        account_hash: row.account_hash.clone(),
        matched_by: matched_by.to_string(),
        nickname: row.nickname.clone(),
        display_account_id: row.display_account_id.clone(),
        primary_account: row.primary_account,
        account_type: row.account_type.clone(),
    }
}

/// Formats an ambiguous account match without exposing raw account numbers.
#[must_use]
fn compact_account_label(row: &AccountRow) -> String {
    let nickname = row.nickname.as_deref().unwrap_or("<no nickname>");
    let display_account_id = row
        .display_account_id
        .as_deref()
        .unwrap_or("<no display id>");
    format!("{nickname} ({display_account_id})")
}

#[cfg(test)]
mod tests;

pub mod cli;
