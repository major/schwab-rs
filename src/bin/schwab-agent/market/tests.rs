use super::*;

/// Convert a float literal to [`schwab::Number`] for test assertions.
///
/// Under the default feature set `Number` is `f64` so this is a no-op.
/// With `--features decimal` the value is parsed through its string
/// representation, matching the serde round-trip path.
#[cfg(not(feature = "decimal"))]
fn num(v: f64) -> schwab::Number {
    v
}

/// Convert a float literal to [`schwab::Number`] for test assertions.
///
/// Under the default feature set `Number` is `f64` so this is a no-op.
/// With `--features decimal` the value is parsed through its string
/// representation, matching the serde round-trip path.
#[cfg(feature = "decimal")]
fn num(v: f64) -> schwab::Number {
    use core::str::FromStr;
    schwab::Number::from_str(&format!("{v}")).expect("test float must be a valid Decimal")
}

use schwab::{
    AssetMainType, ContractType, EquityQuote, EquityReference, EquityResponse, ForexQuote,
    ForexReference, ForexResponse, FutureOptionQuote, FutureOptionReference, FutureOptionResponse,
    FutureQuote, FutureReference, FutureResponse, IndexQuote, IndexReference, IndexResponse,
    MutualFundQuote, MutualFundReference, MutualFundResponse, OptionQuote, OptionReference,
    OptionResponse, QuoteError,
};

// -- Equity: populated fields --

#[test]
fn summarize_equity_populated() {
    let response = EquityResponse {
        asset_main_type: Some(AssetMainType::Equity),
        asset_sub_type: None,
        extended: None,
        fundamental: None,
        quote: Some(EquityQuote {
            week_high_52: None,
            week_low_52: None,
            ask_mic_id: None,
            ask_price: Some(num(151.50)),
            ask_size: None,
            ask_time: None,
            bid_mic_id: None,
            bid_price: Some(num(151.40)),
            bid_size: None,
            bid_time: None,
            close_price: None,
            high_price: None,
            last_mic_id: None,
            last_price: Some(num(151.45)),
            last_size: None,
            low_price: None,
            mark: Some(num(151.46)),
            mark_change: None,
            mark_percent_change: None,
            net_change: Some(num(1.25)),
            net_percent_change: Some(num(0.83)),
            open_price: None,
            quote_time: Some(1700000000000),
            security_status: Some("Normal".to_string()),
            total_volume: Some(45_000_000),
            trade_time: Some(1700000001000),
            volatility: None,
        }),
        quote_type: None,
        realtime: Some(true),
        reference: Some(EquityReference {
            cusip: None,
            description: Some("Apple Inc".to_string()),
            exchange: None,
            exchange_name: Some("NASDAQ".to_string()),
            fsi_desc: None,
            htb_quantity: None,
            htb_rate: None,
            is_hard_to_borrow: None,
            is_shortable: None,
            otc_market_tier: None,
        }),
        regular: None,
        ssid: None,
        symbol: Some("AAPL".to_string()),
    };

    let summary = summarize_quote("AAPL".to_string(), QuoteResponseObject::Equity(response));

    assert_eq!(summary.requested_symbol, "AAPL");
    assert_eq!(summary.symbol, Some("AAPL".to_string()));
    assert_eq!(summary.asset_type, Some("Equity".to_string()));
    assert_eq!(summary.description, Some("Apple Inc".to_string()));
    assert_eq!(summary.exchange, Some("NASDAQ".to_string()));
    assert_eq!(summary.bid, Some(num(151.40)));
    assert_eq!(summary.ask, Some(num(151.50)));
    assert_eq!(summary.last, Some(num(151.45)));
    assert_eq!(summary.mark, Some(num(151.46)));
    assert_eq!(summary.net_change, Some(num(1.25)));
    assert_eq!(summary.net_percent_change, Some(num(0.83)));
    assert_eq!(summary.volume, Some(45_000_000));
    assert_eq!(summary.quote_time, Some(1700000000000));
    assert_eq!(summary.trade_time, Some(1700000001000));
    assert_eq!(summary.security_status, Some("Normal".to_string()));
    assert_eq!(summary.realtime, Some(true));
    // Equity never sets option-specific fields.
    assert_eq!(summary.underlying, None);
    assert_eq!(summary.put_call, None);
    assert_eq!(summary.strike_price, None);
    assert_eq!(summary.days_to_expiration, None);
    assert!(summary.error.is_none());
}

// -- Equity: all-None fields --

#[test]
fn summarize_equity_all_none() {
    let response = EquityResponse {
        asset_main_type: None,
        asset_sub_type: None,
        extended: None,
        fundamental: None,
        quote: None,
        quote_type: None,
        realtime: None,
        reference: None,
        regular: None,
        ssid: None,
        symbol: None,
    };

    let summary = summarize_quote("MISSING".to_string(), QuoteResponseObject::Equity(response));

    assert_eq!(summary.requested_symbol, "MISSING");
    assert_eq!(summary.symbol, None);
    assert_eq!(summary.asset_type, None);
    assert_eq!(summary.description, None);
    assert_eq!(summary.exchange, None);
    assert_eq!(summary.bid, None);
    assert_eq!(summary.ask, None);
    assert_eq!(summary.last, None);
    assert_eq!(summary.mark, None);
    assert_eq!(summary.net_change, None);
    assert_eq!(summary.net_percent_change, None);
    assert_eq!(summary.volume, None);
    assert_eq!(summary.quote_time, None);
    assert_eq!(summary.trade_time, None);
    assert_eq!(summary.security_status, None);
    assert_eq!(summary.realtime, None);
    assert!(summary.error.is_none());
}

// -- Option: populated with option-specific fields --

