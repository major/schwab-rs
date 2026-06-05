//! Dashboard command handler for category-grouped technical analysis output.

use core::str::FromStr;

use schwab::{Client, PriceHistoryOptions};
use serde_json::{Value, to_value};

use crate::cli::DashboardArgs;
use crate::error::AppError;
use crate::ta::candles::{OhlcvData, extract_ohlcv, validate_candles};
use crate::ta::custom::{historical_volatility, vwap};
use crate::ta::indicators::{
    adx, atr, bbands, ema, macd, rsi, sma, stochastic, strip_leading_nans,
};
use crate::ta::interval::{HistoryParams, Interval, interval_to_history_params};
use crate::ta::types::{
    AdxPoint, BbandsPoint, DashboardOutput, DerivedFields, MacdPoint, MomentumIndicators,
    MomentumSignal, SignalSummary, StochPoint, TaPoint, TrendIndicators, TrendSignal,
    VolatilityIndicators, VolatilitySignal, VolumeIndicators, VolumeSignal,
};

const LONG_RANGE_CANDLES: usize = 252;
const RANGE_20_CANDLES: usize = 20;
const TREND_SHORT_PERIOD: usize = 21;
const TREND_MEDIUM_PERIOD: usize = 50;
const TREND_LONG_PERIOD: usize = 200;
const RSI_PERIOD: usize = 14;
const MACD_FAST_PERIOD: usize = 12;
const MACD_SLOW_PERIOD: usize = 26;
const MACD_SIGNAL_PERIOD: usize = 9;
const STOCH_K_PERIOD: usize = 14;
const STOCH_SMOOTH_K_PERIOD: usize = 1;
const STOCH_D_PERIOD: usize = 3;
const ADX_PERIOD: usize = 14;
const ATR_PERIOD: usize = 14;
const BBANDS_PERIOD: usize = 20;
const BBANDS_STD_DEV: f64 = 2.0;
const HISTORICAL_VOLATILITY_PERIOD: usize = 20;
const AVG_VOLUME_PERIOD: usize = 20;
const PERCENT_MULTIPLIER: f64 = 100.0;
const RSI_OVERBOUGHT: f64 = 70.0;
const RSI_OVERSOLD: f64 = 30.0;
const STOCH_OVERBOUGHT: f64 = 80.0;
const STOCH_OVERSOLD: f64 = 20.0;
const ADX_TRENDING: f64 = 25.0;
const ATR_ELEVATED_PERCENT: f64 = 5.0;
const HIGH_RELATIVE_VOLUME: f64 = 1.5;
const PRICE_BASIS_PREVIOUS_CLOSE: &str = "previous_close";

#[derive(Debug, Clone)]
struct IndicatorSeries {
    sma_21: Vec<f64>,
    sma_50: Vec<f64>,
    sma_200: Vec<f64>,
    ema_21: Vec<f64>,
    rsi_14: Vec<f64>,
    macd: MacdSeries,
    stochastic: StochSeries,
    adx: AdxSeries,
    atr_14: Vec<f64>,
    bollinger_bands: BbandsSeries,
    historical_volatility: Vec<f64>,
    vwap: Vec<f64>,
    avg_volume_20: Vec<f64>,
}

#[derive(Debug, Clone)]
struct MacdSeries {
    macd: Vec<f64>,
    signal: Vec<f64>,
    histogram: Vec<f64>,
}

#[derive(Debug, Clone)]
struct BbandsSeries {
    upper: Vec<f64>,
    middle: Vec<f64>,
    lower: Vec<f64>,
}

#[derive(Debug, Clone)]
struct StochSeries {
    k: Vec<f64>,
    d: Vec<f64>,
}

