//! Raw Schwab API requests with response normalization.
//!
//! The `schwab` crate deserializes API responses directly into typed structs,
//! which fails when the Schwab API returns unexpected formats. This module
//! bypasses the crate's typed deserialization to normalize known quirks:
//!
//! - **Object-wrapped arrays** (GitHub #17): account endpoints sometimes
//!   return `{"key": [...]}` instead of a bare `[...]`.
//! - **Bare user preference objects** (GitHub #46): the user preference endpoint
//!   can return a single object where the typed crate expects a one-item array.
//! - **Linked-account envelopes** (GitHub #62): the account-number endpoint can
//!   return linked account hashes under a named object field.
//! - **Boolean `false` for absent numerics** (GitHub #18): some numeric fields
//!   serialize as `false` instead of `null` or `0`.
//! - **New preview severities** (GitHub #89): the preview endpoint can return
//!   validation severities before the typed `schwab` crate models know them.

use schwab::{Account, AccountNumberHash, UserPreference};
use serde::Serialize;
use serde_json::Value;

#[cfg(test)]
use std::cell::RefCell;

use crate::error::AppError;

/// Schwab Trader API accounts endpoint.
#[cfg(not(test))]
const ACCOUNTS_URL: &str = "https://api.schwabapi.com/trader/v1/accounts";

/// Schwab Trader API account number hash endpoint.
#[cfg(not(test))]
const ACCOUNT_NUMBERS_URL: &str = "https://api.schwabapi.com/trader/v1/accounts/accountNumbers";

/// Schwab Trader API user preference endpoint.
#[cfg(not(test))]
const USER_PREFERENCE_URL: &str = "https://api.schwabapi.com/trader/v1/userPreference";

/// Account number response fields that can contain linked account hashes.
const ACCOUNT_NUMBER_ARRAY_FIELDS: &[&str] = &["accounts", "accountNumbers", "linkedAccounts"];

/// Schwab Trader API cross-account orders endpoint.
#[cfg(not(test))]
const ORDERS_URL: &str = "https://api.schwabapi.com/trader/v1/orders";

/// Schwab Trader API accounts endpoint prefix for per-account order listing.
const ACCOUNT_ORDERS_URL_PREFIX: &str = "https://api.schwabapi.com/trader/v1/accounts";

/// Stable warning code for non-fatal preview warnings.
const PREVIEW_WARNING_CODE: &str = "order.preview_warning";

#[cfg(test)]
thread_local! {
    static RAW_URL_PREFIX_FOR_TESTS: RefCell<Option<String>> = const { RefCell::new(None) };
    static PREVIEW_ORDER_URL_PREFIX_FOR_TESTS: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[cfg(test)]
pub(crate) struct RawUrlPrefixGuard {
    previous: Option<String>,
}

#[cfg(test)]
impl Drop for RawUrlPrefixGuard {
    fn drop(&mut self) {
        RAW_URL_PREFIX_FOR_TESTS.with_borrow_mut(|prefix| {
            *prefix = self.previous.take();
        });
    }
}

#[cfg(test)]
pub(crate) fn set_raw_url_prefix_for_tests(prefix: String) -> RawUrlPrefixGuard {
    let previous =
        RAW_URL_PREFIX_FOR_TESTS.with_borrow_mut(|override_prefix| override_prefix.replace(prefix));
    RawUrlPrefixGuard { previous }
}

#[cfg(test)]
pub(crate) struct PreviewOrderUrlPrefixGuard {
    previous: Option<String>,
}

#[cfg(test)]
impl Drop for PreviewOrderUrlPrefixGuard {
    fn drop(&mut self) {
        PREVIEW_ORDER_URL_PREFIX_FOR_TESTS.with_borrow_mut(|prefix| {
            *prefix = self.previous.take();
        });
    }
}

#[cfg(test)]
pub(crate) fn set_preview_order_url_prefix_for_tests(prefix: String) -> PreviewOrderUrlPrefixGuard {
    let previous = PREVIEW_ORDER_URL_PREFIX_FOR_TESTS
        .with_borrow_mut(|override_prefix| override_prefix.replace(prefix));
    PreviewOrderUrlPrefixGuard { previous }
}

/// Warning emitted when an order activity contains an unrecognized enum value.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct OrderActivityWarning {
    /// Stable warning code for machine readers.
    pub(crate) code: &'static str,
    /// JSON field that contained the unrecognized value.
    pub(crate) field: &'static str,
    /// The unrecognized Schwab enum value, without account or order details.
    pub(crate) value: String,
    /// Count of activity entries containing this field/value pair.
    pub(crate) count: usize,
}

/// Non-fatal warning emitted by the Schwab preview endpoint.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewWarning {
    /// Stable warning code for machine readers.
    pub(crate) code: &'static str,
    /// Schwab validation severity, usually `WARN`.
    pub(crate) severity: String,
    /// Human-readable validation message, when Schwab provides one.
    pub(crate) message: Option<String>,
    /// Human-readable activity message, when Schwab provides one.
    pub(crate) activity_message: Option<String>,
    /// Schwab validation rule identifier, when present.
    pub(crate) validation_rule_name: Option<String>,
}

/// Parameters for raw order list requests.
#[derive(Debug, Clone, Copy)]
pub(crate) struct OrderListQuery<'a> {
    /// Inclusive start instant in RFC3339 format.
    pub(crate) from_entered_time: &'a str,
    /// Inclusive end instant in RFC3339 format.
    pub(crate) to_entered_time: &'a str,
    /// Optional maximum number of orders to return.
    pub(crate) max_results: Option<u32>,
    /// Optional Schwab order status filter.
    pub(crate) status: Option<&'a str>,
}

/// Fetches accounts using a caller-provided HTTP client.
///
/// Reusing one client lets callers preserve connection pooling when multiple
/// raw Schwab requests are made as part of one command.
///
/// # Errors
///
/// Returns an error when the HTTP request fails, the server returns a
/// non-success status, or the normalized JSON cannot be deserialized into
/// `Vec<Account>`.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn fetch_accounts_with_client(
    http: &reqwest::Client,
    bearer_token: &str,
    fields: Option<&str>,
) -> Result<Vec<Account>, AppError> {
    let accounts_url = accounts_url();
    let mut request = http.get(accounts_url).bearer_auth(bearer_token);

    if let Some(fields) = fields {
        request = request.query(&[("fields", fields)]);
    }

    let response = request.send().await.map_err(schwab::Error::Request)?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.map_err(schwab::Error::Request)?;
        return Err(schwab::Error::HttpStatus { status, body }.into());
    }

    let text = response.text().await.map_err(schwab::Error::Request)?;
    let value: Value = serde_json::from_str(&text)?;

    let array = unwrap_accounts_array(value);
    let normalized = normalize_false_to_null(array);

    Ok(serde_json::from_value(normalized)?)
}