#[test]
fn summarize_option_populated() {
    let response = OptionResponse {
        asset_main_type: Some(AssetMainType::Option),
        quote: Some(OptionQuote {
            week_high_52: None,
            week_low_52: None,
            ask_price: Some(num(5.20)),
            ask_size: None,
            bid_price: Some(num(5.00)),
            bid_size: None,
            close_price: None,
            delta: None,
            gamma: None,
            high_price: None,
            implied_yield: None,
            ind_ask_price: None,
            ind_bid_price: None,
            ind_quote_time: None,
            last_price: Some(num(5.10)),
            last_size: None,
            low_price: None,
            mark: Some(num(5.10)),
            mark_change: None,
            mark_percent_change: None,
            money_intrinsic_value: None,
            net_change: Some(num(0.30)),
            net_percent_change: Some(num(6.25)),
            open_interest: None,
            open_price: None,
            quote_time: Some(1700000000000),
            rho: None,
            security_status: Some("Normal".to_string()),
            theoretical_option_value: None,
            theta: None,
            time_value: None,
            total_volume: Some(1200),
            trade_time: Some(1700000001000),
            underlying_price: None,
            vega: None,
            volatility: None,
        }),
        realtime: Some(true),
        reference: Some(OptionReference {
            contract_type: Some(ContractType::Call),
            cusip: None,
            days_to_expiration: Some(30),
            deliverables: None,
            description: Some("AAPL Jan 170 Call".to_string()),
            exchange: None,
            exchange_name: Some("CBOE".to_string()),
            exercise_type: None,
            expiration_day: None,
            expiration_month: None,
            expiration_type: None,
            expiration_year: None,
            is_penny_pilot: None,
            last_trading_day: None,
            multiplier: None,
            settlement_type: None,
            strike_price: Some(num(170.0)),
            underlying: Some("AAPL".to_string()),
        }),
        ssid: None,
        symbol: Some("AAPL  240119C00170000".to_string()),
    };

    let summary = summarize_quote(
        "AAPL_C170".to_string(),
        QuoteResponseObject::Option(response),
    );

    assert_eq!(summary.requested_symbol, "AAPL_C170");
    assert_eq!(summary.symbol, Some("AAPL  240119C00170000".to_string()));
    assert_eq!(summary.asset_type, Some("Option".to_string()));
    assert_eq!(summary.description, Some("AAPL Jan 170 Call".to_string()));
    assert_eq!(summary.exchange, Some("CBOE".to_string()));
    assert_eq!(summary.bid, Some(num(5.00)));
    assert_eq!(summary.ask, Some(num(5.20)));
    assert_eq!(summary.last, Some(num(5.10)));
    assert_eq!(summary.mark, Some(num(5.10)));
    assert_eq!(summary.net_change, Some(num(0.30)));
    assert_eq!(summary.net_percent_change, Some(num(6.25)));
    assert_eq!(summary.volume, Some(1200));
    assert_eq!(summary.quote_time, Some(1700000000000));
    assert_eq!(summary.trade_time, Some(1700000001000));
    assert_eq!(summary.security_status, Some("Normal".to_string()));
    assert_eq!(summary.realtime, Some(true));
    // Option-specific fields.
    assert_eq!(summary.underlying, Some("AAPL".to_string()));
    assert_eq!(summary.put_call, Some("Call".to_string()));
    assert_eq!(summary.strike_price, Some(num(170.0)));
    assert_eq!(summary.days_to_expiration, Some(30));
    assert!(summary.error.is_none());
}

// -- Option: all-None --

#[test]
fn summarize_option_all_none() {
    let response = OptionResponse {
        asset_main_type: None,
        quote: None,
        realtime: None,
        reference: None,
        ssid: None,
        symbol: None,
    };

    let summary = summarize_quote("OPT".to_string(), QuoteResponseObject::Option(response));

    assert_eq!(summary.requested_symbol, "OPT");
    assert_eq!(summary.symbol, None);
    assert_eq!(summary.bid, None);
    assert_eq!(summary.underlying, None);
    assert_eq!(summary.put_call, None);
    assert_eq!(summary.strike_price, None);
    assert_eq!(summary.days_to_expiration, None);
}

// -- MutualFund: populated (bid/ask always None, last/mark from nav) --

#[test]
fn summarize_mutual_fund_populated() {
    let response = MutualFundResponse {
        asset_main_type: Some(AssetMainType::MutualFund),
        asset_sub_type: None,
        fundamental: None,
        quote: Some(MutualFundQuote {
            week_high_52: None,
            week_low_52: None,
            close_price: None,
            nav: Some(num(42.50)),
            net_change: Some(num(0.15)),
            net_percent_change: Some(num(0.35)),
            security_status: Some("Normal".to_string()),
            total_volume: Some(100_000),
            trade_time: Some(1700000001000),
        }),
        realtime: Some(false),
        reference: Some(MutualFundReference {
            cusip: None,
            description: Some("Vanguard 500 Index".to_string()),
            exchange: None,
            exchange_name: Some("NASDAQ".to_string()),
        }),
        ssid: None,
        symbol: Some("VFIAX".to_string()),
    };

    let summary = summarize_quote(
        "VFIAX".to_string(),
        QuoteResponseObject::MutualFund(response),
    );

    assert_eq!(summary.requested_symbol, "VFIAX");
    assert_eq!(summary.symbol, Some("VFIAX".to_string()));
    assert_eq!(summary.asset_type, Some("MutualFund".to_string()));
    assert_eq!(summary.description, Some("Vanguard 500 Index".to_string()));
    assert_eq!(summary.exchange, Some("NASDAQ".to_string()));
    // Mutual fund: bid/ask always None, last/mark from nav.
    assert_eq!(summary.bid, None);
    assert_eq!(summary.ask, None);
    assert_eq!(summary.last, Some(num(42.50)));
    assert_eq!(summary.mark, Some(num(42.50)));
    assert_eq!(summary.net_change, Some(num(0.15)));
    assert_eq!(summary.net_percent_change, Some(num(0.35)));
    assert_eq!(summary.volume, Some(100_000));
    // Mutual fund: quote_time always None.
    assert_eq!(summary.quote_time, None);
    assert_eq!(summary.trade_time, Some(1700000001000));
    assert_eq!(summary.security_status, Some("Normal".to_string()));
    assert_eq!(summary.realtime, Some(false));
    assert!(summary.error.is_none());
}

// -- MutualFund: all-None --

