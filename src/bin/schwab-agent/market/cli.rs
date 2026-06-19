use clap::{Args, Subcommand};

/// Market-data commands.
#[derive(Debug, Subcommand)]
pub enum MarketCommand {
    /// Get price history candles for a symbol.
    History(HistoryArgs),
    /// Get compact quote summaries for one or more symbols.
    Quote(QuoteArgs),
}

/// Arguments for `market history`.
#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  \
    schwab-agent market history SPY\n      \
    Get compact default candles for SPY.\n\n  \
    schwab-agent market history SPY --from 2026-01-01 --to 2026-01-31 --fields ts,close,vol\n      \
    Get an inclusive date range with selected candle columns.\n\n  \
    schwab-agent market history AAPL --period-type day --period 5 --frequency-type minute --frequency 5 --extended-hours\n      \
    Get recent 5-minute candles including extended-hours data.")]
pub struct HistoryArgs {
    /// Ticker symbol, for example AAPL.
    #[arg(required = true)]
    pub symbol: String,

    /// Comma-separated output fields. Defaults to ts,open,high,low,close,vol.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,

    /// Return the full Schwab price history object instead of compact rows.
    #[arg(long, conflicts_with = "fields")]
    pub all_fields: bool,

    /// Period type (day, month, year, ytd).
    #[arg(long)]
    pub period_type: Option<String>,

    /// Number of periods to return.
    #[arg(long)]
    pub period: Option<i64>,

    /// Frequency type (minute, daily, weekly, monthly).
    #[arg(long)]
    pub frequency_type: Option<String>,

    /// Frequency value (e.g. 1, 5, 15).
    #[arg(long)]
    pub frequency: Option<i64>,

    /// Start date as YYYY-MM-DD, RFC3339, or epoch milliseconds.
    #[arg(
        long,
        long_help = "Start date as YYYY-MM-DD, RFC3339, or epoch milliseconds. Examples: 2026-01-01, 2026-01-01T09:30:00Z, 1767225600000."
    )]
    pub from: Option<String>,

    /// End date as YYYY-MM-DD, RFC3339, or epoch milliseconds.
    #[arg(
        long,
        long_help = "End date as YYYY-MM-DD, RFC3339, or epoch milliseconds. Examples: 2026-01-31, 2026-01-31T16:00:00Z, 1769817600000."
    )]
    pub to: Option<String>,

    /// Include extended-hours data.
    #[arg(long)]
    pub extended_hours: bool,
}

/// Arguments for `market quote`.
#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  \
    schwab-agent market quote AAPL\n      \
    Get a compact quote row for one public ticker.\n\n  \
    schwab-agent market quote AAPL MSFT GOOG --fields sym,last,pct,vol\n      \
    Get selected compact fields for multiple public tickers.\n\n  \
    schwab-agent market quote AAPL --all-fields\n      \
    Return the full detailed quote object instead of compact rows.")]
pub struct QuoteArgs {
    /// Symbols to quote, for example AAPL MSFT $SPX.
    #[arg(required = true)]
    pub symbols: Vec<String>,

    /// Comma-separated output fields. Defaults to req,sym,bid,ask,last,mark,chg,pct,vol,err.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,

    /// Return the full detailed quote object instead of compact rows.
    #[arg(long, conflicts_with = "fields")]
    pub all_fields: bool,

    /// Schwab quote field groups to request from the API, for example quote,reference.
    #[arg(long)]
    pub api_fields: Option<String>,
}
