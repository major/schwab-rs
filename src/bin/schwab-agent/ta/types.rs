//! Output types for TA commands.

use serde::Serialize;

/// Single data point for simple indicators.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct TaPoint {
    /// Candle timestamp as a Unix epoch value.
    pub timestamp: i64,
    /// Indicator value at the timestamp.
    pub value: f64,
}

/// MACD data point with three series.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct MacdPoint {
    /// Candle timestamp as a Unix epoch value.
    pub timestamp: i64,
    /// MACD line value.
    pub macd: f64,
    /// MACD signal line value.
    pub signal: f64,
    /// MACD histogram value.
    pub histogram: f64,
}

/// Bollinger Bands data point.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct BbandsPoint {
    /// Candle timestamp as a Unix epoch value.
    pub timestamp: i64,
    /// Upper Bollinger band.
    pub upper: f64,
    /// Middle Bollinger band.
    pub middle: f64,
    /// Lower Bollinger band.
    pub lower: f64,
}

/// Stochastic oscillator data point.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct StochPoint {
    /// Candle timestamp as a Unix epoch value.
    pub timestamp: i64,
    /// %K line value.
    pub k: f64,
    /// %D line value.
    pub d: f64,
}

/// ADX directional indicator data point.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct AdxPoint {
    /// Candle timestamp as a Unix epoch value.
    pub timestamp: i64,
    /// Average directional index value.
    pub adx: f64,
    /// Positive directional indicator.
    pub plus_di: f64,
    /// Negative directional indicator.
    pub minus_di: f64,
}

/// Trend indicator series (SMA, EMA).
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct TrendIndicators {
    /// 21-period simple moving average.
    pub sma_21: Vec<TaPoint>,
    /// 50-period simple moving average.
    pub sma_50: Vec<TaPoint>,
    /// 200-period simple moving average.
    pub sma_200: Vec<TaPoint>,
    /// 21-period exponential moving average.
    pub ema_21: Vec<TaPoint>,
}

/// Momentum indicator series (RSI, MACD, Stochastic, ADX).
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct MomentumIndicators {
    /// 14-period RSI series.
    pub rsi_14: Vec<TaPoint>,
    /// MACD series.
    pub macd: Vec<MacdPoint>,
    /// Stochastic oscillator series.
    pub stochastic: Vec<StochPoint>,
    /// ADX series.
    pub adx: Vec<AdxPoint>,
}

/// Volatility indicator series (ATR, BBands, HV).
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct VolatilityIndicators {
    /// 14-period average true range.
    pub atr_14: Vec<TaPoint>,
    /// Bollinger band series.
    pub bollinger_bands: Vec<BbandsPoint>,
    /// Historical volatility series.
    pub historical_volatility: Vec<TaPoint>,
}

/// Volume indicator series (VWAP, avg volume, relative volume).
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct VolumeIndicators {
    /// Volume weighted average price series.
    pub vwap: Vec<TaPoint>,
    /// 20-period average volume series.
    pub avg_volume_20: Vec<TaPoint>,
    /// Relative volume scalar.
    pub relative_volume: Option<f64>,
}

/// Derived scalar fields computed from indicator outputs.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct DerivedFields {
    /// Price basis used for derived calculations.
    pub price_basis: String,
    /// Price value used for derived calculations.
    pub price_basis_value: f64,
    /// Timestamp for the price basis as a Unix epoch millisecond value.
    pub price_basis_timestamp: i64,
    /// ATR as a percentage of price.
    pub atr_percent: f64,
    /// 20-day high price range.
    pub range_20_high: f64,
    /// 20-day low price range.
    pub range_20_low: f64,
    /// 252-day high price range.
    pub range_252_high: f64,
    /// 252-day low price range.
    pub range_252_low: f64,
    /// Distance from the 21-period SMA.
    pub distance_from_sma_21: f64,
    /// Distance from the 50-period SMA.
    pub distance_from_sma_50: f64,
    /// Distance from the 200-period SMA.
    pub distance_from_sma_200: f64,
}

/// Signal interpretation for trend indicators.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct TrendSignal {
    /// Price above the 21-period SMA.
    pub above_sma_21: bool,
    /// Price above the 50-period SMA.
    pub above_sma_50: bool,
    /// Price above the 200-period SMA.
    pub above_sma_200: bool,
    /// 21-period SMA above the 50-period SMA.
    pub sma_21_above_sma_50: bool,
    /// 50-period SMA above the 200-period SMA.
    pub sma_50_above_sma_200: bool,
}