#[test]
fn summarize_mutual_fund_all_none() {
    let response = MutualFundResponse {
        asset_main_type: None,
        asset_sub_type: None,
        fundamental: None,
        quote: None,
        realtime: None,
        reference: None,
        ssid: None,
        symbol: None,
    };

    let summary = summarize_quote("MF".to_string(), QuoteResponseObject::MutualFund(response));

    assert_eq!(summary.requested_symbol, "MF");
    assert_eq!(summary.symbol, None);
    assert_eq!(summary.bid, None);
    assert_eq!(summary.ask, None);
    assert_eq!(summary.last, None);
    assert_eq!(summary.mark, None);
}

// -- Forex: populated --

#[test]
fn summarize_forex_populated() {
    let response = ForexResponse {
        asset_main_type: Some(AssetMainType::Forex),
        quote: Some(ForexQuote {
            week_high_52: None,
            week_low_52: None,
            ask_price: Some(num(1.13456)),
            ask_size: None,
            bid_price: Some(num(1.13434)),
            bid_size: None,
            close_price: None,
            high_price: None,
            last_price: Some(num(1.13445)),
            last_size: None,
            low_price: None,
            mark: Some(num(1.13445)),
            net_change: Some(num(0.00254)),
            net_percent_change: Some(num(0.22)),
            open_price: None,
            quote_time: Some(1637236739892),
            security_status: Some("Unknown".to_string()),
            tick: None,
            tick_amount: None,
            total_volume: Some(0),
            trade_time: Some(1637236739892),
        }),
        realtime: Some(true),
        reference: Some(ForexReference {
            description: Some("Euro/USDollar Spot".to_string()),
            exchange: None,
            exchange_name: None,
            is_tradable: None,
            market_maker: None,
            product: None,
            trading_hours: None,
        }),
        ssid: None,
        symbol: Some("EUR/USD".to_string()),
    };

    let summary = summarize_quote("EUR/USD".to_string(), QuoteResponseObject::Forex(response));

    assert_eq!(summary.requested_symbol, "EUR/USD");
    assert_eq!(summary.symbol, Some("EUR/USD".to_string()));
    assert_eq!(summary.asset_type, Some("Forex".to_string()));
    assert_eq!(summary.description, Some("Euro/USDollar Spot".to_string()));
    // simple_quote path: exchange always None.
    assert_eq!(summary.exchange, None);
    assert_eq!(summary.bid, Some(num(1.13434)));
    assert_eq!(summary.ask, Some(num(1.13456)));
    assert_eq!(summary.last, Some(num(1.13445)));
    assert_eq!(summary.mark, Some(num(1.13445)));
    assert_eq!(summary.net_change, Some(num(0.00254)));
    assert_eq!(summary.net_percent_change, Some(num(0.22)));
    assert_eq!(summary.volume, Some(0));
    assert_eq!(summary.quote_time, Some(1637236739892));
    assert_eq!(summary.trade_time, Some(1637236739892));
    assert_eq!(summary.security_status, Some("Unknown".to_string()));
    assert_eq!(summary.realtime, Some(true));
    assert_eq!(summary.underlying, None);
    assert!(summary.error.is_none());
}

// -- Forex: all-None --

#[test]
fn summarize_forex_all_none() {
    let response = ForexResponse {
        asset_main_type: None,
        quote: None,
        realtime: None,
        reference: None,
        ssid: None,
        symbol: None,
    };

    let summary = summarize_quote("FX".to_string(), QuoteResponseObject::Forex(response));

    assert_eq!(summary.requested_symbol, "FX");
    assert_eq!(summary.symbol, None);
    assert_eq!(summary.asset_type, None);
    assert_eq!(summary.description, None);
    assert_eq!(summary.bid, None);
    assert_eq!(summary.last, None);
}

// -- Future: populated (uses future_percent_change for net_percent_change) --

#[test]
fn summarize_future_populated() {
    let response = FutureResponse {
        asset_main_type: Some(AssetMainType::Future),
        quote: Some(FutureQuote {
            ask_mic_id: None,
            ask_price: Some(num(4500.50)),
            ask_size: None,
            ask_time: None,
            bid_mic_id: None,
            bid_price: Some(num(4500.25)),
            bid_size: None,
            bid_time: None,
            close_price: None,
            future_percent_change: Some(num(0.45)),
            high_price: None,
            last_mic_id: None,
            last_price: Some(num(4500.375)),
            last_size: None,
            low_price: None,
            mark: Some(num(4500.40)),
            net_change: Some(num(20.125)),
            open_interest: None,
            open_price: None,
            quote_time: Some(1700000000000),
            quoted_in_session: None,
            security_status: Some("Normal".to_string()),
            settle_time: None,
            tick: None,
            tick_amount: None,
            total_volume: Some(500_000),
            trade_time: Some(1700000001000),
        }),
        realtime: Some(true),
        reference: Some(FutureReference {
            description: Some("E-mini S&P 500".to_string()),
            exchange: None,
            exchange_name: None,
            future_active_symbol: None,
            future_expiration_date: None,
            future_is_active: None,
            future_is_tradable: None,
            future_multiplier: None,
            future_price_format: None,
            future_settlement_price: None,
            future_trading_hours: None,
            product: None,
        }),
        ssid: None,
        symbol: Some("/ES".to_string()),
    };

    let summary = summarize_quote("/ES".to_string(), QuoteResponseObject::Future(response));

    assert_eq!(summary.requested_symbol, "/ES");
    assert_eq!(summary.symbol, Some("/ES".to_string()));
    assert_eq!(summary.asset_type, Some("Future".to_string()));
    assert_eq!(summary.description, Some("E-mini S&P 500".to_string()));
    assert_eq!(summary.bid, Some(num(4500.25)));
    assert_eq!(summary.ask, Some(num(4500.50)));
    assert_eq!(summary.last, Some(num(4500.375)));
    assert_eq!(summary.mark, Some(num(4500.40)));
    assert_eq!(summary.net_change, Some(num(20.125)));
    // Future maps future_percent_change to net_percent_change.
    assert_eq!(summary.net_percent_change, Some(num(0.45)));
    assert_eq!(summary.volume, Some(500_000));
    assert_eq!(summary.quote_time, Some(1700000000000));
    assert_eq!(summary.trade_time, Some(1700000001000));
    assert_eq!(summary.security_status, Some("Normal".to_string()));
    assert_eq!(summary.exchange, None);
    assert!(summary.error.is_none());
}

