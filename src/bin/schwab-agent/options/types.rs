use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::LazyLock;

use schwab::{Number, OptionChain, OptionContract, PutCall};
use serde::Serialize;
use serde_json::{Value, to_value};
use time::{Date, Month, OffsetDateTime};

use crate::error::AppError;

/// Extracts a field from an `Option<T>` by reference, returning `None` if the
/// outer option is `None`. Use `clone` for non-`Copy` fields like `String`.
macro_rules! opt_field {
    ($opt:expr, $field:ident) => {
        $opt.as_ref().and_then(|v| v.$field)
    };
    ($opt:expr, clone $field:ident) => {
        $opt.as_ref().and_then(|v| v.$field.clone())
    };
}

/// Defines one selectable option contract output field.
#[derive(Clone, Copy)]
pub struct FieldDef {
    /// Display alias emitted in the selected columns list.
    pub name: &'static str,
    /// Extracts this field from a raw Schwab option contract.
    pub extractor: fn(&OptionContract) -> Option<Value>,
}

/// Shared compact option chain field set used by table-shaped outputs.
const DEFAULT_OPTION_FIELDS: [&str; 16] = [
    "symbol",
    "expiration",
    "dte",
    "strike",
    "type",
    "bid",
    "ask",
    "mark",
    "last",
    "volume",
    "openInterest",
    "delta",
    "gamma",
    "theta",
    "vega",
    "iv",
];

/// Default compact option chain fields.
pub static CHAIN_FIELDS: [&str; 16] = DEFAULT_OPTION_FIELDS;

/// Default option screen fields.
pub static SCREEN_FIELDS: [&str; 16] = DEFAULT_OPTION_FIELDS;

const FIELD_DEFS: &[(&str, FieldDef)] = &[
    ("expiration", FieldDef::computed("expiration")),
    ("expiry", FieldDef::computed("expiration")),
    ("dte", FieldDef::computed("dte")),
    ("strike", FieldDef::computed("strike")),
    ("type", FieldDef::computed("type")),
    ("contract_type", FieldDef::computed("contract_type")),
    ("cp", FieldDef::computed("contract_type")),
    ("symbol", FieldDef::new("symbol", extract_symbol)),
    (
        "description",
        FieldDef::new("description", extract_description),
    ),
    ("bid", FieldDef::new("bid", extract_bid)),
    ("ask", FieldDef::new("ask", extract_ask)),
    ("mark", FieldDef::new("mark", extract_mark)),
    ("last", FieldDef::new("last", extract_last)),
    ("close", FieldDef::new("close", extract_close)),
    ("highPrice", FieldDef::new("highPrice", extract_high_price)),
    ("lowPrice", FieldDef::new("lowPrice", extract_low_price)),
    ("delta", FieldDef::new("delta", extract_delta)),
    ("gamma", FieldDef::new("gamma", extract_gamma)),
    ("theta", FieldDef::new("theta", extract_theta)),
    ("vega", FieldDef::new("vega", extract_vega)),
    ("rho", FieldDef::new("rho", extract_rho)),
    ("iv", FieldDef::new("iv", extract_iv)),
    ("volatility", FieldDef::new("iv", extract_iv)),
    ("oi", FieldDef::new("oi", extract_oi)),
    ("openInterest", FieldDef::new("openInterest", extract_oi)),
    ("volume", FieldDef::new("volume", extract_volume)),
    ("totalVolume", FieldDef::new("volume", extract_volume)),
    ("itm", FieldDef::new("itm", extract_itm)),
    ("inTheMoney", FieldDef::new("inTheMoney", extract_itm)),
    (
        "theoreticalValue",
        FieldDef::new("theoreticalValue", extract_theoretical_value),
    ),
    (
        "intrinsicValue",
        FieldDef::new("intrinsicValue", extract_intrinsic_value),
    ),
    (
        "extrinsicValue",
        FieldDef::new("extrinsicValue", extract_extrinsic_value),
    ),
    ("timeValue", FieldDef::new("timeValue", extract_time_value)),
    (
        "multiplier",
        FieldDef::new("multiplier", extract_multiplier),
    ),
    (
        "exerciseType",
        FieldDef::new("exerciseType", extract_exercise_type),
    ),
    (
        "settlementType",
        FieldDef::new("settlementType", extract_settlement_type),
    ),
    (
        "expirationType",
        FieldDef::new("expirationType", extract_expiration_type),
    ),
    (
        "percentChange",
        FieldDef::new("percentChange", extract_percent_change),
    ),
    (
        "markChange",
        FieldDef::new("markChange", extract_mark_change),
    ),
    (
        "markPercentChange",
        FieldDef::new("markPercentChange", extract_mark_percent_change),
    ),
    (
        "daysToExpiration",
        FieldDef::new("daysToExpiration", extract_days_to_expiration),
    ),
];