/// Signal interpretation for momentum indicators.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct MomentumSignal {
    /// RSI above the overbought threshold.
    pub rsi_overbought: bool,
    /// RSI below the oversold threshold.
    pub rsi_oversold: bool,
    /// MACD line above the signal line.
    pub macd_bullish: bool,
    /// Stochastic %K above the overbought threshold.
    pub stoch_overbought: bool,
    /// Stochastic %K below the oversold threshold.
    pub stoch_oversold: bool,
    /// ADX above the trending threshold.
    pub adx_trending: bool,
}

/// Signal interpretation for volatility indicators.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct VolatilitySignal {
    /// ATR percent above the elevated threshold.
    pub atr_elevated: bool,
    /// Price near the upper Bollinger band.
    pub price_near_upper_band: bool,
    /// Price near the lower Bollinger band.
    pub price_near_lower_band: bool,
}

/// Signal interpretation for volume indicators.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct VolumeSignal {
    /// Relative volume above the elevated threshold.
    pub high_relative_volume: bool,
    /// Price above VWAP.
    pub price_above_vwap: bool,
}

/// Summary of signal interpretations across all categories.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct SignalSummary {
    /// Trend signals.
    pub trend: TrendSignal,
    /// Momentum signals.
    pub momentum: MomentumSignal,
    /// Volatility signals.
    pub volatility: VolatilitySignal,
    /// Volume signals.
    pub volume: VolumeSignal,
}

/// Full dashboard output with category-grouped indicators.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct DashboardOutput {
    /// Symbol being analyzed.
    pub symbol: String,
    /// Interval used for the analysis.
    pub interval: String,
    /// Number of points included.
    pub points: usize,
    /// Trend indicator series.
    pub trend: TrendIndicators,
    /// Momentum indicator series.
    pub momentum: MomentumIndicators,
    /// Volatility indicator series.
    pub volatility: VolatilityIndicators,
    /// Volume indicator series.
    pub volume: VolumeIndicators,
    /// Derived scalar fields.
    pub derived: DerivedFields,
    /// Signal summary.
    pub signals: SignalSummary,
}

/// Expected move output from ATM straddle pricing.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct ExpectedMoveOutput {
    /// Symbol being analyzed.
    pub symbol: String,
    /// Underlying price at the time of calculation.
    pub underlying_price: f64,
    /// Expiration date used for the straddle.
    pub expiration: String,
    /// Days to expiration.
    pub dte: u32,
    /// ATM straddle price.
    pub straddle_price: f64,
    /// Expected move in price terms.
    pub expected_move: f64,
    /// Expected move as a percentage.
    pub expected_move_percent: f64,
    /// Upper expected move bound.
    pub upper_range: f64,
    /// Lower expected move bound.
    pub lower_range: f64,
    /// Optional implied volatility used in the estimate.
    pub implied_volatility: Option<f64>,
    /// ATM call price.
    pub call_price: f64,
    /// ATM put price.
    pub put_price: f64,
}

/// Per-symbol result for the analyze command (partial-failure aware).
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct AnalyzeSymbolResult {
    /// Symbol being analyzed.
    pub symbol: String,
    /// Raw quote payload when available.
    pub quote: Option<serde_json::Value>,
    /// Dashboard analysis payload when available.
    pub analysis: Option<DashboardOutput>,
    /// Expected move payload when requested and available.
    pub expected_move: Option<serde_json::Value>,
    /// Quote retrieval failure message.
    pub quote_error: Option<String>,
    /// Analysis failure message.
    pub analysis_error: Option<String>,
    /// Expected move failure message.
    pub expected_move_error: Option<String>,
}