#[derive(Debug, Clone)]
struct AdxSeries {
    adx: Vec<f64>,
    plus_di: Vec<f64>,
    minus_di: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
struct LatestValues {
    close: f64,
    timestamp: i64,
    volume: f64,
    sma_21: f64,
    sma_50: f64,
    sma_200: f64,
    rsi_14: f64,
    macd_histogram: f64,
    stoch_k: f64,
    adx: f64,
    atr_14: f64,
    upper_bband: f64,
    lower_bband: f64,
    vwap: f64,
    avg_volume_20: f64,
}

/// Fetches candles, computes all TA dashboard indicators, and returns JSON output.
///
/// # Errors
///
/// Returns [`AppError`] if the interval is invalid, Schwab price history fails,
/// candle data is insufficient or malformed, indicator math fails, or JSON
/// serialization fails.
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn dashboard(client: &Client, args: &DashboardArgs) -> Result<Value, AppError> {
    let interval = Interval::from_str(&args.interval)?;
    let required_candles = LONG_RANGE_CANDLES + args.points;
    let params = interval_to_history_params(interval, required_candles)?;
    let candle_list = client
        .get_price_history(&args.symbol, price_history_options(&params))
        .await?;
    let candles = validate_candles(&candle_list, required_candles)?;
    let ohlcv = extract_ohlcv(candles)?;

    render_dashboard(args, interval, &ohlcv)
}

fn render_dashboard(
    args: &DashboardArgs,
    interval: Interval,
    ohlcv: &OhlcvData,
) -> Result<Value, AppError> {
    let indicators = compute_indicators(ohlcv)?;
    let latest = latest_values(ohlcv, &indicators)?;
    let derived = compute_derived_fields(&latest, &ohlcv.highs, &ohlcv.lows)?;
    let relative_volume = relative_volume(latest.volume, latest.avg_volume_20);
    let signals = interpret_signals(&latest, &derived, relative_volume);

    to_value(assemble_output(
        args,
        interval,
        ohlcv,
        indicators,
        derived,
        relative_volume,
        signals,
    )?)
    .map_err(AppError::from)
}

fn price_history_options(params: &HistoryParams) -> PriceHistoryOptions {
    PriceHistoryOptions::new()
        .parameter("periodType", &params.period_type)
        .integer_parameter("period", i64::from(params.period))
        .parameter("frequencyType", &params.frequency_type)
        .integer_parameter("frequency", i64::from(params.frequency))
}

fn compute_indicators(ohlcv: &OhlcvData) -> Result<IndicatorSeries, AppError> {
    let macd_result = macd(
        &ohlcv.closes,
        MACD_FAST_PERIOD,
        MACD_SLOW_PERIOD,
        MACD_SIGNAL_PERIOD,
    );
    let bbands_result = bbands(&ohlcv.closes, BBANDS_PERIOD, BBANDS_STD_DEV);
    let stoch_result = stochastic(
        &ohlcv.highs,
        &ohlcv.lows,
        &ohlcv.closes,
        STOCH_K_PERIOD,
        STOCH_SMOOTH_K_PERIOD,
        STOCH_D_PERIOD,
    );
    let adx_result = adx(&ohlcv.highs, &ohlcv.lows, &ohlcv.closes, ADX_PERIOD);

    Ok(IndicatorSeries {
        sma_21: cleaned(sma(&ohlcv.closes, TREND_SHORT_PERIOD)),
        sma_50: cleaned(sma(&ohlcv.closes, TREND_MEDIUM_PERIOD)),
        sma_200: cleaned(sma(&ohlcv.closes, TREND_LONG_PERIOD)),
        ema_21: cleaned(ema(&ohlcv.closes, TREND_SHORT_PERIOD)),
        rsi_14: cleaned(rsi(&ohlcv.closes, RSI_PERIOD)),
        macd: MacdSeries {
            macd: cleaned(macd_result.macd),
            signal: cleaned(macd_result.signal),
            histogram: cleaned(macd_result.histogram),
        },
        stochastic: StochSeries {
            k: cleaned(stoch_result.k),
            d: cleaned(stoch_result.d),
        },
        adx: AdxSeries {
            adx: cleaned(adx_result.adx),
            plus_di: cleaned(adx_result.plus_di),
            minus_di: cleaned(adx_result.minus_di),
        },
        atr_14: cleaned(atr(&ohlcv.highs, &ohlcv.lows, &ohlcv.closes, ATR_PERIOD)),
        bollinger_bands: BbandsSeries {
            upper: cleaned(bbands_result.upper),
            middle: cleaned(bbands_result.middle),
            lower: cleaned(bbands_result.lower),
        },
        historical_volatility: cleaned(historical_volatility(
            &ohlcv.closes,
            HISTORICAL_VOLATILITY_PERIOD,
        )),
        vwap: cleaned(vwap(
            &ohlcv.highs,
            &ohlcv.lows,
            &ohlcv.closes,
            &ohlcv.volumes,
        )?),
        avg_volume_20: cleaned(sma(&ohlcv.volumes, AVG_VOLUME_PERIOD)),
    })
}

#[must_use]
fn cleaned(values: Vec<f64>) -> Vec<f64> {
    strip_leading_nans(&values)
}

fn assemble_output(
    args: &DashboardArgs,
    interval: Interval,
    ohlcv: &OhlcvData,
    indicators: IndicatorSeries,
    derived: DerivedFields,
    relative_volume: Option<f64>,
    signals: SignalSummary,
) -> Result<DashboardOutput, AppError> {
    let timestamps = &ohlcv.timestamps;
    Ok(DashboardOutput {
        symbol: args.symbol.clone(),
        interval: interval.to_string(),
        points: args.points,
        trend: TrendIndicators {
            sma_21: align_simple_points(&indicators.sma_21, timestamps, args.points, "sma_21")?,
            sma_50: align_simple_points(&indicators.sma_50, timestamps, args.points, "sma_50")?,
            sma_200: align_simple_points(&indicators.sma_200, timestamps, args.points, "sma_200")?,
            ema_21: align_simple_points(&indicators.ema_21, timestamps, args.points, "ema_21")?,
        },
        momentum: MomentumIndicators {
            rsi_14: align_simple_points(&indicators.rsi_14, timestamps, args.points, "rsi_14")?,
            macd: align_macd_points(&indicators.macd, timestamps, args.points)?,
            stochastic: align_stoch_points(&indicators.stochastic, timestamps, args.points)?,
            adx: align_adx_points(&indicators.adx, timestamps, args.points)?,
        },
        volatility: VolatilityIndicators {
            atr_14: align_simple_points(&indicators.atr_14, timestamps, args.points, "atr_14")?,
            bollinger_bands: align_bbands_points(
                &indicators.bollinger_bands,
                timestamps,
                args.points,
            )?,
            historical_volatility: align_simple_points(
                &indicators.historical_volatility,
                timestamps,
                args.points,
                "historical_volatility",
            )?,
        },
        volume: VolumeIndicators {
            vwap: align_simple_points(&indicators.vwap, timestamps, args.points, "vwap")?,
            avg_volume_20: align_simple_points(
                &indicators.avg_volume_20,
                timestamps,
                args.points,
                "avg_volume_20",
            )?,
            relative_volume: relative_volume.map(round2),
        },
        derived: DerivedFields {
            price_basis: derived.price_basis,
            price_basis_value: round2(derived.price_basis_value),
            price_basis_timestamp: derived.price_basis_timestamp,
            atr_percent: round2(derived.atr_percent),
            range_20_high: round2(derived.range_20_high),
            range_20_low: round2(derived.range_20_low),
            range_252_high: round2(derived.range_252_high),
            range_252_low: round2(derived.range_252_low),
            distance_from_sma_21: round2(derived.distance_from_sma_21),
            distance_from_sma_50: round2(derived.distance_from_sma_50),
            distance_from_sma_200: round2(derived.distance_from_sma_200),
        },
        signals,
    })
}

fn latest_values(
    ohlcv: &OhlcvData,
    indicators: &IndicatorSeries,
) -> Result<LatestValues, AppError> {
    Ok(LatestValues {
        close: latest(&ohlcv.closes, "close")?,
        timestamp: latest_timestamp(&ohlcv.timestamps, "timestamp")?,
        volume: latest(&ohlcv.volumes, "volume")?,
        sma_21: latest(&indicators.sma_21, "sma_21")?,
        sma_50: latest(&indicators.sma_50, "sma_50")?,
        sma_200: latest(&indicators.sma_200, "sma_200")?,
        rsi_14: latest(&indicators.rsi_14, "rsi_14")?,
        macd_histogram: latest(&indicators.macd.histogram, "macd_histogram")?,
        stoch_k: latest(&indicators.stochastic.k, "stoch_k")?,
        adx: latest(&indicators.adx.adx, "adx")?,
        atr_14: latest(&indicators.atr_14, "atr_14")?,
        upper_bband: latest(&indicators.bollinger_bands.upper, "upper_bband")?,
        lower_bband: latest(&indicators.bollinger_bands.lower, "lower_bband")?,
        vwap: latest(&indicators.vwap, "vwap")?,
        avg_volume_20: latest(&indicators.avg_volume_20, "avg_volume_20")?,
    })
}

fn compute_derived_fields(
    latest: &LatestValues,
    highs: &[f64],
    lows: &[f64],
) -> Result<DerivedFields, AppError> {
    Ok(DerivedFields {
        price_basis: PRICE_BASIS_PREVIOUS_CLOSE.to_string(),
        price_basis_value: latest.close,
        price_basis_timestamp: latest.timestamp,
        atr_percent: percentage(latest.atr_14, latest.close, "atr_percent")?,
        range_20_high: range_high(highs, RANGE_20_CANDLES)?,
        range_20_low: range_low(lows, RANGE_20_CANDLES)?,
        range_252_high: range_high(highs, LONG_RANGE_CANDLES)?,
        range_252_low: range_low(lows, LONG_RANGE_CANDLES)?,
        distance_from_sma_21: distance_from_average(latest.close, latest.sma_21, "sma_21")?,
        distance_from_sma_50: distance_from_average(latest.close, latest.sma_50, "sma_50")?,
        distance_from_sma_200: distance_from_average(latest.close, latest.sma_200, "sma_200")?,
    })
}

#[must_use]
fn relative_volume(volume: f64, avg_volume: f64) -> Option<f64> {
    (avg_volume != 0.0).then_some(volume / avg_volume)
}

#[must_use]
fn interpret_signals(
    latest: &LatestValues,
    derived: &DerivedFields,
    relative_volume: Option<f64>,
) -> SignalSummary {
    SignalSummary {
        trend: TrendSignal {
            above_sma_21: latest.close > latest.sma_21,
            above_sma_50: latest.close > latest.sma_50,
            above_sma_200: latest.close > latest.sma_200,
            sma_21_above_sma_50: latest.sma_21 > latest.sma_50,
            sma_50_above_sma_200: latest.sma_50 > latest.sma_200,
        },
        momentum: MomentumSignal {
            rsi_overbought: latest.rsi_14 >= RSI_OVERBOUGHT,
            rsi_oversold: latest.rsi_14 <= RSI_OVERSOLD,
            macd_bullish: latest.macd_histogram > 0.0,
            stoch_overbought: latest.stoch_k >= STOCH_OVERBOUGHT,
            stoch_oversold: latest.stoch_k <= STOCH_OVERSOLD,
            adx_trending: latest.adx >= ADX_TRENDING,
        },
        volatility: VolatilitySignal {
            atr_elevated: derived.atr_percent >= ATR_ELEVATED_PERCENT,
            price_near_upper_band: latest.close >= latest.upper_bband,
            price_near_lower_band: latest.close <= latest.lower_bband,
        },
        volume: VolumeSignal {
            high_relative_volume: relative_volume
                .is_some_and(|value| value >= HIGH_RELATIVE_VOLUME),
            price_above_vwap: latest.close > latest.vwap,
        },
    }
}

fn align_simple_points(
    values: &[f64],
    timestamps: &[i64],
    points: usize,
    indicator: &str,
) -> Result<Vec<TaPoint>, AppError> {
    ensure_alignable(values.len(), timestamps.len(), indicator)?;
    let start = timestamps.len() - values.len();
    let skip = values.len().saturating_sub(points);

    Ok(values
        .iter()
        .zip(&timestamps[start..])
        .skip(skip)
        .map(|(value, timestamp)| TaPoint {
            timestamp: *timestamp,
            value: round2(*value),
        })
        .collect())
}

fn align_macd_points(
    values: &MacdSeries,
    timestamps: &[i64],
    points: usize,
) -> Result<Vec<MacdPoint>, AppError> {
    let len = min_len(&[
        values.macd.len(),
        values.signal.len(),
        values.histogram.len(),
    ]);
    ensure_alignable(len, timestamps.len(), "macd")?;
    let macd = tail(&values.macd, len);
    let signal = tail(&values.signal, len);
    let histogram = tail(&values.histogram, len);
    let timestamp_start = timestamps.len() - len;
    let skip = len.saturating_sub(points);

    Ok(macd
        .iter()
        .zip(signal)
        .zip(histogram)
        .zip(&timestamps[timestamp_start..])
        .skip(skip)
        .map(|(((macd, signal), histogram), timestamp)| MacdPoint {
            timestamp: *timestamp,
            macd: round2(*macd),
            signal: round2(*signal),
            histogram: round2(*histogram),
        })
        .collect())
}

fn align_bbands_points(
    values: &BbandsSeries,
    timestamps: &[i64],
    points: usize,
) -> Result<Vec<BbandsPoint>, AppError> {
    let len = min_len(&[values.upper.len(), values.middle.len(), values.lower.len()]);
    ensure_alignable(len, timestamps.len(), "bollinger_bands")?;
    let upper = tail(&values.upper, len);
    let middle = tail(&values.middle, len);
    let lower = tail(&values.lower, len);
    let timestamp_start = timestamps.len() - len;
    let skip = len.saturating_sub(points);

    Ok(upper
        .iter()
        .zip(middle)
        .zip(lower)
        .zip(&timestamps[timestamp_start..])
        .skip(skip)
        .map(|(((upper, middle), lower), timestamp)| BbandsPoint {
            timestamp: *timestamp,
            upper: round2(*upper),
            middle: round2(*middle),
            lower: round2(*lower),
        })
        .collect())
}

fn align_stoch_points(
    values: &StochSeries,
    timestamps: &[i64],
    points: usize,
) -> Result<Vec<StochPoint>, AppError> {
    let len = min_len(&[values.k.len(), values.d.len()]);
    ensure_alignable(len, timestamps.len(), "stochastic")?;
    let k = tail(&values.k, len);
    let d = tail(&values.d, len);
    let timestamp_start = timestamps.len() - len;
    let skip = len.saturating_sub(points);

    Ok(k.iter()
        .zip(d)
        .zip(&timestamps[timestamp_start..])
        .skip(skip)
        .map(|((k, d), timestamp)| StochPoint {
            timestamp: *timestamp,
            k: round2(*k),
            d: round2(*d),
        })
        .collect())
}

fn align_adx_points(
    values: &AdxSeries,
    timestamps: &[i64],
    points: usize,
) -> Result<Vec<AdxPoint>, AppError> {
    let len = min_len(&[
        values.adx.len(),
        values.plus_di.len(),
        values.minus_di.len(),
    ]);
    ensure_alignable(len, timestamps.len(), "adx")?;
    let adx = tail(&values.adx, len);
    let plus_di = tail(&values.plus_di, len);
    let minus_di = tail(&values.minus_di, len);
    let timestamp_start = timestamps.len() - len;
    let skip = len.saturating_sub(points);

    Ok(adx
        .iter()
        .zip(plus_di)
        .zip(minus_di)
        .zip(&timestamps[timestamp_start..])
        .skip(skip)
        .map(|(((adx, plus_di), minus_di), timestamp)| AdxPoint {
            timestamp: *timestamp,
            adx: round2(*adx),
            plus_di: round2(*plus_di),
            minus_di: round2(*minus_di),
        })
        .collect())
}

/// Round a float to 2 decimal places to eliminate floating-point noise in output.
#[must_use]
fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn ensure_alignable(
    series_len: usize,
    timestamp_len: usize,
    indicator: &str,
) -> Result<(), AppError> {
    if series_len == 0 {
        return Err(insufficient_data(1, 0, indicator));
    }
    if series_len > timestamp_len {
        return Err(calculation_error(
            indicator,
            format!("series has {series_len} values but only {timestamp_len} timestamps"),
        ));
    }

    Ok(())
}

#[must_use]
fn tail(values: &[f64], len: usize) -> &[f64] {
    &values[values.len() - len..]
}

#[must_use]
fn min_len(values: &[usize]) -> usize {
    values.iter().copied().min().unwrap_or(0)
}

fn latest(values: &[f64], indicator: &str) -> Result<f64, AppError> {
    values
        .last()
        .copied()
        .ok_or_else(|| insufficient_data(1, 0, indicator))
}

fn latest_timestamp(values: &[i64], indicator: &str) -> Result<i64, AppError> {
    values
        .last()
        .copied()
        .ok_or_else(|| insufficient_data(1, 0, indicator))
}

fn range_high(highs: &[f64], window: usize) -> Result<f64, AppError> {
    let values = last_window(highs, window, "range_high")?;
    Ok(values.iter().copied().fold(f64::NEG_INFINITY, f64::max))
}

fn range_low(lows: &[f64], window: usize) -> Result<f64, AppError> {
    let values = last_window(lows, window, "range_low")?;
    Ok(values.iter().copied().fold(f64::INFINITY, f64::min))
}

fn last_window<'a>(
    values: &'a [f64],
    window: usize,
    indicator: &str,
) -> Result<&'a [f64], AppError> {
    if window == 0 || values.len() < window {
        return Err(insufficient_data(window.max(1), values.len(), indicator));
    }

    Ok(&values[values.len() - window..])
}