// -- Future: all-None --

#[test]
fn summarize_future_all_none() {
    let response = FutureResponse {
        asset_main_type: None,
        quote: None,
        realtime: None,
        reference: None,
        ssid: None,
        symbol: None,
    };

    let summary = summarize_quote("FUT".to_string(), QuoteResponseObject::Future(response));

    assert_eq!(summary.requested_symbol, "FUT");
    assert_eq!(summary.symbol, None);
    assert_eq!(summary.net_percent_change, None);
}

// -- FutureOption: populated --

#[test]
fn summarize_future_option_populated() {
    let response = FutureOptionResponse {
        asset_main_type: Some(AssetMainType::FutureOption),
        quote: Some(FutureOptionQuote {
            ask_mic_id: None,
            ask_price: Some(num(12.50)),
            ask_size: None,
            bid_mic_id: None,
            bid_price: Some(num(12.00)),
            bid_size: None,
            close_price: None,
            high_price: None,
            last_mic_id: None,
            last_price: Some(num(12.25)),
            last_size: None,
            low_price: None,
            mark: Some(num(12.25)),
            mark_change: None,
            net_change: Some(num(0.75)),
            net_percent_change: Some(num(6.52)),
            open_interest: None,
            open_price: None,
            quote_time: Some(1700000000000),
            security_status: Some("Normal".to_string()),
            settlemet_price: None,
            tick: None,
            tick_amount: None,
            total_volume: Some(300),
            trade_time: Some(1700000001000),
        }),
        realtime: Some(true),
        reference: Some(FutureOptionReference {
            contract_type: None,
            description: Some("ES Dec 4500 Call".to_string()),
            exchange: None,
            exchange_name: None,
            expiration_date: None,
            expiration_style: None,
            multiplier: None,
            strike_price: None,
            underlying: None,
        }),
        ssid: None,
        symbol: Some("./ESZ3C4500".to_string()),
    };

    let summary = summarize_quote(
        "./ESZ3C4500".to_string(),
        QuoteResponseObject::FutureOption(response),
    );

    assert_eq!(summary.requested_symbol, "./ESZ3C4500");
    assert_eq!(summary.symbol, Some("./ESZ3C4500".to_string()));
    assert_eq!(summary.asset_type, Some("FutureOption".to_string()));
    assert_eq!(summary.description, Some("ES Dec 4500 Call".to_string()));
    assert_eq!(summary.bid, Some(num(12.00)));
    assert_eq!(summary.ask, Some(num(12.50)));
    assert_eq!(summary.last, Some(num(12.25)));
    assert_eq!(summary.net_change, Some(num(0.75)));
    assert_eq!(summary.net_percent_change, Some(num(6.52)));
    assert_eq!(summary.volume, Some(300));
    assert_eq!(summary.exchange, None);
    assert!(summary.error.is_none());
}

// -- FutureOption: all-None --

#[test]
fn summarize_future_option_all_none() {
    let response = FutureOptionResponse {
        asset_main_type: None,
        quote: None,
        realtime: None,
        reference: None,
        ssid: None,
        symbol: None,
    };

    let summary = summarize_quote(
        "FO".to_string(),
        QuoteResponseObject::FutureOption(response),
    );

    assert_eq!(summary.requested_symbol, "FO");
    assert_eq!(summary.symbol, None);
    assert_eq!(summary.bid, None);
}

// -- Index: populated (bid/ask None, mark = last_price, quote_time None) --

#[test]
fn summarize_index_populated() {
    let response = IndexResponse {
        asset_main_type: Some(AssetMainType::Index),
        quote: Some(IndexQuote {
            week_high_52: None,
            week_low_52: None,
            close_price: None,
            high_price: None,
            last_price: Some(num(34436.13)),
            low_price: None,
            net_change: Some(num(150.0)),
            net_percent_change: Some(num(0.44)),
            open_price: None,
            security_status: Some("Normal".to_string()),
            total_volume: Some(628_009_977),
            trade_time: Some(1700000001000),
        }),
        realtime: Some(true),
        reference: Some(IndexReference {
            description: Some("Dow Jones Industrial Average".to_string()),
            exchange: None,
            exchange_name: None,
        }),
        ssid: None,
        symbol: Some("$DJI".to_string()),
    };

    let summary = summarize_quote("$DJI".to_string(), QuoteResponseObject::Index(response));

    assert_eq!(summary.requested_symbol, "$DJI");
    assert_eq!(summary.symbol, Some("$DJI".to_string()));
    assert_eq!(summary.asset_type, Some("Index".to_string()));
    assert_eq!(
        summary.description,
        Some("Dow Jones Industrial Average".to_string()),
    );
    // Index: bid/ask are None, mark equals last_price, quote_time is None.
    assert_eq!(summary.bid, None);
    assert_eq!(summary.ask, None);
    assert_eq!(summary.last, Some(num(34436.13)));
    assert_eq!(summary.mark, Some(num(34436.13)));
    assert_eq!(summary.net_change, Some(num(150.0)));
    assert_eq!(summary.net_percent_change, Some(num(0.44)));
    assert_eq!(summary.volume, Some(628_009_977));
    assert_eq!(summary.quote_time, None);
    assert_eq!(summary.trade_time, Some(1700000001000));
    assert_eq!(summary.security_status, Some("Normal".to_string()));
    assert_eq!(summary.realtime, Some(true));
    assert_eq!(summary.exchange, None);
    assert!(summary.error.is_none());
}

// -- Index: all-None --

#[test]
fn summarize_index_all_none() {
    let response = IndexResponse {
        asset_main_type: None,
        quote: None,
        realtime: None,
        reference: None,
        ssid: None,
        symbol: None,
    };

    let summary = summarize_quote("IDX".to_string(), QuoteResponseObject::Index(response));

    assert_eq!(summary.requested_symbol, "IDX");
    assert_eq!(summary.symbol, None);
    assert_eq!(summary.bid, None);
    assert_eq!(summary.ask, None);
    assert_eq!(summary.last, None);
    assert_eq!(summary.mark, None);
    assert_eq!(summary.quote_time, None);
}

