use clap::Args;

/// Arguments for the top-level `analyze` command.
#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  \
    schwab-agent analyze AAPL\n      \
    Analyze one symbol with a compact one-point TA summary plus quote data.\n\n  \
    schwab-agent analyze AAPL MSFT GOOG\n      \
    Analyze multiple public tickers and keep partial per-symbol errors in the JSON output.\n\n  \
    schwab-agent analyze AAPL --interval weekly --points 10\n      \
    Request a weekly dashboard depth similar to ta dashboard. The default is 1 point, while ta dashboard defaults to 20, because analyze is optimized for compact multi-symbol output. Quote fields may be live, while derived daily TA fields name their completed-candle price basis under analysis.derived.")]
pub struct AnalyzeArgs {
    /// One or more ticker symbols to analyze.
    #[arg(required = true)]
    pub symbols: Vec<String>,
    /// Candle interval for the dashboard.
    #[arg(long, default_value = "daily")]
    pub interval: String,
    /// Number of data points to return per indicator series.
    #[arg(long, default_value = "1")]
    pub points: usize,
}