/// Fetches account number hashes using a caller-provided HTTP client.
///
/// Reusing one client lets callers preserve connection pooling when multiple
/// raw Schwab requests are made as part of one command.
///
/// # Errors
///
/// Returns an error when the HTTP request fails, the server returns a
/// non-success status, or the normalized JSON cannot be deserialized into
/// `Vec<AccountNumberHash>`.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn fetch_account_numbers_with_client(
    http: &reqwest::Client,
    bearer_token: &str,
) -> Result<Vec<AccountNumberHash>, AppError> {
    let account_numbers_url = account_numbers_url();
    let value = fetch_json_with_client(http, &account_numbers_url, bearer_token).await?;
    let array = account_numbers_array(value)?;

    Ok(serde_json::from_value(array)?)
}

/// Fetches user preferences using a caller-provided HTTP client.
///
/// Reusing one client lets callers preserve connection pooling when multiple
/// raw Schwab requests are made as part of one command.
///
/// # Errors
///
/// Returns an error when the HTTP request fails, the server returns a
/// non-success status, or the normalized JSON cannot be deserialized into
/// `Vec<UserPreference>`.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn fetch_user_preference_with_client(
    http: &reqwest::Client,
    bearer_token: &str,
) -> Result<Vec<UserPreference>, AppError> {
    let user_preference_url = user_preference_url();
    let value = fetch_json_with_client(http, &user_preference_url, bearer_token).await?;
    let array = normalize_user_preference_response(value);

    Ok(serde_json::from_value(array)?)
}

/// Fetches order list JSON directly from Schwab without typed order decoding.
///
/// Schwab occasionally adds order activity variants before the `schwab` crate's
/// typed models know about them. Returning raw JSON keeps read-only order
/// listing resilient while still using the same bearer token and endpoint
/// semantics as the typed client.
///
/// # Errors
///
/// Returns an error when the HTTP request fails, Schwab returns a non-success
/// status, or the body is not valid JSON.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn fetch_order_list(
    bearer_token: &str,
    account_hash: Option<&str>,
    query: &OrderListQuery<'_>,
) -> Result<Value, AppError> {
    let url = account_hash.map_or_else(orders_url, |hash| {
        format!("{}/{hash}/orders", account_orders_url_prefix())
    });

    fetch_order_json_query(&url, bearer_token, query).await
}

/// Fetches transaction list JSON directly from Schwab.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn fetch_transaction_list(
    bearer_token: &str,
    account_hash: &str,
    params: &[(&str, &str)],
) -> Result<Value, AppError> {
    let url = format!(
        "{}/{account_hash}/transactions",
        account_orders_url_prefix()
    );
    fetch_json_query(&url, bearer_token, params).await
}

/// Fetches one transaction JSON directly from Schwab.
pub(crate) async fn fetch_transaction_by_id(
    bearer_token: &str,
    account_hash: &str,
    transaction_id: i64,
) -> Result<Value, AppError> {
    let url = format!(
        "{}/{account_hash}/transactions/{transaction_id}",
        account_orders_url_prefix()
    );
    fetch_json_with_client(&reqwest::Client::new(), &url, bearer_token).await
}

#[cfg(test)]
fn raw_url_prefix() -> String {
    RAW_URL_PREFIX_FOR_TESTS
        .with_borrow(Clone::clone)
        .unwrap_or_else(|| "https://api.schwabapi.com/trader/v1".to_string())
}

#[must_use]
#[cfg(test)]
fn accounts_url() -> String {
    format!("{}/accounts", raw_url_prefix())
}

#[must_use]
#[cfg(not(test))]
fn accounts_url() -> String {
    ACCOUNTS_URL.to_string()
}

#[must_use]
#[cfg(test)]
fn account_numbers_url() -> String {
    format!("{}/accounts/accountNumbers", raw_url_prefix())
}

#[must_use]
#[cfg(not(test))]
fn account_numbers_url() -> String {
    ACCOUNT_NUMBERS_URL.to_string()
}

#[must_use]
#[cfg(test)]
fn user_preference_url() -> String {
    format!("{}/userPreference", raw_url_prefix())
}

#[must_use]
#[cfg(not(test))]
fn user_preference_url() -> String {
    USER_PREFERENCE_URL.to_string()
}

#[must_use]
#[cfg(test)]
fn orders_url() -> String {
    format!("{}/orders", raw_url_prefix())
}

#[must_use]
#[cfg(not(test))]
fn orders_url() -> String {
    ORDERS_URL.to_string()
}

#[must_use]
#[cfg(test)]
fn account_orders_url_prefix() -> String {
    format!("{}/accounts", raw_url_prefix())
}

#[must_use]
#[cfg(not(test))]
fn account_orders_url_prefix() -> String {
    ACCOUNT_ORDERS_URL_PREFIX.to_string()
}

/// Previews an order as raw JSON using a caller-provided HTTP client.
///
/// Schwab can return preview validation severities before the `schwab` crate's
/// typed enum knows about them. Returning raw JSON keeps preview workflows
/// resilient while still using the documented endpoint and bearer token.
///
/// Reusing one client lets callers preserve connection pooling when multiple
/// raw Schwab requests are made as part of one command.
///
/// # Errors
///
/// Returns an error when the order cannot be serialized, the HTTP request fails,
/// Schwab returns a non-success status, or the body is not valid JSON.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn preview_order_with_client<T: Serialize + ?Sized>(
    http: &reqwest::Client,
    bearer_token: &str,
    account_hash: &str,
    order: &T,
) -> Result<Value, AppError> {
    preview_order_at_url(http, &preview_order_url(account_hash), bearer_token, order).await
}

/// Returns the Schwab preview endpoint URL for an account hash.
#[must_use]
fn preview_order_url(account_hash: &str) -> String {
    format!("{}/{account_hash}/previewOrder", preview_order_url_prefix())
}

#[cfg(test)]
fn preview_order_url_prefix() -> String {
    PREVIEW_ORDER_URL_PREFIX_FOR_TESTS
        .with_borrow(Clone::clone)
        .unwrap_or_else(|| ACCOUNT_ORDERS_URL_PREFIX.to_string())
}

#[cfg(not(test))]
fn preview_order_url_prefix() -> &'static str {
    ACCOUNT_ORDERS_URL_PREFIX
}

/// Previews an order as raw JSON against an explicit URL.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn preview_order_at_url<T: Serialize + ?Sized>(
    http: &reqwest::Client,
    url: &str,
    bearer_token: &str,
    order: &T,
) -> Result<Value, AppError> {
    let body = serde_json::to_value(order).map_err(schwab::Error::Encode)?;

    post_json_with_client(http, url, bearer_token, &body).await
}

/// Extracts sanitized non-fatal warnings from a raw preview response.
#[must_use]
pub(crate) fn preview_warnings(value: &Value) -> Vec<PreviewWarning> {
    value
        .get("orderValidationResult")
        .and_then(|result| result.get("warns"))
        .and_then(Value::as_array)
        .map(|warns| warns.iter().map(preview_warning).collect())
        .unwrap_or_default()
}