// -- Error variant --

#[test]
fn summarize_error_with_invalid_symbols() {
    let error = QuoteError {
        invalid_symbols: Some(vec!["FAKESYM".to_string(), "NOTREAL".to_string()]),
        invalid_cusips: Some(vec!["000000000".to_string()]),
        invalid_ssids: Some(vec![12345, 67890]),
    };

    let summary = summarize_quote("FAKESYM".to_string(), QuoteResponseObject::Error(error));

    assert_eq!(summary.requested_symbol, "FAKESYM");
    assert_eq!(summary.symbol, None);
    assert_eq!(summary.asset_type, None);
    assert_eq!(summary.bid, None);
    assert_eq!(summary.realtime, None);

    let err = summary.error.expect("error field should be Some");
    assert_eq!(
        err.invalid_symbols,
        Some(vec!["FAKESYM".to_string(), "NOTREAL".to_string()]),
    );
    assert_eq!(err.invalid_cusips, Some(vec!["000000000".to_string()]));
    assert_eq!(err.invalid_ssids, Some(vec![12345, 67890]));
}

#[test]
fn summarize_error_all_none() {
    let error = QuoteError {
        invalid_symbols: None,
        invalid_cusips: None,
        invalid_ssids: None,
    };

    let summary = summarize_quote("ERR".to_string(), QuoteResponseObject::Error(error));

    let err = summary.error.expect("error field should be Some");
    assert_eq!(err.invalid_symbols, None);
    assert_eq!(err.invalid_cusips, None);
    assert_eq!(err.invalid_ssids, None);
}

// -- Quote output serialization: verifies compact and full-detail shapes --

#[test]
fn selected_history_fields_defaults_to_compact_columns() {
    let fields = selected_history_fields(None).expect("default fields should be valid");

    assert_eq!(fields, vec!["ts", "open", "high", "low", "close", "vol"]);
}

#[test]
fn selected_history_fields_accepts_aliases() {
    let fields = selected_history_fields(Some("datetime,o,h,l,c,volume,datetimeISO8601"))
        .expect("known aliases should be valid");

    assert_eq!(
        fields,
        vec!["ts", "open", "high", "low", "close", "vol", "iso"]
    );
}

#[test]
fn selected_history_fields_rejects_unknown_fields() {
    let error = selected_history_fields(Some("close,nope"))
        .expect_err("unknown field should fail validation");

    assert!(matches!(error, AppError::MarketValidation { .. }));
    assert!(error.to_string().contains("unknown history field(s): nope"));
}

#[test]
fn selected_history_fields_rejects_empty_lists() {
    let error =
        selected_history_fields(Some(",, ")).expect_err("empty field list should fail validation");

    assert!(matches!(error, AppError::MarketValidation { .. }));
    assert!(
        error
            .to_string()
            .contains("history --fields cannot be empty")
    );
}

#[test]
fn parse_history_date_only_start_produces_midnight_utc_ms() {
    let millis = parse_history_instant("2026-01-15", HistoryRangeBoundary::Start)
        .expect("date-only start should parse");

    assert_eq!(millis, 1_768_435_200_000);
}

#[test]
fn parse_history_date_only_end_produces_end_of_day_utc_ms() {
    let millis = parse_history_instant("2026-01-15", HistoryRangeBoundary::End)
        .expect("date-only end should parse");

    assert_eq!(millis, 1_768_521_599_999);
}

#[test]
fn parse_history_rfc3339_produces_epoch_ms() {
    let millis = parse_history_instant("2026-01-15T09:30:45.123Z", HistoryRangeBoundary::Start)
        .expect("RFC3339 instant should parse");

    assert_eq!(millis, 1_768_469_445_123);
}

#[test]
fn parse_history_epoch_ms_passthrough_is_unchanged() {
    let millis = parse_history_instant("1700000000000", HistoryRangeBoundary::Start)
        .expect("epoch milliseconds should parse");

    assert_eq!(millis, 1_700_000_000_000);
}

#[test]
fn parse_history_invalid_date_rejects_before_api_call() {
    let error = parse_history_instant("2026-02-30", HistoryRangeBoundary::Start)
        .expect_err("invalid calendar date should fail");

    assert!(matches!(error, AppError::MarketValidation { .. }));
    assert!(error.to_string().contains("invalid market history date"));
    assert!(error.to_string().contains("2026-02-30"));
}

#[test]
fn parse_history_invalid_string_rejects_before_api_call() {
    let error = parse_history_instant("not-a-date", HistoryRangeBoundary::Start)
        .expect_err("invalid text should fail");

    assert!(matches!(error, AppError::MarketValidation { .. }));
    assert!(error.to_string().contains("expected YYYY-MM-DD"));
    assert!(error.to_string().contains("not-a-date"));
}

#[test]
fn history_rows_output_serializes_compact_table() {
    let history = serde_json::json!({
        "symbol": "SPY",
        "empty": false,
        "previousClose": 650.0,
        "candles": [
            {
                "datetime": 1_700_000_000_000_i64,
                "open": 650.0,
                "high": 651.5,
                "low": 649.5,
                "close": 651.0,
                "volume": 12_345
            }
        ]
    });

    let fields = selected_history_fields(None).expect("default fields should be valid");
    let output = select_history_fields(&history, &fields);
    let json = serde_json::to_value(&output).unwrap();

    assert_eq!(json["symbol"], serde_json::json!("SPY"));
    assert_eq!(
        json["columns"],
        serde_json::json!(["ts", "open", "high", "low", "close", "vol"])
    );
    assert_eq!(json["rowCount"], 1);
    assert_eq!(
        json["rows"],
        serde_json::json!([[1_700_000_000_000_i64, 650.0, 651.5, 649.5, 651.0, 12_345]])
    );
}