/// All selectable option fields keyed by accepted request name.
pub static ALL_FIELDS: LazyLock<HashMap<&'static str, FieldDef>> =
    LazyLock::new(|| FIELD_DEFS.iter().map(|(name, def)| (*name, *def)).collect());

impl FieldDef {
    /// Creates a field definition backed by a raw contract extractor.
    const fn new(name: &'static str, extractor: fn(&OptionContract) -> Option<Value>) -> Self {
        Self { name, extractor }
    }

    /// Creates a field definition whose value is supplied by [`FlatContract`].
    const fn computed(name: &'static str) -> Self {
        Self {
            name,
            extractor: extract_none,
        }
    }
}

/// Flattened option contract row with pre-extracted selectable values.
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct FlatContract {
    /// Expiration date in `YYYY-MM-DD` format.
    pub expiration: String,
    /// Days to expiration parsed from Schwab's expiration map key.
    pub dte: i32,
    /// Contract strike price.
    pub strike: Number,
    /// Contract side as `CALL` or `PUT`.
    pub contract_type: String,
    /// OCC option symbol.
    pub symbol: Option<Value>,
    /// Human-readable contract description.
    pub description: Option<Value>,
    /// Current bid price.
    pub bid: Option<Value>,
    /// Current ask price.
    pub ask: Option<Value>,
    /// Schwab mark price.
    pub mark: Option<Value>,
    /// Last traded price.
    pub last: Option<Value>,
    /// Previous close price.
    pub close: Option<Value>,
    /// Session high price.
    pub high_price: Option<Value>,
    /// Session low price.
    pub low_price: Option<Value>,
    /// Option delta.
    pub delta: Option<Value>,
    /// Option gamma.
    pub gamma: Option<Value>,
    /// Option theta.
    pub theta: Option<Value>,
    /// Option vega.
    pub vega: Option<Value>,
    /// Option rho.
    pub rho: Option<Value>,
    /// Implied volatility, exposed as the `iv` alias.
    pub iv: Option<Value>,
    /// Open interest, exposed as the `oi` alias.
    pub oi: Option<Value>,
    /// Total volume.
    pub volume: Option<Value>,
    /// Whether the contract is in the money.
    pub itm: Option<Value>,
    /// Schwab theoretical option value.
    pub theoretical_value: Option<Value>,
    /// Intrinsic value.
    pub intrinsic_value: Option<Value>,
    /// Extrinsic value, backed by Schwab's time value.
    pub extrinsic_value: Option<Value>,
    /// Schwab time value.
    pub time_value: Option<Value>,
    /// Option multiplier.
    pub multiplier: Option<Value>,
    /// Exercise type. Schwab option chain contracts do not currently expose this.
    pub exercise_type: Option<Value>,
    /// Settlement type enum serialized with Schwab's wire value.
    pub settlement_type: Option<Value>,
    /// Expiration type enum serialized with Schwab's wire value.
    pub expiration_type: Option<Value>,
    /// Percent price change.
    pub percent_change: Option<Value>,
    /// Mark price change.
    pub mark_change: Option<Value>,
    /// Mark percent price change.
    pub mark_percent_change: Option<Value>,
    /// Schwab's raw days-to-expiration value when present.
    pub days_to_expiration: Option<Value>,
}

