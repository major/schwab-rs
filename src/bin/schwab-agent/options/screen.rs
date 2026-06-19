use std::cmp::Ordering;

use schwab::{Client, Number, OptionChain, OptionChainOptions};
use serde_json::{Value, json, to_value};

use crate::cli::ScreenArgs;
use crate::error::AppError;
use crate::shared::to_number;

use super::types::{
    ALL_FIELDS, FlatContract, SCREEN_FIELDS, filter_by_ask, filter_by_bid, filter_by_delta,
    filter_by_oi, filter_by_premium, filter_by_spread_pct, filter_by_strike, filter_by_volume,
    flatten_chain, select_fields, sort_contracts, validate_fields,
};

/// Fetches an option chain and returns filtered `option screen` rows.
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn handle(client: &Client, args: &ScreenArgs) -> Result<Value, AppError> {
    let options = build_chain_options(args);
    let chain = client.get_option_chain(options).await?;

    screen_chain(&chain, args)
}

pub(super) fn screen_chain(chain: &OptionChain, args: &ScreenArgs) -> Result<Value, AppError> {
    let underlying_price = underlying_price(chain);
    let mut contracts = flatten_chain(chain);
    sort_contracts(&mut contracts);

    if contracts.is_empty() {
        return Err(AppError::OptionsSymbolNotFound {
            symbol: args.symbol.clone(),
        });
    }

    let total_scanned = contracts.len();
    let mut filters_applied = Vec::new();

    apply_filters(&mut contracts, args, &mut filters_applied)?;
    apply_sort(&mut contracts, args, &mut filters_applied)?;

    if let Some(limit) = args.limit {
        contracts.truncate(limit);
    }

    let fields = selected_fields(args)?;
    let field_refs = fields.iter().map(String::as_str).collect::<Vec<_>>();
    let (columns, rows) = select_fields(&contracts, &field_refs);

    Ok(json!({
        "underlying": args.symbol,
        "underlyingPrice": underlying_price,
        "columns": columns,
        "rows": rows,
        "rowCount": rows.len(),
        "totalScanned": total_scanned,
        "filtersApplied": filters_applied,
    }))
}

fn build_chain_options(args: &ScreenArgs) -> OptionChainOptions {
    let mut options = OptionChainOptions::new(args.symbol.as_str())
        .parameter("strategy", "SINGLE")
        .include_underlying_quote(true);

    if let Some(contract_type) = args.contract_type.as_deref() {
        options = options.parameter("contractType", contract_type.to_uppercase());
    }
    if let Some(strike_range) = args.strike_range.as_deref() {
        options = options.parameter("range", strike_range);
    }

    options
}