#[test]
fn history_rows_output_selects_extra_iso_field() {
    let history = serde_json::json!({
        "symbol": "SPY",
        "candles": [
            {
                "datetime": 1_700_000_000_000_i64,
                "datetimeISO8601": "2023-11-14T22:13:20Z",
                "close": 651.0,
                "volume": 12_345
            }
        ]
    });

    let fields = selected_history_fields(Some("timestamp,datetime_iso8601,close,volume"))
        .expect("known aliases should be valid");
    let output = select_history_fields(&history, &fields);
    let json = serde_json::to_value(&output).unwrap();

    assert_eq!(
        json["columns"],
        serde_json::json!(["ts", "iso", "close", "vol"])
    );
    assert_eq!(
        json["rows"],
        serde_json::json!([[1_700_000_000_000_i64, "2023-11-14T22:13:20Z", 651.0, 12_345]])
    );
}

#[test]
fn history_rows_output_includes_null_for_missing_selected_fields() {
    let history = serde_json::json!({
        "symbol": "SPY",
        "candles": [{"datetime": 1_700_000_000_000_i64, "close": 651.0}]
    });

    let output = select_history_fields(&history, &["ts", "iso", "open", "close", "vol"]);
    let json = serde_json::to_value(&output).unwrap();

    assert_eq!(
        json["columns"],
        serde_json::json!(["ts", "iso", "open", "close", "vol"])
    );
    assert_eq!(json["rowCount"], 1);
    assert_eq!(
        json["rows"],
        serde_json::json!([[1_700_000_000_000_i64, null, null, 651.0, null]])
    );
}

#[test]
fn history_rows_output_handles_empty_candles() {
    let history = serde_json::json!({"symbol": "SPY", "candles": []});

    let output = select_history_fields(&history, &["ts", "close"]);
    let json = serde_json::to_value(&output).unwrap();

    assert_eq!(json["symbol"], serde_json::json!("SPY"));
    assert_eq!(json["columns"], serde_json::json!(["ts", "close"]));
    assert_eq!(json["rowCount"], 0);
    assert_eq!(json["rows"], serde_json::json!([]));
}

#[test]
fn selected_quote_fields_defaults_to_compact_columns() {
    let fields = selected_quote_fields(None).expect("default fields should be valid");

    assert_eq!(
        fields,
        vec![
            "req", "sym", "bid", "ask", "last", "mark", "chg", "pct", "vol", "err"
        ]
    );
}

#[test]
fn default_quote_rows_show_requested_symbol_and_errors() {
    let fields = selected_quote_fields(None).expect("default fields should be valid");
    let summaries = vec![summarize_quote(
        "BAD".to_string(),
        QuoteResponseObject::Error(QuoteError {
            invalid_symbols: Some(vec!["BAD".to_string()]),
            invalid_cusips: None,
            invalid_ssids: None,
        }),
    )];

    let output = select_quote_fields(&summaries, &fields);
    let json = serde_json::to_value(&output).unwrap();

    assert_eq!(
        json["columns"],
        serde_json::json!([
            "req", "sym", "bid", "ask", "last", "mark", "chg", "pct", "vol", "err"
        ])
    );
    assert_eq!(json["rowCount"], 1);
    assert_eq!(
        json["rows"],
        serde_json::json!([["BAD", null, null, null, null, null, null, null, null, {"invalid_symbols": ["BAD"]}]])
    );
}

#[test]
fn normalize_quote_summaries_replaces_generic_error_rows() {
    let summaries = vec![QuoteSummary {
        requested_symbol: "errors".to_string(),
        ..QuoteSummary::default()
    }];
    let requested_symbols = vec!["BAD".to_string()];

    let output = normalize_quote_summaries(summaries, &requested_symbols);

    assert_eq!(output.len(), 1);
    assert_eq!(output[0].requested_symbol, "BAD");
    assert_eq!(
        output[0]
            .error
            .as_ref()
            .and_then(|error| error.invalid_symbols.as_ref()),
        Some(&vec!["BAD".to_string()])
    );
}

#[test]
fn normalize_quote_summaries_preserves_generic_error_details() {
    let summaries = vec![QuoteSummary {
        requested_symbol: "errors".to_string(),
        error: Some(QuoteErrorSummary {
            invalid_symbols: None,
            invalid_cusips: Some(vec!["000000000".to_string()]),
            invalid_ssids: Some(vec![12345]),
        }),
        ..QuoteSummary::default()
    }];
    let requested_symbols = vec!["000000000".to_string()];

    let output = normalize_quote_summaries(summaries, &requested_symbols);

    assert_eq!(output.len(), 1);
    assert_eq!(output[0].requested_symbol, "000000000");
    let error = output[0]
        .error
        .as_ref()
        .expect("generic API error details should be preserved");
    assert_eq!(error.invalid_symbols, None);
    assert_eq!(error.invalid_cusips, Some(vec!["000000000".to_string()]));
    assert_eq!(error.invalid_ssids, Some(vec![12345]));
}

#[test]
fn normalize_quote_summaries_applies_matching_generic_symbol_errors() {
    let summaries = vec![QuoteSummary {
        requested_symbol: "errors".to_string(),
        error: Some(QuoteErrorSummary {
            invalid_symbols: Some(vec!["BAD".to_string()]),
            invalid_cusips: Some(vec!["111111111".to_string()]),
            invalid_ssids: None,
        }),
        ..QuoteSummary::default()
    }];
    let requested_symbols = vec!["BAD".to_string(), "ALSO_BAD".to_string()];

    let output = normalize_quote_summaries(summaries, &requested_symbols);

    assert_eq!(output.len(), 2);
    let also_bad = output
        .iter()
        .find(|summary| summary.requested_symbol == "ALSO_BAD")
        .expect("missing requested symbol should receive fallback error");
    assert_eq!(
        also_bad
            .error
            .as_ref()
            .and_then(|error| error.invalid_symbols.as_ref()),
        Some(&vec!["ALSO_BAD".to_string()])
    );

    let bad = output
        .iter()
        .find(|summary| summary.requested_symbol == "BAD")
        .expect("matching requested symbol should receive API error details");
    let error = bad
        .error
        .as_ref()
        .expect("matching generic error should be preserved");
    assert_eq!(error.invalid_symbols, Some(vec!["BAD".to_string()]));
    assert_eq!(error.invalid_cusips, Some(vec!["111111111".to_string()]));
}