/// Flattens Schwab's call and put expiration maps into flat option contract rows.
#[must_use]
pub fn flatten_chain(chain: &OptionChain) -> Vec<FlatContract> {
    let mut contracts = Vec::new();
    collect_exp_date_map(&mut contracts, chain.call_exp_date_map.as_ref(), "CALL");
    collect_exp_date_map(&mut contracts, chain.put_exp_date_map.as_ref(), "PUT");
    contracts
}

/// Sorts contracts by expiration, strike, then call before put.
pub fn sort_contracts(contracts: &mut [FlatContract]) {
    contracts.sort_by(|left, right| {
        left.expiration
            .cmp(&right.expiration)
            .then_with(|| compare_number(&left.strike, &right.strike))
            .then_with(|| compare_contract_type(&left.contract_type, &right.contract_type))
    });
}

/// Projects flattened contracts into a column list and row values.
#[must_use]
pub fn select_fields(
    contracts: &[FlatContract],
    fields: &[&str],
) -> (Vec<String>, Vec<Vec<Value>>) {
    let columns = fields
        .iter()
        .map(|field| {
            ALL_FIELDS
                .get(*field)
                .map_or_else(|| (*field).to_string(), |def| def.name.to_string())
        })
        .collect::<Vec<_>>();
    let rows = contracts
        .iter()
        .map(|contract| {
            fields
                .iter()
                .map(|field| selected_field_value(contract, field))
                .collect()
        })
        .collect();

    (columns, rows)
}

/// Validates that all requested field names are known option field aliases.
pub fn validate_fields(requested: &[String]) -> Result<(), AppError> {
    let unknown = requested
        .iter()
        .filter(|field| !ALL_FIELDS.contains_key(field.as_str()))
        .map(String::as_str)
        .collect::<Vec<_>>();

    if unknown.is_empty() {
        return Ok(());
    }

    Err(AppError::OptionsValidation {
        message: format!(
            "unknown option field(s): {}; hint: available fields: {}",
            unknown.join(", "),
            available_fields().join(", ")
        ),
    })
}

/// Computes calendar days from today's UTC date to `expiration_str`.
#[must_use]
pub fn compute_dte(expiration_str: &str) -> Option<i32> {
    let expiration = parse_date(expiration_str)?;
    let today = OffsetDateTime::now_utc().date();
    i32::try_from((expiration - today).whole_days()).ok()
}

/// Returns true when a contract delta falls within the optional inclusive bounds.
#[must_use]
pub fn filter_by_delta(contract: &FlatContract, min: Option<Number>, max: Option<Number>) -> bool {
    bounded_field(&contract.delta, min, max)
}

/// Returns true when a contract strike matches an exact value or inclusive bounds.
#[must_use]
pub fn filter_by_strike(
    contract: &FlatContract,
    min: Option<Number>,
    max: Option<Number>,
    exact: Option<Number>,
) -> bool {
    let Some(strike) = number_to_f64(contract.strike) else {
        return false;
    };

    if let Some(target) = exact {
        let Some(target) = number_to_f64(target) else {
            return false;
        };
        return (strike - target).abs() <= f64::EPSILON;
    }

    bounded_f64(strike, min, max)
}

/// Returns true when bid is at least `min`.
#[must_use]
pub fn filter_by_bid(contract: &FlatContract, min: Number) -> bool {
    bounded_field(&contract.bid, Some(min), None)
}

/// Returns true when ask is at most `max`.
#[must_use]
pub fn filter_by_ask(contract: &FlatContract, max: Number) -> bool {
    bounded_field(&contract.ask, None, Some(max))
}