fn distance_from_average(close: f64, average: f64, indicator: &str) -> Result<f64, AppError> {
    percentage(close - average, average, indicator)
}

fn percentage(numerator: f64, denominator: f64, indicator: &str) -> Result<f64, AppError> {
    if denominator == 0.0 {
        return Err(calculation_error(
            indicator,
            "cannot compute percentage with a zero denominator".to_string(),
        ));
    }

    Ok((numerator / denominator) * PERCENT_MULTIPLIER)
}

fn insufficient_data(needed: usize, got: usize, indicator: &str) -> AppError {
    AppError::TaInsufficientData {
        needed,
        got,
        indicator: indicator.to_string(),
    }
}

fn calculation_error(indicator: &str, reason: String) -> AppError {
    AppError::TaCalculationError {
        indicator: indicator.to_string(),
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-9;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= EPSILON,
            "actual {actual} differs from expected {expected}",
        );
    }

    fn latest_fixture() -> LatestValues {
        LatestValues {
            close: 110.0,
            timestamp: 1_700_000_000,
            volume: 1500.0,
            sma_21: 100.0,
            sma_50: 95.0,
            sma_200: 90.0,
            rsi_14: 75.0,
            macd_histogram: 0.25,
            stoch_k: 85.0,
            adx: 30.0,
            atr_14: 5.5,
            upper_bband: 108.0,
            lower_bband: 92.0,
            vwap: 105.0,
            avg_volume_20: 1000.0,
        }
    }

    fn dashboard_args(points: usize) -> DashboardArgs {
        DashboardArgs {
            symbol: "XYZ".to_string(),
            interval: "daily".to_string(),
            points,
        }
    }

    fn mock_ohlcv(len: usize) -> OhlcvData {
        let values = (0..len).map(|index| index as f64).collect::<Vec<_>>();
        OhlcvData {
            opens: values.iter().map(|value| value + 90.0).collect(),
            highs: values.iter().map(|value| value + 100.0).collect(),
            lows: values.iter().map(|value| value + 50.0).collect(),
            closes: values.iter().map(|value| value + 75.0).collect(),
            volumes: values.iter().map(|value| value + 1000.0).collect(),
            timestamps: (0..len).map(|index| 1_700_000_000 + index as i64).collect(),
        }
    }

    fn indicator_series(len: usize) -> IndicatorSeries {
        let values = (0..len).map(|index| index as f64).collect::<Vec<_>>();
        IndicatorSeries {
            sma_21: values.iter().map(|value| value + 1.0).collect(),
            sma_50: values.iter().map(|value| value + 2.0).collect(),
            sma_200: values.iter().map(|value| value + 3.0).collect(),
            ema_21: values.iter().map(|value| value + 4.0).collect(),
            rsi_14: values.iter().map(|value| value + 5.0).collect(),
            macd: MacdSeries {
                macd: values.iter().map(|value| value + 6.0).collect(),
                signal: values.iter().map(|value| value + 7.0).collect(),
                histogram: values.iter().map(|value| value + 8.0).collect(),
            },
            stochastic: StochSeries {
                k: values.iter().map(|value| value + 9.0).collect(),
                d: values.iter().map(|value| value + 10.0).collect(),
            },
            adx: AdxSeries {
                adx: values.iter().map(|value| value + 11.0).collect(),
                plus_di: values.iter().map(|value| value + 12.0).collect(),
                minus_di: values.iter().map(|value| value + 13.0).collect(),
            },
            atr_14: values.iter().map(|value| value + 14.0).collect(),
            bollinger_bands: BbandsSeries {
                upper: values.iter().map(|value| value + 15.0).collect(),
                middle: values.iter().map(|value| value + 16.0).collect(),
                lower: values.iter().map(|value| value + 17.0).collect(),
            },
            historical_volatility: values.iter().map(|value| value + 18.0).collect(),
            vwap: values.iter().map(|value| value + 19.0).collect(),
            avg_volume_20: values.iter().map(|value| value + 20.0).collect(),
        }
    }

    #[test]
    fn dashboard_derived_fields_use_latest_values_and_recent_ranges() {
        let latest = latest_fixture();
        let highs = (1..=260).map(f64::from).collect::<Vec<_>>();
        let lows = (1..=260).map(f64::from).collect::<Vec<_>>();

        let derived =
            compute_derived_fields(&latest, &highs, &lows).expect("derived fields should compute");

        assert_close(derived.atr_percent, 5.0);
        assert_close(derived.distance_from_sma_21, 10.0);
        assert_close(derived.distance_from_sma_50, 15.789_473_684_210_526);
        assert_close(derived.distance_from_sma_200, 22.222_222_222_222_22);
        assert_close(derived.range_20_high, 260.0);
        assert_close(derived.range_20_low, 241.0);
        assert_close(derived.range_252_high, 260.0);
        assert_close(derived.range_252_low, 9.0);
    }

    #[test]
    fn dashboard_latest_values_include_latest_candle_timestamp() {
        let ohlcv = mock_ohlcv(260);
        let indicators = indicator_series(20);

        let latest = latest_values(&ohlcv, &indicators).expect("latest values should compute");

        assert_close(latest.close, 334.0);
        assert_eq!(latest.timestamp, 1_700_000_259);
    }

    #[test]
    fn dashboard_signal_interpretations_match_thresholds() {
        let latest = latest_fixture();
        let derived = compute_derived_fields(
            &latest,
            &(1..=260).map(f64::from).collect::<Vec<_>>(),
            &(1..=260).map(f64::from).collect::<Vec<_>>(),
        )
        .expect("derived fields should compute");

        let signals = interpret_signals(&latest, &derived, relative_volume(1500.0, 1000.0));

        assert!(signals.trend.above_sma_21);
        assert!(signals.trend.above_sma_50);
        assert!(signals.trend.above_sma_200);
        assert!(signals.trend.sma_21_above_sma_50);
        assert!(signals.trend.sma_50_above_sma_200);
        assert!(signals.momentum.rsi_overbought);
        assert!(!signals.momentum.rsi_oversold);
        assert!(signals.momentum.macd_bullish);
        assert!(signals.momentum.stoch_overbought);
        assert!(!signals.momentum.stoch_oversold);
        assert!(signals.momentum.adx_trending);
        assert!(signals.volatility.atr_elevated);
        assert!(signals.volatility.price_near_upper_band);
        assert!(!signals.volatility.price_near_lower_band);
        assert!(signals.volume.high_relative_volume);
        assert!(signals.volume.price_above_vwap);
    }

    #[test]
    fn dashboard_signal_interpretations_cover_oversold_and_bearish_cases() {
        let mut latest = latest_fixture();
        latest.close = 80.0;
        latest.rsi_14 = 25.0;
        latest.macd_histogram = -0.1;
        latest.stoch_k = 20.0;
        latest.adx = 24.9;
        latest.atr_14 = 1.0;
        latest.upper_bband = 90.0;
        latest.lower_bband = 82.0;
        latest.vwap = 85.0;
        let derived = compute_derived_fields(
            &latest,
            &(1..=260).map(f64::from).collect::<Vec<_>>(),
            &(1..=260).map(f64::from).collect::<Vec<_>>(),
        )
        .expect("derived fields should compute");

        let signals = interpret_signals(&latest, &derived, relative_volume(700.0, 1000.0));

        assert!(!signals.trend.above_sma_21);
        assert!(signals.momentum.rsi_oversold);
        assert!(!signals.momentum.rsi_overbought);
        assert!(!signals.momentum.macd_bullish);
        assert!(signals.momentum.stoch_oversold);
        assert!(!signals.momentum.adx_trending);
        assert!(!signals.volatility.atr_elevated);
        assert!(signals.volatility.price_near_lower_band);
        assert!(!signals.volume.high_relative_volume);
        assert!(!signals.volume.price_above_vwap);
    }

    #[test]
    fn dashboard_aligns_indicator_series_to_tail_timestamps() {
        let timestamps = [10, 20, 30, 40, 50];
        let values = [1.0, 2.0, 3.0];

        let points =
            align_simple_points(&values, &timestamps, 3, "test").expect("series should align");

        assert_eq!(points.len(), 3);
        assert_eq!(points[0].timestamp, 30);
        assert_close(points[0].value, 1.0);
        assert_eq!(points[2].timestamp, 50);
        assert_close(points[2].value, 3.0);
    }

    #[test]
    fn dashboard_trims_series_to_requested_points() {
        let timestamps = [10, 20, 30, 40, 50, 60, 70];
        let values = [1.0, 2.0, 3.0, 4.0, 5.0];

        let points =
            align_simple_points(&values, &timestamps, 5, "test").expect("series should align");

        assert_eq!(points.len(), 5);
        assert_eq!(points[0].timestamp, 30);
        assert_eq!(points[4].timestamp, 70);

        let trimmed =
            align_simple_points(&values, &timestamps, 2, "test").expect("series should trim");

        assert_eq!(trimmed.len(), 2);
        assert_eq!(trimmed[0].timestamp, 60);
        assert_close(trimmed[0].value, 4.0);
        assert_eq!(trimmed[1].timestamp, 70);
        assert_close(trimmed[1].value, 5.0);
    }

    #[test]
    fn dashboard_aligns_multi_value_indicator_series() {
        let timestamps = [10, 20, 30, 40, 50];
        let macd = MacdSeries {
            macd: vec![1.111, 2.222, 3.333],
            signal: vec![0.111, 0.222, 0.333],
            histogram: vec![1.0, 2.0, 3.0],
        };
        let bbands = BbandsSeries {
            upper: vec![11.111, 12.222, 13.333],
            middle: vec![10.111, 10.222, 10.333],
            lower: vec![9.111, 8.222, 7.333],
        };
        let stoch = StochSeries {
            k: vec![80.111, 70.222, 60.333],
            d: vec![75.111, 65.222, 55.333],
        };
        let adx = AdxSeries {
            adx: vec![20.111, 25.222, 30.333],
            plus_di: vec![21.111, 26.222, 31.333],
            minus_di: vec![19.111, 24.222, 29.333],
        };

        let macd_points = align_macd_points(&macd, &timestamps, 2).unwrap();
        let bbands_points = align_bbands_points(&bbands, &timestamps, 2).unwrap();
        let stoch_points = align_stoch_points(&stoch, &timestamps, 2).unwrap();
        let adx_points = align_adx_points(&adx, &timestamps, 2).unwrap();

        assert_eq!(macd_points[0].timestamp, 40);
        assert_close(macd_points[0].macd, 2.22);
        assert_close(macd_points[1].histogram, 3.0);
        assert_eq!(bbands_points[0].timestamp, 40);
        assert_close(bbands_points[0].upper, 12.22);
        assert_close(bbands_points[1].lower, 7.33);
        assert_eq!(stoch_points[0].timestamp, 40);
        assert_close(stoch_points[0].k, 70.22);
        assert_close(stoch_points[1].d, 55.33);
        assert_eq!(adx_points[0].timestamp, 40);
        assert_close(adx_points[0].plus_di, 26.22);
        assert_close(adx_points[1].minus_di, 29.33);
    }

    #[test]
    fn dashboard_alignment_reports_empty_or_overlong_series() {
        let empty = align_simple_points(&[], &[10, 20], 1, "empty").unwrap_err();
        let overlong = align_simple_points(&[1.0, 2.0, 3.0], &[10, 20], 1, "overlong").unwrap_err();

        assert!(matches!(empty, AppError::TaInsufficientData { .. }));
        assert!(matches!(overlong, AppError::TaCalculationError { .. }));
    }

    #[test]
    fn dashboard_helpers_report_insufficient_data_and_zero_denominators() {
        assert!(matches!(
            latest(&[], "close"),
            Err(AppError::TaInsufficientData { .. })
        ));
        assert!(matches!(
            latest_timestamp(&[], "timestamp"),
            Err(AppError::TaInsufficientData { .. })
        ));
        assert!(matches!(
            last_window(&[1.0, 2.0], 3, "window"),
            Err(AppError::TaInsufficientData { .. })
        ));
        assert!(matches!(
            last_window(&[1.0, 2.0], 0, "window"),
            Err(AppError::TaInsufficientData { needed: 1, .. })
        ));
        assert!(matches!(
            percentage(1.0, 0.0, "pct"),
            Err(AppError::TaCalculationError { .. })
        ));
        assert_eq!(relative_volume(1000.0, 0.0), None);
    }

    #[test]
    fn dashboard_output_assembly_populates_category_grouped_fields() {
        let args = dashboard_args(5);
        let ohlcv = mock_ohlcv(260);
        let indicators = indicator_series(20);
        let latest = latest_fixture();
        let derived = compute_derived_fields(&latest, &ohlcv.highs, &ohlcv.lows)
            .expect("derived fields should compute");
        let relative_volume = Some(1.5);
        let signals = interpret_signals(&latest, &derived, relative_volume);

        let output = assemble_output(
            &args,
            Interval::Daily,
            &ohlcv,
            indicators,
            derived,
            relative_volume,
            signals,
        )
        .expect("output should assemble");

        assert_eq!(output.symbol, "XYZ");
        assert_eq!(output.interval, "daily");
        assert_eq!(output.points, 5);
        assert_eq!(output.trend.sma_21.len(), 5);
        assert_eq!(output.trend.sma_50.len(), 5);
        assert_eq!(output.trend.sma_200.len(), 5);
        assert_eq!(output.trend.ema_21.len(), 5);
        assert_eq!(output.momentum.rsi_14.len(), 5);
        assert_eq!(output.momentum.macd.len(), 5);
        assert_eq!(output.momentum.stochastic.len(), 5);
        assert_eq!(output.momentum.adx.len(), 5);
        assert_eq!(output.volatility.atr_14.len(), 5);
        assert_eq!(output.volatility.bollinger_bands.len(), 5);
        assert_eq!(output.volatility.historical_volatility.len(), 5);
        assert_eq!(output.volume.vwap.len(), 5);
        assert_eq!(output.volume.avg_volume_20.len(), 5);
        assert_close(output.volume.relative_volume.unwrap(), 1.5);
        assert!(output.signals.trend.above_sma_21);
        assert_close(output.trend.sma_21[0].value, 16.0);
        assert_eq!(output.trend.sma_21[0].timestamp, 1_700_000_255);
        assert_close(output.momentum.macd[4].histogram, 27.0);
        assert_close(output.volatility.bollinger_bands[4].upper, 34.0);
        assert_eq!(output.derived.price_basis, "previous_close");
        assert_close(output.derived.price_basis_value, 110.0);
        assert_eq!(output.derived.price_basis_timestamp, 1_700_000_000);
    }
}