/// Top-level analyze command output.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct AnalyzeOutput {
    /// Per-symbol results.
    pub results: Vec<AnalyzeSymbolResult>,
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn sample_point() -> TaPoint {
        TaPoint {
            timestamp: 1,
            value: 2.5,
        }
    }

    fn sample_dashboard() -> DashboardOutput {
        DashboardOutput {
            symbol: "AAPL".to_string(),
            interval: "daily".to_string(),
            points: 1,
            trend: TrendIndicators {
                sma_21: vec![sample_point()],
                sma_50: vec![sample_point()],
                sma_200: vec![sample_point()],
                ema_21: vec![sample_point()],
            },
            momentum: MomentumIndicators {
                rsi_14: vec![sample_point()],
                macd: vec![MacdPoint {
                    timestamp: 1,
                    macd: 1.0,
                    signal: 2.0,
                    histogram: 3.0,
                }],
                stochastic: vec![StochPoint {
                    timestamp: 1,
                    k: 4.0,
                    d: 5.0,
                }],
                adx: vec![AdxPoint {
                    timestamp: 1,
                    adx: 6.0,
                    plus_di: 7.0,
                    minus_di: 8.0,
                }],
            },
            volatility: VolatilityIndicators {
                atr_14: vec![sample_point()],
                bollinger_bands: vec![BbandsPoint {
                    timestamp: 1,
                    upper: 9.0,
                    middle: 10.0,
                    lower: 11.0,
                }],
                historical_volatility: vec![sample_point()],
            },
            volume: VolumeIndicators {
                vwap: vec![sample_point()],
                avg_volume_20: vec![sample_point()],
                relative_volume: None,
            },
            derived: DerivedFields {
                price_basis: "previous_close".to_string(),
                price_basis_value: 101.0,
                price_basis_timestamp: 1,
                atr_percent: 1.0,
                range_20_high: 2.0,
                range_20_low: 3.0,
                range_252_high: 4.0,
                range_252_low: 5.0,
                distance_from_sma_21: 6.0,
                distance_from_sma_50: 7.0,
                distance_from_sma_200: 8.0,
            },
            signals: SignalSummary {
                trend: TrendSignal {
                    above_sma_21: true,
                    above_sma_50: false,
                    above_sma_200: true,
                    sma_21_above_sma_50: false,
                    sma_50_above_sma_200: true,
                },
                momentum: MomentumSignal {
                    rsi_overbought: false,
                    rsi_oversold: true,
                    macd_bullish: true,
                    stoch_overbought: false,
                    stoch_oversold: true,
                    adx_trending: true,
                },
                volatility: VolatilitySignal {
                    atr_elevated: true,
                    price_near_upper_band: false,
                    price_near_lower_band: true,
                },
                volume: VolumeSignal {
                    high_relative_volume: false,
                    price_above_vwap: true,
                },
            },
        }
    }

    #[test]
    fn dashboard_output_serializes_with_category_keys() {
        let value: Value = serde_json::to_value(sample_dashboard()).unwrap();
        let object = value.as_object().unwrap();

        assert!(object.contains_key("trend"));
        assert!(object.contains_key("momentum"));
        assert!(object.contains_key("volatility"));
        assert!(object.contains_key("volume"));
        assert!(object.contains_key("derived"));
        assert!(object.contains_key("signals"));
    }

    #[test]
    fn expected_move_output_serializes_with_expected_keys() {
        let output = ExpectedMoveOutput {
            symbol: "AAPL".to_string(),
            underlying_price: 100.0,
            expiration: "2026-01-16".to_string(),
            dte: 30,
            straddle_price: 4.5,
            expected_move: 4.5,
            expected_move_percent: 4.5,
            upper_range: 104.5,
            lower_range: 95.5,
            implied_volatility: Some(0.25),
            call_price: 2.25,
            put_price: 2.25,
        };
        let value: Value = serde_json::to_value(output).unwrap();
        let object = value.as_object().unwrap();

        assert_eq!(object.get("straddle_price"), Some(&json!(4.5)));
        assert_eq!(object.get("upper_range"), Some(&json!(104.5)));
        assert_eq!(object.get("lower_range"), Some(&json!(95.5)));
    }

    #[test]
    fn volume_indicators_omit_none_relative_volume() {
        let value: Value = serde_json::to_value(VolumeIndicators {
            vwap: vec![sample_point()],
            avg_volume_20: vec![sample_point()],
            relative_volume: None,
        })
        .unwrap();
        let object = value.as_object().unwrap();

        assert!(object.get("relative_volume").is_none());
    }

    #[test]
    fn analyze_symbol_result_omits_empty_error_fields() {
        let value: Value = serde_json::to_value(AnalyzeSymbolResult {
            symbol: "AAPL".to_string(),
            quote: Some(json!({"last": 100.0})),
            analysis: Some(sample_dashboard()),
            expected_move: None,
            quote_error: None,
            analysis_error: None,
            expected_move_error: None,
        })
        .unwrap();
        let object = value.as_object().unwrap();

        assert!(object.get("quote_error").is_none());
        assert!(object.get("analysis_error").is_none());
        assert!(object.get("expected_move_error").is_none());
    }
}