/// Returns true when volume is at least `min`.
#[must_use]
pub fn filter_by_volume(contract: &FlatContract, min: Number) -> bool {
    bounded_field(&contract.volume, Some(min), None)
}

/// Returns true when open interest is at least `min`.
#[must_use]
pub fn filter_by_oi(contract: &FlatContract, min: Number) -> bool {
    bounded_field(&contract.oi, Some(min), None)
}

/// Returns true when the bid-ask spread percentage of mark is at most `max`.
#[must_use]
pub fn filter_by_spread_pct(contract: &FlatContract, max: Number) -> bool {
    let Some(bid) = value_to_f64(contract.bid.as_ref()) else {
        return false;
    };
    let Some(ask) = value_to_f64(contract.ask.as_ref()) else {
        return false;
    };
    let Some(mark) = value_to_f64(contract.mark.as_ref()) else {
        return false;
    };
    let Some(max) = number_to_f64(max) else {
        return false;
    };
    if mark.abs() <= f64::EPSILON {
        return false;
    }

    ((ask - bid) / mark) * 100.0 <= max
}

/// Returns true when mark price falls within the optional inclusive bounds.
#[must_use]
pub fn filter_by_premium(
    contract: &FlatContract,
    min: Option<Number>,
    max: Option<Number>,
) -> bool {
    bounded_field(&contract.mark, min, max)
}

fn collect_exp_date_map(
    contracts: &mut Vec<FlatContract>,
    exp_date_map: Option<&HashMap<String, HashMap<String, Vec<OptionContract>>>>,
    fallback_contract_type: &str,
) {
    let Some(exp_date_map) = exp_date_map else {
        return;
    };

    for (expiration_key, strikes) in exp_date_map {
        let Some((expiration, dte)) = parse_expiration_key(expiration_key) else {
            continue;
        };

        for (strike_key, strike_contracts) in strikes {
            for contract in strike_contracts {
                if let Some(flat) = flatten_contract(
                    contract,
                    &expiration,
                    dte,
                    strike_key,
                    fallback_contract_type,
                ) {
                    contracts.push(flat);
                }
            }
        }
    }
}

fn flatten_contract(
    contract: &OptionContract,
    expiration: &str,
    dte: i32,
    strike_key: &str,
    fallback_contract_type: &str,
) -> Option<FlatContract> {
    let strike = contract
        .strike_price
        .or_else(|| strike_key.parse::<Number>().ok())?;
    let contract_type = contract_type_label(contract.put_call.as_ref(), fallback_contract_type);

    Some(FlatContract {
        expiration: expiration.to_string(),
        dte,
        strike,
        contract_type,
        symbol: extracted_field(contract, "symbol"),
        description: extracted_field(contract, "description"),
        bid: extracted_field(contract, "bid"),
        ask: extracted_field(contract, "ask"),
        mark: extracted_field(contract, "mark"),
        last: extracted_field(contract, "last"),
        close: extracted_field(contract, "close"),
        high_price: extracted_field(contract, "highPrice"),
        low_price: extracted_field(contract, "lowPrice"),
        delta: extracted_field(contract, "delta"),
        gamma: extracted_field(contract, "gamma"),
        theta: extracted_field(contract, "theta"),
        vega: extracted_field(contract, "vega"),
        rho: extracted_field(contract, "rho"),
        iv: extracted_field(contract, "iv"),
        oi: extracted_field(contract, "oi"),
        volume: extracted_field(contract, "volume"),
        itm: extracted_field(contract, "itm"),
        theoretical_value: extracted_field(contract, "theoreticalValue"),
        intrinsic_value: extracted_field(contract, "intrinsicValue"),
        extrinsic_value: extracted_field(contract, "extrinsicValue"),
        time_value: extracted_field(contract, "timeValue"),
        multiplier: extracted_field(contract, "multiplier"),
        exercise_type: extracted_field(contract, "exerciseType"),
        settlement_type: extracted_field(contract, "settlementType"),
        expiration_type: extracted_field(contract, "expirationType"),
        percent_change: extracted_field(contract, "percentChange"),
        mark_change: extracted_field(contract, "markChange"),
        mark_percent_change: extracted_field(contract, "markPercentChange"),
        days_to_expiration: extracted_field(contract, "daysToExpiration"),
    })
}

