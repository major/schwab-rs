//! Multi-symbol analyze command.

use std::collections::HashMap;

use schwab::{Client, QuoteResponseObject};
use serde::Deserialize;
use serde_json::Value;

use crate::cli::{AnalyzeArgs, DashboardArgs};
use crate::error::AppError;
use crate::market;
use crate::ta;
use crate::ta::types::{
    AdxPoint, AnalyzeOutput, AnalyzeSymbolResult, BbandsPoint, DashboardOutput, DerivedFields,
    MacdPoint, MomentumIndicators, MomentumSignal, SignalSummary, StochPoint, TaPoint,
    TrendIndicators, TrendSignal, VolatilityIndicators, VolatilitySignal, VolumeIndicators,
    VolumeSignal,
};

/// Runs quote retrieval and technical analysis for one or more symbols.
///
/// # Errors
///
/// Returns [`AppError`] only for unexpected serialization failures that prevent
/// building the command output. Per-symbol quote and dashboard failures are
/// captured in each result's `quote_error` and `analysis_error` fields.
pub async fn analyze(client: &Client, args: &AnalyzeArgs) -> Result<Value, AppError> {
    let mut results = Vec::with_capacity(args.symbols.len());

    for symbol in &args.symbols {
        results.push(analyze_symbol(client, args, symbol).await);
    }

    let output = AnalyzeOutput { results };
    Ok(serde_json::to_value(&output)?)
}

async fn analyze_symbol(client: &Client, args: &AnalyzeArgs, symbol: &str) -> AnalyzeSymbolResult {
    let (quote, quote_error) = quote_result(client, symbol).await;
    let (analysis, analysis_error) = dashboard_result(client, args, symbol).await;

    AnalyzeSymbolResult {
        symbol: symbol.to_string(),
        quote,
        analysis,
        quote_error,
        analysis_error,
    }
}

async fn quote_result(client: &Client, symbol: &str) -> (Option<Value>, Option<String>) {
    match client.get_quote(symbol, None).await {
        Ok(quote) => match quote_map_to_value(quote) {
            Ok(value) => (Some(value), None),
            Err(error) => (None, Some(error.to_string())),
        },
        Err(error) => (None, Some(error.to_string())),
    }
}

fn quote_map_to_value(
    quotes: HashMap<String, QuoteResponseObject>,
) -> Result<Value, serde_json::Error> {
    let mut summaries: Vec<_> = quotes
        .into_iter()
        .map(|(symbol, quote)| market::summarize_quote(symbol, quote))
        .collect();
    summaries.sort_by(|left, right| left.requested_symbol.cmp(&right.requested_symbol));
    serde_json::to_value(summaries)
}

async fn dashboard_result(
    client: &Client,
    args: &AnalyzeArgs,
    symbol: &str,
) -> (Option<DashboardOutput>, Option<String>) {
    let dashboard_args = DashboardArgs {
        symbol: symbol.to_string(),
        interval: args.interval.clone(),
        points: args.points,
    };

    match ta::dashboard(client, &dashboard_args).await {
        Ok(value) => match dashboard_from_value(value) {
            Ok(output) => (Some(output), None),
            Err(error) => (None, Some(error.to_string())),
        },
        Err(error) => (None, Some(error.to_string())),
    }
}

fn dashboard_from_value(value: Value) -> Result<DashboardOutput, AppError> {
    Ok(serde_json::from_value::<DashboardOutputWire>(value)?.into())
}

#[must_use]
fn map_into<T, U>(values: Vec<T>) -> Vec<U>
where
    T: Into<U>,
{
    values.into_iter().map(Into::into).collect()
}

#[derive(Deserialize)]
struct TaPointWire {
    timestamp: i64,
    value: f64,
}

impl From<TaPointWire> for TaPoint {
    fn from(value: TaPointWire) -> Self {
        Self {
            timestamp: value.timestamp,
            value: value.value,
        }
    }
}

#[derive(Deserialize)]
struct MacdPointWire {
    timestamp: i64,
    macd: f64,
    signal: f64,
    histogram: f64,
}