/// Fetches a Schwab API URL as raw JSON using a caller-provided HTTP client.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn fetch_json_with_client(
    http: &reqwest::Client,
    url: &str,
    bearer_token: &str,
) -> Result<Value, AppError> {
    let response = http
        .get(url)
        .bearer_auth(bearer_token)
        .send()
        .await
        .map_err(schwab::Error::Request)?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.map_err(schwab::Error::Request)?;
        return Err(schwab::Error::HttpStatus { status, body }.into());
    }

    let text = response.text().await.map_err(schwab::Error::Request)?;

    Ok(serde_json::from_str(&text)?)
}

/// Posts JSON to a Schwab API URL and returns the raw JSON response body.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn post_json_with_client(
    http: &reqwest::Client,
    url: &str,
    bearer_token: &str,
    body: &Value,
) -> Result<Value, AppError> {
    let response = http
        .post(url)
        .bearer_auth(bearer_token)
        .header(reqwest::header::ACCEPT, "application/json")
        .json(body)
        .send()
        .await
        .map_err(schwab::Error::Request)?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.map_err(schwab::Error::Request)?;
        return Err(schwab::Error::HttpStatus { status, body }.into());
    }

    let text = response.text().await.map_err(schwab::Error::Request)?;

    Ok(serde_json::from_str(&text)?)
}

/// Fetches a Schwab order list URL as raw JSON with query parameters.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn fetch_order_json_query(
    url: &str,
    bearer_token: &str,
    query: &OrderListQuery<'_>,
) -> Result<Value, AppError> {
    let max_results = query.max_results.map(|value| value.to_string());
    let mut params = vec![
        ("fromEnteredTime", query.from_entered_time),
        ("toEnteredTime", query.to_entered_time),
    ];

    if let Some(max_results) = max_results.as_deref() {
        params.push(("maxResults", max_results));
    }
    if let Some(status) = query.status {
        params.push(("status", status));
    }

    fetch_json_query(url, bearer_token, &params).await
}

/// Fetches a Schwab URL as raw JSON with query parameters.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn fetch_json_query(
    url: &str,
    bearer_token: &str,
    params: &[(&str, &str)],
) -> Result<Value, AppError> {
    let response = reqwest::Client::new()
        .get(url)
        .bearer_auth(bearer_token)
        .query(params)
        .send()
        .await
        .map_err(schwab::Error::Request)?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.map_err(schwab::Error::Request)?;
        return Err(schwab::Error::HttpStatus { status, body }.into());
    }

    let text = response.text().await.map_err(schwab::Error::Request)?;

    Ok(serde_json::from_str(&text)?)
}

/// Converts one raw Schwab preview warning object into sanitized output.
///
/// The returned warning keeps only the stable code, normalized severity, and
/// optional human-readable validation fields that are safe to show to agents.
#[must_use]
fn preview_warning(value: &Value) -> PreviewWarning {
    PreviewWarning {
        code: PREVIEW_WARNING_CODE,
        severity: preview_warning_severity(value).to_string(),
        message: optional_string(value, "message"),
        activity_message: optional_string(value, "activityMessage"),
        validation_rule_name: optional_string(value, "validationRuleName"),
    }
}

/// Returns the best available warning severity label.
#[must_use]
fn preview_warning_severity(value: &Value) -> &str {
    value
        .get("originalSeverity")
        .or_else(|| value.get("overrideSeverity"))
        .and_then(Value::as_str)
        .unwrap_or("WARN")
}