fn parse_expiration_key(expiration_key: &str) -> Option<(String, i32)> {
    let (expiration, dte) = expiration_key.split_once(':')?;
    if expiration.contains(':') || !is_valid_date(expiration) {
        return None;
    }
    Some((expiration.to_string(), dte.parse().ok()?))
}

fn is_valid_date(value: &str) -> bool {
    parse_date(value).is_some()
}

fn parse_date(value: &str) -> Option<Date> {
    let mut parts = value.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = Month::try_from(parts.next()?.parse::<u8>().ok()?).ok()?;
    let day = parts.next()?.parse::<u8>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Date::from_calendar_date(year, month, day).ok()
}

fn contract_type_label(put_call: Option<&PutCall>, fallback_contract_type: &str) -> String {
    match put_call {
        Some(PutCall::Call) => "CALL".to_string(),
        Some(PutCall::Put) => "PUT".to_string(),
        Some(_) | None => fallback_contract_type.to_string(),
    }
}

fn selected_field_value(contract: &FlatContract, field: &str) -> Value {
    match field {
        "expiration" | "expiry" => Value::String(contract.expiration.clone()),
        "dte" => Value::from(contract.dte),
        "strike" => to_value(contract.strike).unwrap_or_default(),
        "type" => Value::String(contract.contract_type.clone()),
        "contract_type" | "cp" => Value::String(contract.contract_type.clone()),
        "symbol" => option_value(&contract.symbol),
        "description" => option_value(&contract.description),
        "bid" => option_value(&contract.bid),
        "ask" => option_value(&contract.ask),
        "mark" => option_value(&contract.mark),
        "last" => option_value(&contract.last),
        "close" => option_value(&contract.close),
        "highPrice" => option_value(&contract.high_price),
        "lowPrice" => option_value(&contract.low_price),
        "delta" => option_value(&contract.delta),
        "gamma" => option_value(&contract.gamma),
        "theta" => option_value(&contract.theta),
        "vega" => option_value(&contract.vega),
        "rho" => option_value(&contract.rho),
        "iv" | "volatility" => option_value(&contract.iv),
        "oi" | "openInterest" => option_value(&contract.oi),
        "volume" | "totalVolume" => option_value(&contract.volume),
        "itm" | "inTheMoney" => option_value(&contract.itm),
        "theoreticalValue" => option_value(&contract.theoretical_value),
        "intrinsicValue" => option_value(&contract.intrinsic_value),
        "extrinsicValue" => option_value(&contract.extrinsic_value),
        "timeValue" => option_value(&contract.time_value),
        "multiplier" => option_value(&contract.multiplier),
        "exerciseType" => option_value(&contract.exercise_type),
        "settlementType" => option_value(&contract.settlement_type),
        "expirationType" => option_value(&contract.expiration_type),
        "percentChange" => option_value(&contract.percent_change),
        "markChange" => option_value(&contract.mark_change),
        "markPercentChange" => option_value(&contract.mark_percent_change),
        "daysToExpiration" => option_value(&contract.days_to_expiration),
        _ => Value::Null,
    }
}

fn option_value(value: &Option<Value>) -> Value {
    value.clone().unwrap_or_default()
}

fn extracted_field(contract: &OptionContract, field: &str) -> Option<Value> {
    ALL_FIELDS
        .get(field)
        .and_then(|definition| (definition.extractor)(contract))
}