#[test]
fn normalize_quote_summaries_keeps_matching_api_rows() {
    let summaries = vec![QuoteSummary {
        requested_symbol: "aapl".to_string(),
        symbol: Some("AAPL".to_string()),
        ..QuoteSummary::default()
    }];
    let requested_symbols = vec!["AAPL".to_string()];

    let output = normalize_quote_summaries(summaries, &requested_symbols);

    assert_eq!(output.len(), 1);
    assert_eq!(output[0].requested_symbol, "aapl");
    assert_eq!(output[0].symbol, Some("AAPL".to_string()));
    assert!(output[0].error.is_none());
}

#[test]
fn sort_quote_summaries_preserves_generic_errors_for_all_fields() {
    let summaries = vec![
        QuoteSummary {
            requested_symbol: "MSFT".to_string(),
            symbol: Some("MSFT".to_string()),
            ..QuoteSummary::default()
        },
        QuoteSummary {
            requested_symbol: "errors".to_string(),
            error: Some(QuoteErrorSummary {
                invalid_symbols: Some(vec!["BAD".to_string()]),
                invalid_cusips: None,
                invalid_ssids: None,
            }),
            ..QuoteSummary::default()
        },
    ];

    let output = sort_quote_summaries(summaries);

    assert_eq!(output.len(), 2);
    assert_eq!(output[0].requested_symbol, "MSFT");
    assert_eq!(output[1].requested_symbol, "errors");
    assert_eq!(
        output[1]
            .error
            .as_ref()
            .and_then(|error| error.invalid_symbols.as_ref()),
        Some(&vec!["BAD".to_string()])
    );
}

#[test]
fn selected_quote_fields_accepts_aliases() {
    let fields = selected_quote_fields(Some("symbol,net_change,net_percent_change,volume"))
        .expect("known aliases should be valid");

    assert_eq!(fields, vec!["sym", "chg", "pct", "vol"]);
}

#[test]
fn selected_quote_fields_rejects_unknown_fields() {
    let error = selected_quote_fields(Some("symbol,nope"))
        .expect_err("unknown field should fail validation");

    assert!(matches!(error, AppError::MarketValidation { .. }));
    assert!(error.to_string().contains("unknown quote field(s): nope"));
}

#[test]
fn selected_quote_fields_rejects_empty_lists() {
    let error =
        selected_quote_fields(Some(",, ")).expect_err("empty field list should fail validation");

    assert!(matches!(error, AppError::MarketValidation { .. }));
    assert!(error.to_string().contains("quote --fields cannot be empty"));
}

#[test]
fn quote_rows_output_serializes_compact_table() {
    let summaries = vec![summarize_quote(
        "AAPL".to_string(),
        QuoteResponseObject::Equity(EquityResponse {
            asset_main_type: Some(AssetMainType::Equity),
            asset_sub_type: None,
            extended: None,
            fundamental: None,
            quote: Some(EquityQuote {
                week_high_52: None,
                week_low_52: None,
                ask_mic_id: None,
                ask_price: Some(num(151.50)),
                ask_size: None,
                ask_time: None,
                bid_mic_id: None,
                bid_price: Some(num(151.40)),
                bid_size: None,
                bid_time: None,
                close_price: None,
                high_price: None,
                last_mic_id: None,
                last_price: Some(num(151.45)),
                last_size: None,
                low_price: None,
                mark: Some(num(151.46)),
                mark_change: None,
                mark_percent_change: None,
                net_change: Some(num(1.25)),
                net_percent_change: Some(num(0.83)),
                open_price: None,
                quote_time: None,
                security_status: None,
                total_volume: Some(45_000_000),
                trade_time: None,
                volatility: None,
            }),
            quote_type: None,
            realtime: None,
            reference: None,
            regular: None,
            ssid: None,
            symbol: Some("AAPL".to_string()),
        }),
    )];

    let output = select_quote_fields(&summaries, &["sym", "bid", "ask", "chg", "pct", "vol"]);
    let json = serde_json::to_value(&output).unwrap();

    assert_eq!(
        json["columns"],
        serde_json::json!(["sym", "bid", "ask", "chg", "pct", "vol"])
    );
    assert_eq!(json["rowCount"], 1);
    #[cfg(not(feature = "decimal"))]
    assert_eq!(
        json["rows"],
        serde_json::json!([["AAPL", 151.4, 151.5, 1.25, 0.83, 45_000_000]])
    );
    #[cfg(feature = "decimal")]
    assert_eq!(
        json["rows"],
        serde_json::json!([["AAPL", "151.4", "151.5", "1.25", "0.83", 45_000_000]])
    );
}

#[test]
fn quote_rows_output_includes_null_for_missing_selected_fields() {
    let summaries = vec![summarize_quote(
        "$DJI".to_string(),
        QuoteResponseObject::Index(IndexResponse {
            asset_main_type: Some(AssetMainType::Index),
            quote: None,
            realtime: None,
            reference: None,
            ssid: None,
            symbol: Some("$DJI".to_string()),
        }),
    )];

    let output = select_quote_fields(&summaries, &["sym", "bid", "ask"]);
    let json = serde_json::to_value(&output).unwrap();

    assert_eq!(json["columns"], serde_json::json!(["sym", "bid", "ask"]));
    assert_eq!(json["rowCount"], 1);
    assert_eq!(json["rows"], serde_json::json!([["$DJI", null, null]]));
}

#[test]
fn quote_rows_output_can_select_error_details() {
    let summaries = vec![summarize_quote(
        "BAD".to_string(),
        QuoteResponseObject::Error(QuoteError {
            invalid_symbols: Some(vec!["BAD".to_string()]),
            invalid_cusips: None,
            invalid_ssids: None,
        }),
    )];

    let output = select_quote_fields(&summaries, &["req", "err"]);
    let json = serde_json::to_value(&output).unwrap();

    assert_eq!(json["columns"], serde_json::json!(["req", "err"]));
    assert_eq!(json["rowCount"], 1);
    assert_eq!(
        json["rows"],
        serde_json::json!([["BAD", {"invalid_symbols": ["BAD"]}]])
    );
}