impl From<MacdPointWire> for MacdPoint {
    fn from(value: MacdPointWire) -> Self {
        Self {
            timestamp: value.timestamp,
            macd: value.macd,
            signal: value.signal,
            histogram: value.histogram,
        }
    }
}

#[derive(Deserialize)]
struct BbandsPointWire {
    timestamp: i64,
    upper: f64,
    middle: f64,
    lower: f64,
}

impl From<BbandsPointWire> for BbandsPoint {
    fn from(value: BbandsPointWire) -> Self {
        Self {
            timestamp: value.timestamp,
            upper: value.upper,
            middle: value.middle,
            lower: value.lower,
        }
    }
}

#[derive(Deserialize)]
struct StochPointWire {
    timestamp: i64,
    k: f64,
    d: f64,
}

impl From<StochPointWire> for StochPoint {
    fn from(value: StochPointWire) -> Self {
        Self {
            timestamp: value.timestamp,
            k: value.k,
            d: value.d,
        }
    }
}

#[derive(Deserialize)]
struct AdxPointWire {
    timestamp: i64,
    adx: f64,
    plus_di: f64,
    minus_di: f64,
}

impl From<AdxPointWire> for AdxPoint {
    fn from(value: AdxPointWire) -> Self {
        Self {
            timestamp: value.timestamp,
            adx: value.adx,
            plus_di: value.plus_di,
            minus_di: value.minus_di,
        }
    }
}

#[derive(Deserialize)]
struct DashboardOutputWire {
    symbol: String,
    interval: String,
    points: usize,
    trend: TrendIndicatorsWire,
    momentum: MomentumIndicatorsWire,
    volatility: VolatilityIndicatorsWire,
    volume: VolumeIndicatorsWire,
    derived: DerivedFieldsWire,
    signals: SignalSummaryWire,
}

impl From<DashboardOutputWire> for DashboardOutput {
    fn from(value: DashboardOutputWire) -> Self {
        Self {
            symbol: value.symbol,
            interval: value.interval,
            points: value.points,
            trend: value.trend.into(),
            momentum: value.momentum.into(),
            volatility: value.volatility.into(),
            volume: value.volume.into(),
            derived: value.derived.into(),
            signals: value.signals.into(),
        }
    }
}

#[derive(Deserialize)]
struct TrendIndicatorsWire {
    sma_21: Vec<TaPointWire>,
    sma_50: Vec<TaPointWire>,
    sma_200: Vec<TaPointWire>,
    ema_21: Vec<TaPointWire>,
}

impl From<TrendIndicatorsWire> for TrendIndicators {
    fn from(value: TrendIndicatorsWire) -> Self {
        Self {
            sma_21: map_into(value.sma_21),
            sma_50: map_into(value.sma_50),
            sma_200: map_into(value.sma_200),
            ema_21: map_into(value.ema_21),
        }
    }
}

#[derive(Deserialize)]
struct MomentumIndicatorsWire {
    rsi_14: Vec<TaPointWire>,
    macd: Vec<MacdPointWire>,
    stochastic: Vec<StochPointWire>,
    adx: Vec<AdxPointWire>,
}

impl From<MomentumIndicatorsWire> for MomentumIndicators {
    fn from(value: MomentumIndicatorsWire) -> Self {
        Self {
            rsi_14: map_into(value.rsi_14),
            macd: map_into(value.macd),
            stochastic: map_into(value.stochastic),
            adx: map_into(value.adx),
        }
    }
}

#[derive(Deserialize)]
struct VolatilityIndicatorsWire {
    atr_14: Vec<TaPointWire>,
    bollinger_bands: Vec<BbandsPointWire>,
    historical_volatility: Vec<TaPointWire>,
}

impl From<VolatilityIndicatorsWire> for VolatilityIndicators {
    fn from(value: VolatilityIndicatorsWire) -> Self {
        Self {
            atr_14: map_into(value.atr_14),
            bollinger_bands: map_into(value.bollinger_bands),
            historical_volatility: map_into(value.historical_volatility),
        }
    }
}

#[derive(Deserialize)]
struct VolumeIndicatorsWire {
    vwap: Vec<TaPointWire>,
    avg_volume_20: Vec<TaPointWire>,
    relative_volume: Option<f64>,
}

