use clap::Args;

/// Arguments for the top-level `analyze` command.
#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  \
    schwab-agent analyze AAPL\n      \
    Analyze one symbol with quote data and TA.\n\n  \
    schwab-agent analyze AAPL MSFT GOOG\n      \
    Analyze multiple public tickers and keep partial per-symbol errors in the JSON output.\n\n  \
    schwab-agent analyze AAPL --expected-move --dte 45\n      \
    Include expected move from the option chain. Quote fields may be live, while derived daily TA fields name their completed-candle price basis under analysis.derived.")]
pub struct AnalyzeArgs {
    /// One or more ticker symbols to analyze.
    #[arg(required = true)]
    pub symbols: Vec<String>,
    /// Candle interval for the dashboard.
    #[arg(long, default_value = "daily")]
    pub interval: String,
    /// Number of data points to return per indicator series.
    #[arg(long, default_value = "20")]
    pub points: usize,
    /// Include option-chain expected move.
    #[arg(long)]
    pub expected_move: bool,
    /// Target days to expiration for expected move.
    #[arg(long, default_value = "30")]
    pub dte: u32,
}