/// Extracts an optional string field from a JSON object.
#[must_use]
fn optional_string(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

/// Normalizes order list responses into a bare order array.
///
/// Schwab's documented order-list endpoints return arrays. This also accepts
/// named envelopes to match the defensive normalization style used elsewhere in
/// this module.
#[must_use]
pub(crate) fn normalize_order_list_response(value: Value) -> Value {
    unwrap_array_fields(value, &["orders", "orderList"])
}

/// Finds unknown order activity enum values without exposing order identifiers.
#[must_use]
pub(crate) fn order_activity_warnings(value: &Value) -> Vec<OrderActivityWarning> {
    let mut warnings = std::collections::BTreeMap::<(&'static str, String), usize>::new();
    collect_order_activity_warnings(value, &mut warnings);

    warnings
        .into_iter()
        .map(|((field, value), count)| OrderActivityWarning {
            code: "order.activity_unknown_variant",
            field,
            value,
            count,
        })
        .collect()
}

/// Recursively scans order JSON for unknown activity enum values.
fn collect_order_activity_warnings(
    value: &Value,
    warnings: &mut std::collections::BTreeMap<(&'static str, String), usize>,
) {
    match value {
        Value::Object(map) => {
            if let Some(activities) = map.get("orderActivityCollection").and_then(Value::as_array) {
                for activity in activities {
                    collect_activity_warning(
                        activity,
                        "activityType",
                        is_known_activity_type,
                        warnings,
                    );
                    collect_activity_warning(
                        activity,
                        "executionType",
                        is_known_execution_type,
                        warnings,
                    );
                }
            }

            for child in map.values() {
                collect_order_activity_warnings(child, warnings);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_order_activity_warnings(item, warnings);
            }
        }
        _ => {}
    }
}

/// Adds a warning for a single activity enum field when its value is unknown.
fn collect_activity_warning(
    activity: &Value,
    field: &'static str,
    known: fn(&str) -> bool,
    warnings: &mut std::collections::BTreeMap<(&'static str, String), usize>,
) {
    let Some(value) = activity.get(field).and_then(Value::as_str) else {
        return;
    };

    if known(value) {
        return;
    }

    *warnings.entry((field, value.to_string())).or_default() += 1;
}

/// Returns true for currently known Schwab order activity types.
fn is_known_activity_type(value: &str) -> bool {
    matches!(value, "EXECUTION" | "ORDER_ACTION")
}

/// Returns true for currently known Schwab execution activity types.
fn is_known_execution_type(value: &str) -> bool {
    matches!(value, "FILL" | "CANCELED")
}

/// Extracts the accounts array from a potential object wrapper.
///
/// The Schwab accounts endpoint sometimes returns a single-key object wrapper
/// around the array instead of a bare JSON array. This function handles both:
///
/// - Already an array: returned unchanged.
/// - Single-key object whose value is an array: the inner array is extracted.
/// - Anything else: returned as-is for the deserializer to report a type error.
#[must_use]
fn unwrap_accounts_array(value: Value) -> Value {
    unwrap_array_field(value, "accounts")
}

/// Extracts an array from a bare array or any matching object field.
#[must_use]
fn unwrap_array_fields(value: Value, fields: &[&str]) -> Value {
    match &value {
        Value::Object(map) => fields
            .iter()
            .find_map(|field| map.get(*field).filter(|value| value.is_array()).cloned())
            .unwrap_or_else(|| unwrap_array_field(value, "")),
        _ => unwrap_array_field(value, ""),
    }
}

/// Extracts linked account hashes from the account-number response.
fn account_numbers_array(value: Value) -> Result<Value, AppError> {
    let array = unwrap_array_fields(value, ACCOUNT_NUMBER_ARRAY_FIELDS);
    if array.is_array() {
        Ok(array)
    } else {
        Err(AppError::AccountResponseShape {
            endpoint: "accountNumbers",
            expected: "a bare array or object field accounts, accountNumbers, or linkedAccounts containing an array",
            shape: describe_json_shape(&array),
        })
    }
}

/// Returns sanitized JSON shape metadata without including values.
#[must_use]
fn describe_json_shape(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(items) => format!("array(len={})", items.len()),
        Value::Object(map) => {
            let fields = map
                .iter()
                .map(|(key, value)| format!("{}:{}", safe_shape_key(key), json_type(value)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("object(len={}, fields=[{fields}])", map.len())
        }
    }
}

/// Returns a safe field label for shape metadata.
#[must_use]
fn safe_shape_key(key: &str) -> &str {
    match key {
        "accountNumbers" | "accounts" | "errors" | "linkedAccounts" | "metadata"
        | "userPreference" | "userPreferences" => key,
        _ => "<redacted>",
    }
}

/// Returns the top-level JSON type name for a value.
#[must_use]
fn json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Extracts an array from a bare array or an object field.
///
/// Handles the historical single-key object wrapper plus newer multi-key
/// envelopes that expose the desired array by name.
#[must_use]
fn unwrap_array_field(value: Value, field: &str) -> Value {
    match &value {
        Value::Array(_) => value,
        Value::Object(map) if map.get(field).is_some_and(Value::is_array) => map[field].clone(),
        Value::Object(map) if map.len() == 1 => {
            // Safe: we just checked len() == 1.
            let inner = map.values().next().unwrap();
            if inner.is_array() {
                inner.clone()
            } else {
                value
            }
        }
        _ => value,
    }
}

/// Normalizes user preference responses into an array.
#[must_use]
fn normalize_user_preference_response(value: Value) -> Value {
    match &value {
        Value::Array(_) => value,
        Value::Object(map)
            if map
                .get("userPreferences")
                .or_else(|| map.get("userPreference"))
                .is_some_and(Value::is_array) =>
        {
            map.get("userPreferences")
                .or_else(|| map.get("userPreference"))
                .cloned()
                .unwrap_or(value)
        }
        Value::Object(map) if map.len() == 1 => {
            // Safe: we just checked len() == 1.
            let inner = map.values().next().unwrap();
            if inner.is_array() {
                inner.clone()
            } else {
                Value::Array(vec![value])
            }
        }
        Value::Object(_) => Value::Array(vec![value]),
        _ => value,
    }
}

/// Field names to remove from order API responses before output.
///
/// `accountNumber` is the raw numeric Schwab account number. It is a privacy
/// risk when output is displayed, logged, or forwarded to external tools. The
/// account hash is already present and sufficient for correlation.
const REDACTED_ORDER_FIELDS: &[&str] = &["accountNumber"];

/// Strips all `null`-valued keys from JSON objects, recursively.
///
/// Arrays are traversed element-by-element; array elements that are themselves
/// `null` are kept (only object keys whose value is null are removed). All
/// other scalar values are returned unchanged.
///
/// This reduces token overhead in order output by eliminating the ~16 null
/// fields per order (e.g. `activationPrice`, `cancelTime`,
/// `priceLinkBasis`) that the Schwab API includes when absent.
#[must_use]
pub(crate) fn strip_null_fields(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k, strip_null_fields(v)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(strip_null_fields).collect()),
        other => other,
    }
}

/// Removes privacy-sensitive fields from order JSON, recursively.
///
/// Currently removes `accountNumber` (the raw numeric account number). The
/// account hash is already present in the response and is sufficient for
/// identifying the account without exposing the raw number.
#[must_use]
pub(crate) fn redact_order_fields(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter(|(k, _)| !REDACTED_ORDER_FIELDS.contains(&k.as_str()))
                .map(|(k, v)| (k, redact_order_fields(v)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(redact_order_fields).collect()),
        other => other,
    }
}

/// Sanitizes order API output: strips null fields and redacts sensitive fields.
///
/// Combines [`strip_null_fields`] and [`redact_order_fields`] in a single
/// pipeline. Apply to any `Value` derived from a Schwab order API response
/// before returning it to the caller.
#[must_use]
pub(crate) fn sanitize_order(value: Value) -> Value {
    redact_order_fields(strip_null_fields(value))
}

/// Known boolean field names in the Schwab API response (camelCase).
///
/// These keys carry legitimate `false` values and must not be converted to
/// `null` during normalization.
const BOOLEAN_FIELDS: &[&str] = &["isDayTrader", "isClosingOnlyRestricted"];

/// Recursively replaces `false` with `null` throughout a JSON value tree,
/// except for keys listed in [`BOOLEAN_FIELDS`].
///
/// The Schwab API sometimes serializes absent or zero numeric fields as boolean
/// `false` instead of `null` or `0`. Since the `schwab` crate types use
/// `Option<Number>` for these fields, `false` causes a deserialization error.
///
/// Known boolean fields are preserved so that `false` deserializes as
/// `Some(false)` rather than collapsing to `None`.
///
/// Boolean `true` values are always preserved unchanged.
#[must_use]
fn normalize_false_to_null(value: Value) -> Value {
    match value {
        Value::Bool(false) => Value::Null,
        Value::Array(items) => {
            Value::Array(items.into_iter().map(normalize_false_to_null).collect())
        }
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| {
                    if BOOLEAN_FIELDS.contains(&k.as_str()) {
                        (k, v)
                    } else {
                        (k, normalize_false_to_null(v))
                    }
                })
                .collect(),
        ),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread::JoinHandle;

    use serde_json::json;

    use super::*;

    // --- unwrap_accounts_array ---

    #[test]
    fn unwrap_bare_array_unchanged() {
        let input = json!([{"a": 1}, {"b": 2}]);
        assert_eq!(unwrap_accounts_array(input.clone()), input);
    }

    #[test]
    fn unwrap_single_key_object_extracts_array() {
        let inner = json!([{"a": 1}]);
        let wrapped = json!({"accounts": inner});
        assert_eq!(unwrap_accounts_array(wrapped), inner);
    }

    #[test]
    fn unwrap_single_key_non_array_returns_as_is() {
        let input = json!({"key": "not-an-array"});
        assert_eq!(unwrap_accounts_array(input.clone()), input);
    }

    #[test]
    fn unwrap_multi_key_object_returns_as_is() {
        let input = json!({"a": [1], "b": [2]});
        assert_eq!(unwrap_accounts_array(input.clone()), input);
    }

    #[test]
    fn unwrap_multi_key_accounts_envelope_extracts_accounts() {
        let inner = json!([{"accountNumber": "A1"}]);
        let input = json!({"accounts": inner, "metadata": {"ignored": true}});
        assert_eq!(unwrap_accounts_array(input), inner);
    }

    #[test]
    fn unwrap_scalar_returns_as_is() {
        let input = json!("just a string");
        assert_eq!(unwrap_accounts_array(input.clone()), input);
    }

    // --- normalize_false_to_null ---

    #[test]
    fn normalize_false_becomes_null() {
        assert_eq!(normalize_false_to_null(json!(false)), Value::Null);
    }

    #[test]
    fn normalize_true_preserved() {
        assert_eq!(normalize_false_to_null(json!(true)), json!(true));
    }

    #[test]
    fn normalize_replaces_false_in_object() {
        let input = json!({"balance": false, "name": "test"});
        let expected = json!({"balance": null, "name": "test"});
        assert_eq!(normalize_false_to_null(input), expected);
    }

    #[test]
    fn normalize_preserves_true_in_object() {
        let input = json!({"isDayTrader": true, "balance": false});
        let expected = json!({"isDayTrader": true, "balance": null});
        assert_eq!(normalize_false_to_null(input), expected);
    }

    #[test]
    fn normalize_preserves_false_for_known_boolean_fields() {
        let input = json!({
            "isDayTrader": false,
            "isClosingOnlyRestricted": false,
            "balance": false,
            "equity": false
        });
        let expected = json!({
            "isDayTrader": false,
            "isClosingOnlyRestricted": false,
            "balance": null,
            "equity": null
        });
        assert_eq!(normalize_false_to_null(input), expected);
    }

    #[test]
    fn normalize_handles_nested_structures() {
        let input = json!({
            "outer": {"inner": false, "value": 42},
            "list": [false, true, 1, "text"]
        });
        let expected = json!({
            "outer": {"inner": null, "value": 42},
            "list": [null, true, 1, "text"]
        });
        assert_eq!(normalize_false_to_null(input), expected);
    }

    #[test]
    fn normalize_deeply_nested_false() {
        let input = json!([{"account": {"balances": {"equity": false, "cash": 100.0}}}]);
        let expected = json!([{"account": {"balances": {"equity": null, "cash": 100.0}}}]);
        assert_eq!(normalize_false_to_null(input), expected);
    }

    #[test]
    fn normalize_preserves_existing_nulls() {
        let input = json!({"a": null, "b": false, "c": true});
        let expected = json!({"a": null, "b": null, "c": true});
        assert_eq!(normalize_false_to_null(input), expected);
    }

    #[test]
    fn normalize_empty_structures() {
        assert_eq!(normalize_false_to_null(json!({})), json!({}));
        assert_eq!(normalize_false_to_null(json!([])), json!([]));
    }

    // --- strip_null_fields ---

    #[test]
    fn strip_null_removes_null_keyed_fields() {
        let input = json!({
            "activationPrice": null,
            "cancelTime": null,
            "orderType": "LIMIT",
            "price": 150.0
        });
        let result = strip_null_fields(input);
        assert!(result.get("activationPrice").is_none());
        assert!(result.get("cancelTime").is_none());
        assert_eq!(result["orderType"], "LIMIT");
        assert_eq!(result["price"], 150.0);
    }

    #[test]
    fn strip_null_preserves_array_null_elements() {
        // null elements inside arrays are kept; only object-key nulls are stripped
        let input = json!([null, 1, "text"]);
        assert_eq!(strip_null_fields(input), json!([null, 1, "text"]));
    }

    #[test]
    fn strip_null_recurses_into_nested_objects() {
        let input = json!({
            "instrument": {
                "symbol": "AAPL",
                "maturityDate": null,
                "optionDeliverables": null
            }
        });
        let result = strip_null_fields(input);
        let instrument = &result["instrument"];
        assert_eq!(instrument["symbol"], "AAPL");
        assert!(instrument.get("maturityDate").is_none());
        assert!(instrument.get("optionDeliverables").is_none());
    }

    #[test]
    fn strip_null_recurses_into_array_objects() {
        let input = json!([
            {"orderId": 1, "cancelTime": null},
            {"orderId": 2, "cancelTime": null, "price": 5.0}
        ]);
        let result = strip_null_fields(input);
        let arr = result.as_array().unwrap();
        assert!(arr[0].get("cancelTime").is_none());
        assert!(arr[1].get("cancelTime").is_none());
        assert_eq!(arr[1]["price"], 5.0);
    }

    // --- redact_order_fields ---

    #[test]
    fn redact_removes_account_number() {
        let input = json!({
            "orderId": 12345,
            "accountNumber": "123456789",
            "status": "WORKING"
        });
        let result = redact_order_fields(input);
        assert!(result.get("accountNumber").is_none());
        assert_eq!(result["orderId"], 12345);
        assert_eq!(result["status"], "WORKING");
    }

    #[test]
    fn redact_recurses_into_nested_structures() {
        let input = json!([
            {"orderId": 1, "accountNumber": "111", "orderLegCollection": [{"accountNumber": "111"}]},
            {"orderId": 2, "accountNumber": "222"}
        ]);
        let result = redact_order_fields(input);
        let arr = result.as_array().unwrap();
        assert!(arr[0].get("accountNumber").is_none());
        assert!(arr[1].get("accountNumber").is_none());
        // Nested inside orderLegCollection
        assert!(
            arr[0]["orderLegCollection"][0]
                .get("accountNumber")
                .is_none()
        );
    }

    // --- sanitize_order ---

    #[test]
    fn sanitize_order_strips_nulls_and_redacts_account_number() {
        let input = json!({
            "orderId": 42,
            "accountNumber": "987654321",
            "activationPrice": null,
            "price": 200.0,
            "instrument": {
                "symbol": "AAPL",
                "maturityDate": null,
                "variableRate": null
            }
        });
        let result = sanitize_order(input);
        assert!(result.get("accountNumber").is_none());
        assert!(result.get("activationPrice").is_none());
        assert_eq!(result["orderId"], 42);
        assert_eq!(result["price"], 200.0);
        assert_eq!(result["instrument"]["symbol"], "AAPL");
        assert!(result["instrument"].get("maturityDate").is_none());
        assert!(result["instrument"].get("variableRate").is_none());
    }

    // --- order list activity warnings ---

    #[test]
    fn normalize_order_list_accepts_bare_array() {
        let input = json!([{"orderId": 42}]);

        assert_eq!(normalize_order_list_response(input.clone()), input);
    }

    #[test]
    fn normalize_order_list_accepts_named_envelope() {
        let orders = json!([{"orderId": 42}]);
        let input = json!({"orders": orders, "metadata": {"ignored": true}});

        assert_eq!(normalize_order_list_response(input), orders);
    }

    #[test]
    fn canceled_order_activity_is_known_and_preserved() {
        let input = json!([{
            "orderId": 42,
            "accountNumber": "123456789",
            "orderActivityCollection": [{
                "activityType": "EXECUTION",
                "executionType": "CANCELED",
                "quantity": null
            }]
        }]);

        let sanitized = sanitize_order(input);

        assert_eq!(order_activity_warnings(&sanitized), Vec::new());
        assert!(sanitized[0].get("accountNumber").is_none());
        assert_eq!(
            sanitized[0]["orderActivityCollection"][0]["executionType"],
            "CANCELED"
        );
        assert!(
            sanitized[0]["orderActivityCollection"][0]
                .get("quantity")
                .is_none()
        );
    }

    #[test]
    fn unknown_activity_variants_emit_sanitized_warning_counts() {
        let input = json!([{
            "orderId": 42,
            "accountNumber": "123456789",
            "orderActivityCollection": [
                {"activityType": "EXECUTION", "executionType": "REBOOKED"},
                {"activityType": "EXECUTION", "executionType": "REBOOKED"},
                {"activityType": "BROKER_NOTE"}
            ]
        }]);

        let sanitized = sanitize_order(input);
        let warnings = order_activity_warnings(&sanitized);

        assert_eq!(
            warnings,
            vec![
                OrderActivityWarning {
                    code: "order.activity_unknown_variant",
                    field: "activityType",
                    value: "BROKER_NOTE".to_string(),
                    count: 1,
                },
                OrderActivityWarning {
                    code: "order.activity_unknown_variant",
                    field: "executionType",
                    value: "REBOOKED".to_string(),
                    count: 2,
                },
            ]
        );

        let warning_json = serde_json::to_value(warnings).unwrap();
        assert!(!warning_json.to_string().contains("123456789"));
        assert!(!warning_json.to_string().contains("42"));
    }

    #[test]
    fn preview_warns_emit_sanitized_warnings() {
        let input = json!({
            "orderId": 400001,
            "orderStrategy": {
                "accountNumber": "123456789",
                "orderType": "STOP"
            },
            "orderValidationResult": {
                "warns": [{
                    "message": "Stop orders do not guarantee an execution price.",
                    "activityMessage": "Review stop order risk before submitting.",
                    "originalSeverity": "WARN",
                    "validationRuleName": "STOP_ORDER_RISK"
                }],
                "accepts": [],
                "alerts": [],
                "rejects": [],
                "reviews": []
            }
        });

        let warnings = preview_warnings(&input);

        assert_eq!(
            warnings,
            vec![PreviewWarning {
                code: "order.preview_warning",
                severity: "WARN".to_string(),
                message: Some("Stop orders do not guarantee an execution price.".to_string()),
                activity_message: Some("Review stop order risk before submitting.".to_string()),
                validation_rule_name: Some("STOP_ORDER_RISK".to_string()),
            }]
        );

        let warning_json = serde_json::to_value(warnings).unwrap();
        assert!(!warning_json.to_string().contains("123456789"));
        assert!(!warning_json.to_string().contains("400001"));
    }

    #[test]
    fn preview_warns_default_to_warn_severity() {
        let input = json!({
            "orderValidationResult": {
                "warns": [{"message": "Review before placing."}]
            }
        });

        let warnings = preview_warnings(&input);

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].severity, "WARN");
        assert_eq!(
            warnings[0].message.as_deref(),
            Some("Review before placing.")
        );
    }

    #[test]
    fn preview_warns_use_override_severity_and_omit_non_strings() {
        let input = json!({
            "orderValidationResult": {
                "warns": [{
                    "message": 123,
                    "activityMessage": "Review stop risk.",
                    "overrideSeverity": "WARN",
                    "validationRuleName": null
                }]
            }
        });

        let warnings = preview_warnings(&input);

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].severity, "WARN");
        assert_eq!(warnings[0].message, None);
        assert_eq!(
            warnings[0].activity_message.as_deref(),
            Some("Review stop risk.")
        );
        assert_eq!(warnings[0].validation_rule_name, None);

        let serialized = serde_json::to_value(&warnings[0]).unwrap();
        assert!(serialized.get("message").is_none());
        assert!(serialized.get("validationRuleName").is_none());
    }

    #[test]
    fn preview_warns_missing_or_non_array_warns_is_empty() {
        assert!(preview_warnings(&json!({})).is_empty());
        assert!(preview_warnings(&json!({"orderValidationResult": {}})).is_empty());
        assert!(preview_warnings(&json!({"orderValidationResult": {"warns": false}})).is_empty());
    }

    #[test]
    fn preview_order_url_targets_account_preview_endpoint() {
        assert_eq!(
            preview_order_url("HASH123"),
            "https://api.schwabapi.com/trader/v1/accounts/HASH123/previewOrder"
        );
    }

    #[tokio::test]
    async fn preview_order_with_client_uses_configured_client_and_account_url() {
        let (url, request) = spawn_json_response_at_path(
            "/accounts/HASH123/previewOrder",
            "HTTP/1.1 200 OK",
            r#"{"preview":"accepted"}"#,
        );
        let prefix = url
            .strip_suffix("/HASH123/previewOrder")
            .unwrap()
            .to_string();
        let _preview_url = set_preview_order_url_prefix_for_tests(prefix);
        let http = reqwest::Client::new();

        let result =
            preview_order_with_client(&http, "TOKEN123", "HASH123", &json!({"orderType": "STOP"}))
                .await
                .unwrap();

        assert_eq!(result["preview"], "accepted");
        assert!(
            request
                .join()
                .unwrap()
                .starts_with("POST /accounts/HASH123/previewOrder HTTP/1.1")
        );
    }

    #[tokio::test]
    async fn preview_order_at_url_posts_json_and_parses_response() {
        let (url, request) = spawn_json_response(
            "HTTP/1.1 200 OK",
            r#"{"preview":"accepted","orderValidationResult":{"warns":[]}}"#,
        );
        let http = reqwest::Client::new();

        let result = preview_order_at_url(
            &http,
            &url,
            "TOKEN123",
            &json!({"orderType": "STOP", "quantity": 1}),
        )
        .await
        .unwrap();

        assert_eq!(result["preview"], "accepted");

        let request = request.join().unwrap();
        assert!(request.starts_with("POST /previewOrder HTTP/1.1"));
        assert!(request.contains("authorization: Bearer TOKEN123"));
        assert!(request.contains("accept: application/json"));
        assert!(request.contains(r#"{"orderType":"STOP","quantity":1}"#));
    }

    #[tokio::test]
    async fn preview_order_at_url_returns_http_status_errors() {
        let (url, request) = spawn_json_response("HTTP/1.1 400 Bad Request", r#"{"error":"bad"}"#);
        let http = reqwest::Client::new();

        let err = preview_order_at_url(&http, &url, "TOKEN123", &json!({"orderType": "STOP"}))
            .await
            .unwrap_err();

        assert_eq!(err.code(), "schwab.http_status");
        assert!(err.to_string().contains("400"));
        assert!(request.join().unwrap().contains("POST /previewOrder"));
    }

    fn spawn_json_response(
        status_line: &'static str,
        body: &'static str,
    ) -> (String, JoinHandle<String>) {
        spawn_json_response_at_path("/previewOrder", status_line, body)
    }

    fn spawn_json_response_at_path(
        path: &'static str,
        status_line: &'static str,
        body: &'static str,
    ) -> (String, JoinHandle<String>) {
        spawn_json_method_response_at_path("POST", path, status_line, body)
    }

    fn spawn_json_method_response_at_path(
        method: &'static str,
        path: &'static str,
        status_line: &'static str,
        body: &'static str,
    ) -> (String, JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}{}", listener.local_addr().unwrap(), path);
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 16];

            loop {
                let read = stream.read(&mut buffer).unwrap();
                assert_ne!(read, 0, "client closed before headers were complete");
                request.extend_from_slice(&buffer[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }

            let headers_end = request
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .unwrap()
                + 4;
            let headers = String::from_utf8_lossy(&request[..headers_end]).to_ascii_lowercase();
            let content_length = headers
                .lines()
                .find_map(|line| line.strip_prefix("content-length: "))
                .and_then(|value| value.trim().parse::<usize>().ok())
                .unwrap_or_default();

            while request.len() - headers_end < content_length {
                let read = stream.read(&mut buffer).unwrap();
                assert_ne!(read, 0, "client closed before body was complete");
                request.extend_from_slice(&buffer[..read]);
            }

            let response = format!(
                "{status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();

            let request = String::from_utf8(request).unwrap();
            assert!(request.starts_with(&format!("{method} {path}")));
            request
        });

        (url, handle)
    }

    #[tokio::test]
    async fn fetch_user_preference_with_client_uses_configured_client() {
        let (url, request) =
            spawn_json_method_response_at_path("GET", "/userPreference", "HTTP/1.1 200 OK", "[]");
        let prefix = url.strip_suffix("/userPreference").unwrap().to_string();
        let _raw_url = set_raw_url_prefix_for_tests(prefix);
        let http = reqwest::Client::new();

        let preferences = fetch_user_preference_with_client(&http, "TOKEN123")
            .await
            .unwrap();

        assert!(preferences.is_empty());
        assert!(
            request
                .join()
                .unwrap()
                .contains("authorization: Bearer TOKEN123")
        );
    }

    #[tokio::test]
    async fn fetch_accounts_with_client_uses_configured_client_and_fields() {
        let (url, request) = spawn_json_method_response_at_path(
            "GET",
            "/accounts",
            "HTTP/1.1 200 OK",
            r#"[{"securitiesAccount":{"type":"MARGIN","accountNumber":"A1"}}]"#,
        );
        let prefix = url.strip_suffix("/accounts").unwrap().to_string();
        let _raw_url = set_raw_url_prefix_for_tests(prefix);
        let http = reqwest::Client::new();

        let accounts = fetch_accounts_with_client(&http, "TOKEN123", Some("positions"))
            .await
            .unwrap();

        assert_eq!(accounts.len(), 1);
        let request = request.join().unwrap();
        assert!(request.contains("GET /accounts?fields=positions"));
        assert!(request.contains("authorization: Bearer TOKEN123"));
    }

    #[tokio::test]
    async fn fetch_account_numbers_with_client_uses_configured_client() {
        let (url, request) = spawn_json_method_response_at_path(
            "GET",
            "/accounts/accountNumbers",
            "HTTP/1.1 200 OK",
            r#"[{"accountNumber":"A1","hashValue":"HASH123"}]"#,
        );
        let prefix = url
            .strip_suffix("/accounts/accountNumbers")
            .unwrap()
            .to_string();
        let _raw_url = set_raw_url_prefix_for_tests(prefix);
        let http = reqwest::Client::new();

        let hashes = fetch_account_numbers_with_client(&http, "TOKEN123")
            .await
            .unwrap();

        assert_eq!(hashes[0].hash_value.as_deref(), Some("HASH123"));
        assert!(
            request
                .join()
                .unwrap()
                .contains("GET /accounts/accountNumbers HTTP/1.1")
        );
    }

    #[tokio::test]
    async fn fetch_order_list_builds_cross_account_query() {
        let (url, request) = spawn_json_method_response_at_path(
            "GET",
            "/orders",
            "HTTP/1.1 200 OK",
            r#"[{"orderId":1}]"#,
        );
        let prefix = url.strip_suffix("/orders").unwrap().to_string();
        let _raw_url = set_raw_url_prefix_for_tests(prefix);
        let query = OrderListQuery {
            from_entered_time: "2026-01-01T00:00:00Z",
            to_entered_time: "2026-01-02T00:00:00Z",
            max_results: Some(10),
            status: Some("WORKING"),
        };

        let orders = fetch_order_list("TOKEN123", None, &query).await.unwrap();

        assert_eq!(orders[0]["orderId"], 1);
        let request = request.join().unwrap();
        assert!(request.contains("GET /orders?"));
        assert!(request.contains("fromEnteredTime=2026-01-01T00%3A00%3A00Z"));
        assert!(request.contains("toEnteredTime=2026-01-02T00%3A00%3A00Z"));
        assert!(request.contains("maxResults=10"));
        assert!(request.contains("status=WORKING"));
    }

    #[tokio::test]
    async fn fetch_transaction_list_builds_account_query() {
        let (url, request) = spawn_json_method_response_at_path(
            "GET",
            "/accounts/HASH123/transactions",
            "HTTP/1.1 200 OK",
            r#"[{"transactionId":1}]"#,
        );
        let prefix = url
            .strip_suffix("/accounts/HASH123/transactions")
            .unwrap()
            .to_string();
        let _raw_url = set_raw_url_prefix_for_tests(prefix);

        let transactions = fetch_transaction_list(
            "TOKEN123",
            "HASH123",
            &[
                ("startDate", "2026-01-01T00:00:00Z"),
                ("endDate", "2026-01-02T00:00:00Z"),
                ("types", "TRADE"),
                ("symbol", "AAPL"),
            ],
        )
        .await
        .unwrap();

        assert_eq!(transactions[0]["transactionId"], 1);
        let request = request.join().unwrap();
        assert!(request.contains("GET /accounts/HASH123/transactions?"));
        assert!(request.contains("startDate=2026-01-01T00%3A00%3A00Z"));
        assert!(request.contains("endDate=2026-01-02T00%3A00%3A00Z"));
        assert!(request.contains("types=TRADE"));
        assert!(request.contains("symbol=AAPL"));
        assert!(
            request
                .to_ascii_lowercase()
                .contains("authorization: bearer token123")
        );
    }

    #[tokio::test]
    async fn fetch_transaction_by_id_builds_account_path() {
        let (url, request) = spawn_json_method_response_at_path(
            "GET",
            "/accounts/HASH123/transactions/456",
            "HTTP/1.1 200 OK",
            r#"{"transactionId":456}"#,
        );
        let prefix = url
            .strip_suffix("/accounts/HASH123/transactions/456")
            .unwrap()
            .to_string();
        let _raw_url = set_raw_url_prefix_for_tests(prefix);

        let transaction = fetch_transaction_by_id("TOKEN123", "HASH123", 456)
            .await
            .unwrap();

        assert_eq!(transaction["transactionId"], 456);
        let request = request.join().unwrap();
        assert!(request.contains("GET /accounts/HASH123/transactions/456 HTTP/1.1"));
        assert!(
            request
                .to_ascii_lowercase()
                .contains("authorization: bearer token123")
        );
    }

    #[tokio::test]
    async fn fetch_order_list_builds_account_query() {
        let (url, request) = spawn_json_method_response_at_path(
            "GET",
            "/accounts/HASH123/orders",
            "HTTP/1.1 200 OK",
            r#"[]"#,
        );
        let prefix = url
            .strip_suffix("/accounts/HASH123/orders")
            .unwrap()
            .to_string();
        let _raw_url = set_raw_url_prefix_for_tests(prefix);
        let query = OrderListQuery {
            from_entered_time: "2026-01-01T00:00:00Z",
            to_entered_time: "2026-01-02T00:00:00Z",
            max_results: None,
            status: None,
        };

        let orders = fetch_order_list("TOKEN123", Some("HASH123"), &query)
            .await
            .unwrap();

        assert_eq!(orders, json!([]));
        assert!(
            request
                .join()
                .unwrap()
                .contains("GET /accounts/HASH123/orders?")
        );
    }

    // --- full pipeline ---

    #[test]
    fn pipeline_unwrap_then_normalize() {
        let wrapped = json!({
            "accounts": [{
                "securitiesAccount": {
                    "balance": false,
                    "isDayTrader": false,
                    "isClosingOnlyRestricted": true,
                    "equity": 1000
                }
            }]
        });
        let unwrapped = unwrap_accounts_array(wrapped);
        let normalized = normalize_false_to_null(unwrapped);
        assert_eq!(
            normalized,
            json!([{
                "securitiesAccount": {
                    "balance": null,
                    "isDayTrader": false,
                    "isClosingOnlyRestricted": true,
                    "equity": 1000
                }
            }])
        );
    }

    #[test]
    fn account_numbers_pipeline_deserializes_bare_array() {
        let input = json!([{"accountNumber": "12345678", "hashValue": "HASH123"}]);
        let normalized = unwrap_array_field(input, "accounts");
        let hashes: Vec<AccountNumberHash> = serde_json::from_value(normalized).unwrap();

        assert_eq!(hashes.len(), 1);
        assert_eq!(hashes[0].account_number.as_deref(), Some("12345678"));
        assert_eq!(hashes[0].hash_value.as_deref(), Some("HASH123"));
    }

    #[test]
    fn account_numbers_pipeline_deserializes_accounts_envelope() {
        let input = json!({
            "accounts": [{"accountNumber": "12345678", "hashValue": "HASH123"}],
            "metadata": {"ignored": true}
        });
        let normalized = unwrap_array_field(input, "accounts");
        let hashes: Vec<AccountNumberHash> = serde_json::from_value(normalized).unwrap();

        assert_eq!(hashes.len(), 1);
        assert_eq!(hashes[0].account_number.as_deref(), Some("12345678"));
        assert_eq!(hashes[0].hash_value.as_deref(), Some("HASH123"));
    }

    #[test]
    fn account_numbers_pipeline_deserializes_account_numbers_envelope() {
        let input = json!({
            "accountNumbers": [{"accountNumber": "12345678", "hashValue": "HASH123"}],
            "metadata": {"ignored": true}
        });
        let normalized = account_numbers_array(input).unwrap();
        let hashes: Vec<AccountNumberHash> = serde_json::from_value(normalized).unwrap();

        assert_eq!(hashes.len(), 1);
        assert_eq!(hashes[0].account_number.as_deref(), Some("12345678"));
        assert_eq!(hashes[0].hash_value.as_deref(), Some("HASH123"));
    }

    #[test]
    fn account_numbers_pipeline_deserializes_linked_accounts_envelope() {
        let input = json!({
            "linkedAccounts": [{"accountNumber": "12345678", "hashValue": "HASH123"}],
            "metadata": {"requestId": "ignored"}
        });
        let normalized = account_numbers_array(input).unwrap();
        let hashes: Vec<AccountNumberHash> = serde_json::from_value(normalized).unwrap();

        assert_eq!(hashes.len(), 1);
        assert_eq!(hashes[0].account_number.as_deref(), Some("12345678"));
        assert_eq!(hashes[0].hash_value.as_deref(), Some("HASH123"));
    }

    #[test]
    fn account_numbers_pipeline_reports_sanitized_shape_for_unknown_envelope() {
        let input = json!({
            "unexpected": {"accountNumber": "12345678", "hashValue": "HASH123"},
            "metadata": {"requestId": "ignored"}
        });
        let err = account_numbers_array(input).unwrap_err();

        assert_eq!(err.code(), "account.response_shape");
        let message = err.to_string();
        assert!(message.contains("object(len=2, fields=["));
        assert!(message.contains("metadata:object"));
        assert!(message.contains("<redacted>:object"));
        assert!(!message.contains("unexpected:object"));
        assert!(!message.contains("12345678"));
        assert!(!message.contains("HASH123"));
        assert!(!message.contains("ignored"));
    }

    #[test]
    fn user_preferences_pipeline_deserializes_bare_array() {
        let input = json!([{
            "accounts": [{
                "accountNumber": "12345678",
                "primaryAccount": true,
                "type": "BROKERAGE",
                "nickName": "Trading",
                "displayAcctId": "...5678"
            }],
            "streamerInfo": []
        }]);
        let normalized = normalize_user_preference_response(input);
        let preferences: Vec<UserPreference> = serde_json::from_value(normalized).unwrap();

        assert_eq!(preferences.len(), 1);
        let accounts = preferences[0].accounts.as_ref().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].nick_name.as_deref(), Some("Trading"));
        assert_eq!(accounts[0].display_acct_id.as_deref(), Some("...5678"));
    }

    #[test]
    fn user_preferences_pipeline_deserializes_bare_object() {
        let input = json!({
            "accounts": [{
                "accountNumber": "12345678",
                "primaryAccount": true,
                "type": "BROKERAGE",
                "nickName": "Trading",
                "displayAcctId": "...5678"
            }],
            "streamerInfo": []
        });
        let normalized = normalize_user_preference_response(input);
        let preferences: Vec<UserPreference> = serde_json::from_value(normalized).unwrap();

        assert_eq!(preferences.len(), 1);
        let accounts = preferences[0].accounts.as_ref().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].nick_name.as_deref(), Some("Trading"));
        assert_eq!(accounts[0].display_acct_id.as_deref(), Some("...5678"));
    }

    #[test]
    fn user_preferences_pipeline_deserializes_named_envelope() {
        let input = json!({
            "userPreferences": [{
                "accounts": [{
                    "accountNumber": "12345678",
                    "nickName": "Trading",
                    "displayAcctId": "...5678"
                }],
                "streamerInfo": []
            }],
            "metadata": {"ignored": true}
        });
        let normalized = normalize_user_preference_response(input);
        let preferences: Vec<UserPreference> = serde_json::from_value(normalized).unwrap();

        assert_eq!(preferences.len(), 1);
        let accounts = preferences[0].accounts.as_ref().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].nick_name.as_deref(), Some("Trading"));
        assert_eq!(accounts[0].display_acct_id.as_deref(), Some("...5678"));
    }
}