pub(crate) fn available_fields() -> Vec<&'static str> {
    let mut fields = ALL_FIELDS.keys().copied().collect::<Vec<_>>();
    fields.sort_unstable();
    fields
}

fn bounded_field(value: &Option<Value>, min: Option<Number>, max: Option<Number>) -> bool {
    let Some(value) = value_to_f64(value.as_ref()) else {
        return false;
    };
    bounded_f64(value, min, max)
}

fn bounded_f64(value: f64, min: Option<Number>, max: Option<Number>) -> bool {
    if !value.is_finite() {
        return false;
    }
    if let Some(min) = min {
        let Some(min) = number_to_f64(min) else {
            return false;
        };
        if value < min {
            return false;
        }
    }
    if let Some(max) = max {
        let Some(max) = number_to_f64(max) else {
            return false;
        };
        if value > max {
            return false;
        }
    }
    true
}

fn value_to_f64(value: Option<&Value>) -> Option<f64> {
    let value = match value? {
        Value::Number(number) => number.as_f64()?,
        Value::String(number) => number.parse().ok()?,
        _ => return None,
    };
    value.is_finite().then_some(value)
}

fn number_to_f64(value: Number) -> Option<f64> {
    let value = value.to_string().parse::<f64>().ok()?;
    value.is_finite().then_some(value)
}

#[cfg(not(feature = "decimal"))]
fn compare_number(left: &Number, right: &Number) -> Ordering {
    match (left.is_nan(), right.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => left.total_cmp(right),
    }
}

#[cfg(feature = "decimal")]
fn compare_number(left: &Number, right: &Number) -> Ordering {
    left.cmp(right)
}

fn compare_contract_type(left: &str, right: &str) -> Ordering {
    contract_type_rank(left)
        .cmp(&contract_type_rank(right))
        .then_with(|| left.cmp(right))
}

fn contract_type_rank(contract_type: &str) -> u8 {
    match contract_type {
        "CALL" => 0,
        "PUT" => 1,
        _ => 2,
    }
}

fn extract_none(_: &OptionContract) -> Option<Value> {
    None
}

fn extract_value<T>(value: Option<T>) -> Option<Value>
where
    T: Serialize,
{
    value.and_then(|value| to_value(value).ok())
}

fn extract_symbol(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), clone symbol))
}

fn extract_description(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), clone description))
}

fn extract_bid(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), bid_price))
}

fn extract_ask(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), ask_price))
}

fn extract_mark(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), mark_price))
}

fn extract_last(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), last_price))
}

fn extract_close(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), close_price))
}

fn extract_high_price(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), high_price))
}

fn extract_low_price(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), low_price))
}

fn extract_delta(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), delta))
}

fn extract_gamma(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), gamma))
}

fn extract_theta(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), theta))
}

fn extract_vega(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), vega))
}

fn extract_rho(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), rho))
}

fn extract_iv(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), volatility))
}

fn extract_oi(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), open_interest))
}

fn extract_volume(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), total_volume))
}

fn extract_itm(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), is_in_the_money))
}

fn extract_theoretical_value(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), theoretical_option_value))
}

fn extract_intrinsic_value(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), intrinsic_value))
}

fn extract_extrinsic_value(contract: &OptionContract) -> Option<Value> {
    extract_time_value(contract)
}

fn extract_time_value(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), time_value))
}

fn extract_multiplier(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), multiplier))
}

fn extract_exercise_type(_: &OptionContract) -> Option<Value> {
    None
}

fn extract_settlement_type(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), clone settlement_type))
}

fn extract_expiration_type(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), clone expiration_type))
}

fn extract_percent_change(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), percent_change))
}

fn extract_mark_change(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), mark_change))
}

fn extract_mark_percent_change(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), mark_percent_change))
}

fn extract_days_to_expiration(contract: &OptionContract) -> Option<Value> {
    extract_value(opt_field!(Some(contract), days_to_expiration))
}