fn apply_filters(
    contracts: &mut Vec<FlatContract>,
    args: &ScreenArgs,
    filters_applied: &mut Vec<String>,
) -> Result<(), AppError> {
    if let Some(contract_type) = normalized_contract_type(args.contract_type.as_deref())
        && contract_type != "ALL"
    {
        contracts.retain(|contract| {
            normalized_contract_type(Some(&contract.contract_type)).as_deref()
                == Some(contract_type.as_str())
        });
        filters_applied.push(format!("type = {contract_type}"));
    }

    if let Some(dte_min) = args.dte_min {
        contracts.retain(|contract| contract.dte >= dte_min);
        filters_applied.push(format!("dte >= {dte_min}"));
    }
    if let Some(dte_max) = args.dte_max {
        contracts.retain(|contract| contract.dte <= dte_max);
        filters_applied.push(format!("dte <= {dte_max}"));
    }

    if args.strike_min.is_some() || args.strike_max.is_some() {
        let min = optional_number(args.strike_min)?;
        let max = optional_number(args.strike_max)?;
        contracts.retain(|contract| filter_by_strike(contract, min, max, None));
        if let Some(strike_min) = args.strike_min {
            filters_applied.push(format!("strike >= {}", format_number(strike_min)));
        }
        if let Some(strike_max) = args.strike_max {
            filters_applied.push(format!("strike <= {}", format_number(strike_max)));
        }
    }

    if let Some(strike) = args.strike {
        let exact = Some(number_arg(strike)?);
        contracts.retain(|contract| filter_by_strike(contract, None, None, exact));
        filters_applied.push(format!("strike = {}", format_number(strike)));
    }

    if args.delta_min.is_some() || args.delta_max.is_some() {
        let min = optional_number(args.delta_min)?;
        let max = optional_number(args.delta_max)?;
        contracts.retain(|contract| filter_by_delta(contract, min, max));
        if let Some(delta_min) = args.delta_min {
            filters_applied.push(format!("delta >= {}", format_number(delta_min)));
        }
        if let Some(delta_max) = args.delta_max {
            filters_applied.push(format!("delta <= {}", format_number(delta_max)));
        }
    }

    if let Some(min_bid) = args.min_bid {
        let min_bid_number = number_arg(min_bid)?;
        contracts.retain(|contract| filter_by_bid(contract, min_bid_number));
        filters_applied.push(format!("bid >= {}", format_number(min_bid)));
    }
    if let Some(max_ask) = args.max_ask {
        let max_ask_number = number_arg(max_ask)?;
        contracts.retain(|contract| filter_by_ask(contract, max_ask_number));
        filters_applied.push(format!("ask <= {}", format_number(max_ask)));
    }
    if let Some(min_volume) = args.min_volume {
        let min_volume_number = number_arg(min_volume as f64)?;
        contracts.retain(|contract| filter_by_volume(contract, min_volume_number));
        filters_applied.push(format!("volume >= {min_volume}"));
    }
    if let Some(min_oi) = args.min_oi {
        let min_oi_number = number_arg(min_oi as f64)?;
        contracts.retain(|contract| filter_by_oi(contract, min_oi_number));
        filters_applied.push(format!("oi >= {min_oi}"));
    }
    if let Some(max_spread_pct) = args.max_spread_pct {
        let max_spread_pct_number = number_arg(max_spread_pct)?;
        contracts.retain(|contract| filter_by_spread_pct(contract, max_spread_pct_number));
        filters_applied.push(format!("spreadPct <= {}", format_number(max_spread_pct)));
    }
    if args.min_premium.is_some() || args.max_premium.is_some() {
        let min = optional_number(args.min_premium)?;
        let max = optional_number(args.max_premium)?;
        contracts.retain(|contract| filter_by_premium(contract, min, max));
        if let Some(min_premium) = args.min_premium {
            filters_applied.push(format!("premium >= {}", format_number(min_premium)));
        }
        if let Some(max_premium) = args.max_premium {
            filters_applied.push(format!("premium <= {}", format_number(max_premium)));
        }
    }

    Ok(())
}

fn apply_sort(
    contracts: &mut [FlatContract],
    args: &ScreenArgs,
    filters_applied: &mut Vec<String>,
) -> Result<(), AppError> {
    let Some(sort) = args.sort.as_deref() else {
        return Ok(());
    };
    let spec = parse_sort_spec(sort)?;

    contracts.sort_by(|left, right| {
        let ordering = compare_values(
            &sort_value(left, spec.field),
            &sort_value(right, spec.field),
        );
        match spec.direction {
            SortDirection::Asc => ordering,
            SortDirection::Desc => ordering.reverse(),
        }
    });
    filters_applied.push(format!("sort = {}:{}", spec.field, spec.direction.as_str()));

    Ok(())
}

fn selected_fields(args: &ScreenArgs) -> Result<Vec<String>, AppError> {
    let fields = args.fields.as_deref().map_or_else(
        || {
            SCREEN_FIELDS
                .iter()
                .map(|field| (*field).to_string())
                .collect()
        },
        parse_fields,
    );
    let fields = if fields.is_empty() {
        SCREEN_FIELDS
            .iter()
            .map(|field| (*field).to_string())
            .collect()
    } else {
        fields
    };

    validate_fields(&fields)?;
    Ok(fields)
}