#[test]
fn quote_rows_output_handles_empty_summaries() {
    let output = select_quote_fields(&[], &["sym", "last"]);
    let json = serde_json::to_value(&output).unwrap();

    assert_eq!(json["columns"], serde_json::json!(["sym", "last"]));
    assert_eq!(json["rowCount"], 0);
    assert_eq!(json["rows"], serde_json::json!([]));
}

#[test]
fn quote_output_serializes_sorted() {
    let output = QuoteOutput {
        symbols: vec!["AAPL".to_string(), "$DJI".to_string()],
        quotes: vec![
            summarize_quote(
                "$DJI".to_string(),
                QuoteResponseObject::Index(IndexResponse {
                    asset_main_type: Some(AssetMainType::Index),
                    quote: None,
                    realtime: None,
                    reference: None,
                    ssid: None,
                    symbol: Some("$DJI".to_string()),
                }),
            ),
            summarize_quote(
                "AAPL".to_string(),
                QuoteResponseObject::Equity(EquityResponse {
                    asset_main_type: Some(AssetMainType::Equity),
                    asset_sub_type: None,
                    extended: None,
                    fundamental: None,
                    quote: None,
                    quote_type: None,
                    realtime: None,
                    reference: None,
                    regular: None,
                    ssid: None,
                    symbol: Some("AAPL".to_string()),
                }),
            ),
        ],
    };

    let json = serde_json::to_value(&output).unwrap();
    let quotes = json["quotes"].as_array().unwrap();
    assert_eq!(quotes.len(), 2);
    // Verify both have requested_symbol present.
    assert_eq!(quotes[0]["requested_symbol"], "$DJI");
    assert_eq!(quotes[1]["requested_symbol"], "AAPL");
    // None fields should be absent due to skip_serializing_if.
    assert!(quotes[0].get("bid").is_none());
    assert!(quotes[0].get("error").is_none());
}

// -- QuoteSummary serialization: populated fields appear, None fields absent --

#[test]
fn quote_summary_serialization_skip_none() {
    let summary = QuoteSummary {
        requested_symbol: "TEST".to_string(),
        symbol: Some("TEST".to_string()),
        asset_type: None,
        description: None,
        exchange: None,
        bid: Some(num(10.0)),
        ask: None,
        last: None,
        mark: None,
        net_change: None,
        net_percent_change: None,
        volume: None,
        quote_time: None,
        trade_time: None,
        security_status: None,
        realtime: None,
        underlying: None,
        put_call: None,
        strike_price: None,
        days_to_expiration: None,
        error: None,
    };

    let json = serde_json::to_value(&summary).unwrap();
    assert_eq!(json["requested_symbol"], "TEST");
    assert_eq!(json["symbol"], "TEST");
    #[cfg(not(feature = "decimal"))]
    assert_eq!(json["bid"], 10.0);
    #[cfg(feature = "decimal")]
    assert_eq!(json["bid"], "10");
    // None fields should be entirely absent.
    assert!(json.get("asset_type").is_none());
    assert!(json.get("ask").is_none());
    assert!(json.get("error").is_none());
    assert!(json.get("underlying").is_none());
}

// -- QuoteErrorSummary serialization --

#[test]
fn quote_error_summary_serialization() {
    let err = QuoteErrorSummary {
        invalid_symbols: Some(vec!["BAD".to_string()]),
        invalid_cusips: None,
        invalid_ssids: None,
    };

    let json = serde_json::to_value(&err).unwrap();
    assert_eq!(json["invalid_symbols"], serde_json::json!(["BAD"]));
    // None fields absent.
    assert!(json.get("invalid_cusips").is_none());
    assert!(json.get("invalid_ssids").is_none());
}

// -- Sorting verification: summaries sorted by requested_symbol --

#[test]
fn summaries_sort_by_requested_symbol() {
    let mut summaries = [
        summarize_quote(
            "ZZZ".to_string(),
            QuoteResponseObject::Error(QuoteError {
                invalid_symbols: None,
                invalid_cusips: None,
                invalid_ssids: None,
            }),
        ),
        summarize_quote(
            "AAA".to_string(),
            QuoteResponseObject::Error(QuoteError {
                invalid_symbols: None,
                invalid_cusips: None,
                invalid_ssids: None,
            }),
        ),
        summarize_quote(
            "MMM".to_string(),
            QuoteResponseObject::Error(QuoteError {
                invalid_symbols: None,
                invalid_cusips: None,
                invalid_ssids: None,
            }),
        ),
    ];

    summaries.sort_by(|left, right| left.requested_symbol.cmp(&right.requested_symbol));

    assert_eq!(summaries[0].requested_symbol, "AAA");
    assert_eq!(summaries[1].requested_symbol, "MMM");
    assert_eq!(summaries[2].requested_symbol, "ZZZ");
}

// -- Option with Put contract type --

#[test]
fn summarize_option_put_contract() {
    let response = OptionResponse {
        asset_main_type: Some(AssetMainType::Option),
        quote: None,
        realtime: None,
        reference: Some(OptionReference {
            contract_type: Some(ContractType::Put),
            cusip: None,
            days_to_expiration: Some(15),
            deliverables: None,
            description: None,
            exchange: None,
            exchange_name: None,
            exercise_type: None,
            expiration_day: None,
            expiration_month: None,
            expiration_type: None,
            expiration_year: None,
            is_penny_pilot: None,
            last_trading_day: None,
            multiplier: None,
            settlement_type: None,
            strike_price: Some(num(150.0)),
            underlying: Some("AAPL".to_string()),
        }),
        ssid: None,
        symbol: None,
    };

    let summary = summarize_quote(
        "AAPL_P150".to_string(),
        QuoteResponseObject::Option(response),
    );

    assert_eq!(summary.put_call, Some("Put".to_string()));
    assert_eq!(summary.strike_price, Some(num(150.0)));
    assert_eq!(summary.days_to_expiration, Some(15));
    assert_eq!(summary.underlying, Some("AAPL".to_string()));
}