impl From<VolumeIndicatorsWire> for VolumeIndicators {
    fn from(value: VolumeIndicatorsWire) -> Self {
        Self {
            vwap: map_into(value.vwap),
            avg_volume_20: map_into(value.avg_volume_20),
            relative_volume: value.relative_volume,
        }
    }
}

#[derive(Deserialize)]
struct DerivedFieldsWire {
    price_basis: String,
    price_basis_value: f64,
    price_basis_timestamp: i64,
    atr_percent: f64,
    range_20_high: f64,
    range_20_low: f64,
    range_252_high: f64,
    range_252_low: f64,
    distance_from_sma_21: f64,
    distance_from_sma_50: f64,
    distance_from_sma_200: f64,
}

impl From<DerivedFieldsWire> for DerivedFields {
    fn from(value: DerivedFieldsWire) -> Self {
        Self {
            price_basis: value.price_basis,
            price_basis_value: value.price_basis_value,
            price_basis_timestamp: value.price_basis_timestamp,
            atr_percent: value.atr_percent,
            range_20_high: value.range_20_high,
            range_20_low: value.range_20_low,
            range_252_high: value.range_252_high,
            range_252_low: value.range_252_low,
            distance_from_sma_21: value.distance_from_sma_21,
            distance_from_sma_50: value.distance_from_sma_50,
            distance_from_sma_200: value.distance_from_sma_200,
        }
    }
}

#[derive(Deserialize)]
struct SignalSummaryWire {
    trend: TrendSignalWire,
    momentum: MomentumSignalWire,
    volatility: VolatilitySignalWire,
    volume: VolumeSignalWire,
}

impl From<SignalSummaryWire> for SignalSummary {
    fn from(value: SignalSummaryWire) -> Self {
        Self {
            trend: value.trend.into(),
            momentum: value.momentum.into(),
            volatility: value.volatility.into(),
            volume: value.volume.into(),
        }
    }
}

#[derive(Deserialize)]
struct TrendSignalWire {
    above_sma_21: bool,
    above_sma_50: bool,
    above_sma_200: bool,
    sma_21_above_sma_50: bool,
    sma_50_above_sma_200: bool,
}

impl From<TrendSignalWire> for TrendSignal {
    fn from(value: TrendSignalWire) -> Self {
        Self {
            above_sma_21: value.above_sma_21,
            above_sma_50: value.above_sma_50,
            above_sma_200: value.above_sma_200,
            sma_21_above_sma_50: value.sma_21_above_sma_50,
            sma_50_above_sma_200: value.sma_50_above_sma_200,
        }
    }
}

#[derive(Deserialize)]
struct MomentumSignalWire {
    rsi_overbought: bool,
    rsi_oversold: bool,
    macd_bullish: bool,
    stoch_overbought: bool,
    stoch_oversold: bool,
    adx_trending: bool,
}

impl From<MomentumSignalWire> for MomentumSignal {
    fn from(value: MomentumSignalWire) -> Self {
        Self {
            rsi_overbought: value.rsi_overbought,
            rsi_oversold: value.rsi_oversold,
            macd_bullish: value.macd_bullish,
            stoch_overbought: value.stoch_overbought,
            stoch_oversold: value.stoch_oversold,
            adx_trending: value.adx_trending,
        }
    }
}

#[derive(Deserialize)]
struct VolatilitySignalWire {
    atr_elevated: bool,
    price_near_upper_band: bool,
    price_near_lower_band: bool,
}

impl From<VolatilitySignalWire> for VolatilitySignal {
    fn from(value: VolatilitySignalWire) -> Self {
        Self {
            atr_elevated: value.atr_elevated,
            price_near_upper_band: value.price_near_upper_band,
            price_near_lower_band: value.price_near_lower_band,
        }
    }
}

#[derive(Deserialize)]
struct VolumeSignalWire {
    high_relative_volume: bool,
    price_above_vwap: bool,
}

impl From<VolumeSignalWire> for VolumeSignal {
    fn from(value: VolumeSignalWire) -> Self {
        Self {
            high_relative_volume: value.high_relative_volume,
            price_above_vwap: value.price_above_vwap,
        }
    }
}

#[cfg(test)]
mod tests;