fn parse_fields(fields: &str) -> Vec<String> {
    fields
        .split(',')
        .map(str::trim)
        .filter(|field| !field.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_sort_spec(sort: &str) -> Result<SortSpec<'_>, AppError> {
    let (field, direction) = match sort.rsplit_once(':') {
        Some((field, direction)) => (field.trim(), Some(direction.trim())),
        None => (sort.trim(), None),
    };

    if !ALL_FIELDS.contains_key(field) {
        return Err(AppError::OptionsValidation {
            message: format!("unknown sort field: {field}"),
        });
    }

    let direction =
        direction.map_or_else(|| Ok(default_sort_direction(field)), SortDirection::parse)?;

    Ok(SortSpec { field, direction })
}

fn default_sort_direction(field: &str) -> SortDirection {
    match field {
        "bid" | "ask" | "mark" | "last" | "close" | "highPrice" | "lowPrice" | "gamma"
        | "theta" | "vega" | "rho" | "iv" | "volatility" | "oi" | "openInterest" | "volume"
        | "totalVolume" | "theoreticalValue" | "intrinsicValue" | "extrinsicValue"
        | "timeValue" | "multiplier" | "percentChange" | "markChange" | "markPercentChange" => {
            SortDirection::Desc
        }
        _ => SortDirection::Asc,
    }
}

fn sort_value(contract: &FlatContract, field: &str) -> Value {
    match field {
        "expiration" | "expiry" => Value::String(contract.expiration.clone()),
        "dte" => Value::from(contract.dte),
        "strike" => number_value(contract.strike),
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

fn compare_values(left: &Value, right: &Value) -> Ordering {
    match (sort_key(left), sort_key(right)) {
        (SortKey::Null, SortKey::Null) => Ordering::Equal,
        (SortKey::Null, _) => Ordering::Greater,
        (_, SortKey::Null) => Ordering::Less,
        (SortKey::Number(left), SortKey::Number(right)) => left.total_cmp(&right),
        (SortKey::String(left), SortKey::String(right)) => left.cmp(right),
        (SortKey::Bool(left), SortKey::Bool(right)) => left.cmp(&right),
        (left, right) => left.rank().cmp(&right.rank()),
    }
}

fn sort_key(value: &Value) -> SortKey<'_> {
    match value {
        Value::Null => SortKey::Null,
        Value::Bool(value) => SortKey::Bool(*value),
        Value::Number(value) => value.as_f64().map_or(SortKey::Null, SortKey::Number),
        Value::String(value) => value
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .map_or(SortKey::String(value), SortKey::Number),
        _ => SortKey::Null,
    }
}

fn underlying_price(chain: &OptionChain) -> Value {
    chain
        .underlying_price
        .as_ref()
        .or_else(|| {
            chain
                .underlying
                .as_ref()
                .and_then(|underlying| underlying.last.as_ref())
        })
        .or_else(|| {
            chain
                .underlying
                .as_ref()
                .and_then(|underlying| underlying.mark.as_ref())
        })
        .and_then(|value| to_value(value).ok())
        .unwrap_or_default()
}

fn optional_number(value: Option<f64>) -> Result<Option<Number>, AppError> {
    value.map(number_arg).transpose()
}

fn number_arg(value: f64) -> Result<Number, AppError> {
    if !value.is_finite() {
        return Err(AppError::OptionsValidation {
            message: format!("numeric filter value must be finite, got {value}"),
        });
    }

    to_number(value).map_err(|error| AppError::OptionsValidation {
        message: error.to_string(),
    })
}

fn number_value(value: Number) -> Value {
    to_value(value).unwrap_or_default()
}

fn option_value(value: &Option<Value>) -> Value {
    value.clone().unwrap_or_default()
}

fn normalized_contract_type(contract_type: Option<&str>) -> Option<String> {
    contract_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_uppercase)
}

fn format_number(value: f64) -> String {
    let formatted = format!("{value}");
    formatted
}

#[derive(Clone, Copy, Debug)]
struct SortSpec<'a> {
    field: &'a str,
    direction: SortDirection,
}

#[derive(Clone, Copy, Debug)]
enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    fn parse(value: &str) -> Result<Self, AppError> {
        match value.to_ascii_lowercase().as_str() {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            other => Err(AppError::OptionsValidation {
                message: format!("sort direction must be asc or desc, got {other}"),
            }),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

enum SortKey<'a> {
    Null,
    Bool(bool),
    Number(f64),
    String(&'a str),
}

impl SortKey<'_> {
    const fn rank(&self) -> u8 {
        match self {
            Self::Number(_) => 0,
            Self::String(_) => 1,
            Self::Bool(_) => 2,
            Self::Null => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    fn default_screen_args(symbol: &str) -> ScreenArgs {
        ScreenArgs {
            symbol: symbol.to_string(),
            contract_type: None,
            expiration: None,
            strike_range: None,
            strike_count: None,
            dte_min: None,
            dte_max: None,
            strike_min: None,
            strike_max: None,
            strike: None,
            delta_min: None,
            delta_max: None,
            min_bid: None,
            max_ask: None,
            min_volume: None,
            min_oi: None,
            max_spread_pct: None,
            min_premium: None,
            max_premium: None,
            limit: None,
            sort: None,
            fields: None,
        }
    }

    fn chain_from_json(value: Value) -> OptionChain {
        serde_json::from_value(value).unwrap()
    }

    fn screen_chain_fixture() -> OptionChain {
        chain_from_json(json!({
            "symbol": "AAPL",
            "underlyingPrice": 105.5,
            "callExpDateMap": {
                "2026-01-16:239": {
                    "100.0": [{
                        "symbol": "AAPL  260116C00100000",
                        "description": "AAPL Jan 2026 100 Call",
                        "bidPrice": 8.0,
                        "askPrice": 8.4,
                        "markPrice": 8.2,
                        "lastPrice": 8.1,
                        "strikePrice": 100.0,
                        "delta": 0.62,
                        "gamma": 0.04,
                        "theta": -0.02,
                        "vega": 0.11,
                        "volatility": 30.0,
                        "openInterest": 250,
                        "totalVolume": 75,
                        "timeValue": 3.2,
                        "isInTheMoney": true
                    }],
                    "120.0": [{
                        "symbol": "AAPL  260116C00120000",
                        "description": "AAPL Jan 2026 120 Call",
                        "bidPrice": 1.0,
                        "askPrice": 1.8,
                        "markPrice": 1.4,
                        "lastPrice": 1.3,
                        "strikePrice": 120.0,
                        "delta": 0.28,
                        "gamma": 0.03,
                        "theta": -0.01,
                        "vega": 0.08,
                        "volatility": 28.0,
                        "openInterest": 25,
                        "totalVolume": 5,
                        "timeValue": 1.4,
                        "isInTheMoney": false
                    }]
                }
            },
            "putExpDateMap": {
                "2026-01-16:239": {
                    "95.0": [{
                        "symbol": "AAPL  260116P00095000",
                        "description": "AAPL Jan 2026 95 Put",
                        "bidPrice": 2.5,
                        "askPrice": 2.7,
                        "markPrice": 2.6,
                        "lastPrice": 2.55,
                        "strikePrice": 95.0,
                        "delta": -0.25,
                        "gamma": 0.02,
                        "theta": -0.01,
                        "vega": 0.07,
                        "volatility": 32.0,
                        "openInterest": 120,
                        "totalVolume": 40,
                        "timeValue": 2.6,
                        "isInTheMoney": false
                    }]
                }
            }
        }))
    }

    #[test]
    fn build_chain_options_basic() {
        let args = default_screen_args("AAPL");
        let _opts = build_chain_options(&args);
    }

    #[test]
    fn build_chain_options_with_type_and_range() {
        let mut args = default_screen_args("SPY");
        args.contract_type = Some("CALL".to_string());
        args.strike_range = Some("ITM".to_string());
        let _opts = build_chain_options(&args);
    }

    #[test]
    fn parse_sort_spec_default_direction() {
        let spec = parse_sort_spec("strike").unwrap();
        assert_eq!(spec.field, "strike");
        assert!(matches!(spec.direction, SortDirection::Asc));
    }

    #[test]
    fn parse_sort_spec_explicit_desc() {
        let spec = parse_sort_spec("bid:desc").unwrap();
        assert_eq!(spec.field, "bid");
        assert!(matches!(spec.direction, SortDirection::Desc));
    }

    #[test]
    fn parse_sort_spec_explicit_asc() {
        let spec = parse_sort_spec("volume:asc").unwrap();
        assert_eq!(spec.field, "volume");
        assert!(matches!(spec.direction, SortDirection::Asc));
    }

    #[test]
    fn parse_sort_spec_unknown_field_rejected() {
        let err = parse_sort_spec("bogus").unwrap_err();
        assert!(err.to_string().contains("unknown sort field"));
    }

    #[test]
    fn parse_sort_spec_invalid_direction_rejected() {
        let err = parse_sort_spec("strike:sideways").unwrap_err();
        assert!(err.to_string().contains("sort direction must be"));
    }

    #[test]
    fn default_sort_direction_desc_for_price_fields() {
        assert!(matches!(default_sort_direction("bid"), SortDirection::Desc));
        assert!(matches!(default_sort_direction("ask"), SortDirection::Desc));
        assert!(matches!(
            default_sort_direction("volume"),
            SortDirection::Desc
        ));
        assert!(matches!(
            default_sort_direction("theta"),
            SortDirection::Desc
        ));
    }

    #[test]
    fn default_sort_direction_asc_for_other_fields() {
        assert!(matches!(
            default_sort_direction("strike"),
            SortDirection::Asc
        ));
        assert!(matches!(
            default_sort_direction("expiration"),
            SortDirection::Asc
        ));
        assert!(matches!(default_sort_direction("dte"), SortDirection::Asc));
    }

    #[test]
    fn sort_direction_as_str() {
        assert_eq!(SortDirection::Asc.as_str(), "asc");
        assert_eq!(SortDirection::Desc.as_str(), "desc");
    }

    #[test]
    fn compare_values_numbers() {
        let a = Value::from(1.0);
        let b = Value::from(2.0);
        assert_eq!(compare_values(&a, &b), Ordering::Less);
        assert_eq!(compare_values(&b, &a), Ordering::Greater);
        assert_eq!(compare_values(&a, &a), Ordering::Equal);
    }

    #[test]
    fn compare_values_strings() {
        let a = Value::String("alpha".to_string());
        let b = Value::String("beta".to_string());
        assert_eq!(compare_values(&a, &b), Ordering::Less);
    }

    #[test]
    fn compare_values_null_sorts_last() {
        let a = Value::from(1.0);
        let b = Value::Null;
        assert_eq!(compare_values(&a, &b), Ordering::Less);
        assert_eq!(compare_values(&b, &a), Ordering::Greater);
        assert_eq!(compare_values(&b, &b), Ordering::Equal);
    }

    #[test]
    fn compare_values_bools() {
        let a = Value::Bool(false);
        let b = Value::Bool(true);
        assert_eq!(compare_values(&a, &b), Ordering::Less);
    }

    #[test]
    fn compare_values_mixed_types_use_rank() {
        let num = Value::from(1.0);
        let s = Value::String("abc".to_string());
        assert_eq!(compare_values(&num, &s), Ordering::Less);
    }

    #[test]
    fn sort_key_numeric_string_treated_as_number() {
        let v = Value::String("42.5".to_string());
        assert!(matches!(sort_key(&v), SortKey::Number(n) if (n - 42.5).abs() < f64::EPSILON));
    }

    #[test]
    fn sort_key_non_numeric_string_treated_as_string() {
        let v = Value::String("hello".to_string());
        assert!(matches!(sort_key(&v), SortKey::String("hello")));
    }

    #[test]
    fn sort_key_rank_ordering() {
        assert!(SortKey::Number(1.0).rank() < SortKey::String("a").rank());
        assert!(SortKey::String("a").rank() < SortKey::Bool(true).rank());
        assert!(SortKey::Bool(true).rank() < SortKey::Null.rank());
    }

    #[test]
    fn normalized_contract_type_normalizes() {
        assert_eq!(
            normalized_contract_type(Some("call")),
            Some("CALL".to_string())
        );
        assert_eq!(normalized_contract_type(None), None);
        assert_eq!(normalized_contract_type(Some("")), None);
        assert_eq!(normalized_contract_type(Some("  ")), None);
    }

    #[test]
    fn format_number_formats_as_string() {
        assert_eq!(format_number(42.5), "42.5");
        assert_eq!(format_number(100.0), "100");
    }

    #[test]
    fn number_arg_valid() {
        let n = number_arg(10.5).unwrap();
        assert_eq!(n.to_string(), "10.5");
    }

    #[test]
    fn number_value_converts_to_json() {
        let n = crate::shared::to_number(42.0).unwrap();
        let v = number_value(n);
        assert!(!v.is_null());
    }

    #[test]
    fn option_value_returns_value_or_null() {
        assert_eq!(option_value(&Some(json!(42))), json!(42));
        assert_eq!(option_value(&None), Value::Null);
    }

    #[test]
    fn selected_fields_defaults() {
        let args = default_screen_args("AAPL");
        let fields = selected_fields(&args).unwrap();
        assert!(!fields.is_empty());
    }

    #[test]
    fn parse_fields_splits_and_trims() {
        let fields = parse_fields("sym, strike , bid");
        assert_eq!(fields, vec!["sym", "strike", "bid"]);
    }

    #[test]
    fn parse_fields_skips_empty() {
        let fields = parse_fields("sym,,bid");
        assert_eq!(fields, vec!["sym", "bid"]);
    }

    #[test]
    fn selected_fields_empty_string_falls_back_to_defaults() {
        let mut args = default_screen_args("AAPL");
        args.fields = Some(" , , ".to_string());

        let fields = selected_fields(&args).unwrap();

        assert_eq!(fields[0], SCREEN_FIELDS[0]);
        assert_eq!(fields.len(), SCREEN_FIELDS.len());
    }

    #[test]
    fn selected_fields_rejects_unknown_field() {
        let mut args = default_screen_args("AAPL");
        args.fields = Some("symbol,nope".to_string());

        let error = selected_fields(&args).unwrap_err();

        assert_eq!(error.code(), "options.validation_failed");
        assert!(error.to_string().contains("nope"));
    }

    #[test]
    fn screen_chain_applies_filters_sort_and_limit() {
        let mut args = default_screen_args("AAPL");
        args.contract_type = Some("call".to_string());
        args.dte_min = Some(200);
        args.dte_max = Some(300);
        args.strike_min = Some(90.0);
        args.strike_max = Some(125.0);
        args.delta_min = Some(0.2);
        args.delta_max = Some(0.7);
        args.min_bid = Some(1.5);
        args.max_ask = Some(9.0);
        args.min_volume = Some(10);
        args.min_oi = Some(100);
        args.max_spread_pct = Some(10.0);
        args.min_premium = Some(7.0);
        args.max_premium = Some(9.0);
        args.sort = Some("bid:desc".to_string());
        args.limit = Some(1);
        args.fields = Some("symbol,strike,bid,ask,volume,openInterest,delta".to_string());

        let output = screen_chain(&screen_chain_fixture(), &args).unwrap();

        assert_eq!(output["underlying"], "AAPL");
        assert_eq!(
            output["underlyingPrice"],
            number_value(to_number(105.5).unwrap())
        );
        assert_eq!(output["rowCount"], 1);
        assert_eq!(output["totalScanned"], 3);
        assert_eq!(output["rows"][0][0], "AAPL  260116C00100000");
        assert_eq!(
            output["rows"][0][1],
            number_value(to_number(100.0).unwrap())
        );
        assert!(
            output["filtersApplied"]
                .as_array()
                .unwrap()
                .iter()
                .any(|filter| filter == "sort = bid:desc")
        );
    }

    #[test]
    fn screen_chain_exact_strike_and_ascending_sort() {
        let mut args = default_screen_args("AAPL");
        args.strike = Some(95.0);
        args.sort = Some("symbol:asc".to_string());
        args.fields = Some("symbol,type,strike".to_string());

        let output = screen_chain(&screen_chain_fixture(), &args).unwrap();

        assert_eq!(output["rowCount"], 1);
        assert_eq!(output["rows"][0][0], "AAPL  260116P00095000");
        assert_eq!(output["rows"][0][1], "PUT");
        assert!(
            output["filtersApplied"]
                .as_array()
                .unwrap()
                .iter()
                .any(|filter| filter == "strike = 95")
        );
    }

    #[test]
    fn screen_chain_contract_type_filters_rows() {
        let mut args = default_screen_args("AAPL");
        args.contract_type = Some("put".to_string());
        args.fields = Some("symbol,type".to_string());

        let output = screen_chain(&screen_chain_fixture(), &args).unwrap();

        assert_eq!(output["rowCount"], 1);
        assert_eq!(output["rows"][0][0], "AAPL  260116P00095000");
        assert_eq!(output["rows"][0][1], "PUT");
        assert!(
            output["filtersApplied"]
                .as_array()
                .unwrap()
                .iter()
                .any(|filter| filter == "type = PUT")
        );
    }

    #[test]
    fn screen_chain_errors_when_chain_has_no_contracts() {
        let args = default_screen_args("MISSING");
        let chain = chain_from_json(json!({ "symbol": "MISSING" }));

        let error = screen_chain(&chain, &args).unwrap_err();

        assert_eq!(error.code(), "options.symbol_not_found");
    }

    #[test]
    fn screen_chain_reports_invalid_numeric_filter() {
        let mut args = default_screen_args("AAPL");
        args.min_bid = Some(f64::NAN);

        let error = screen_chain(&screen_chain_fixture(), &args).unwrap_err();

        assert_eq!(error.code(), "options.validation_failed");
        assert!(error.to_string().contains("NaN") || error.to_string().contains("number"));
    }

    #[test]
    fn underlying_price_falls_back_to_last_then_mark_then_null() {
        let last = chain_from_json(json!({
            "symbol": "AAPL",
            "underlying": { "last": 101.25, "mark": 101.0 }
        }));
        let mark = chain_from_json(json!({
            "symbol": "AAPL",
            "underlying": { "mark": 99.75 }
        }));
        let none = chain_from_json(json!({ "symbol": "AAPL" }));

        assert_eq!(
            underlying_price(&last),
            number_value(to_number(101.25).unwrap())
        );
        assert_eq!(
            underlying_price(&mark),
            number_value(to_number(99.75).unwrap())
        );
        assert_eq!(underlying_price(&none), Value::Null);
    }

    #[test]
    fn sort_value_supports_all_screen_field_aliases() {
        let contract = flatten_chain(&screen_chain_fixture())
            .into_iter()
            .find(|contract| contract.contract_type == "CALL")
            .unwrap();

        for field in ALL_FIELDS.keys() {
            let _ = sort_value(&contract, field);
        }
    }
}
