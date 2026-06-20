use clap::Args;

/// Arguments for analyze dashboard generation.
#[derive(Debug, Args)]
pub struct DashboardArgs {
    /// Ticker symbol, for example AAPL.
    #[arg(required = true)]
    pub symbol: String,
    /// Candle interval.
    #[arg(long, default_value = "daily")]
    pub interval: String,
    /// Number of data points to return per indicator series.
    #[arg(long, default_value = "20")]
    pub points: usize,
}

/// Arguments for analyze expected move generation.
#[derive(Debug, Args)]
pub struct ExpectedMoveArgs {
    /// Ticker symbol, for example AAPL.
    #[arg(required = true)]
    pub symbol: String,
    /// Target days to expiration for the option chain.
    #[arg(long, default_value = "30")]
    pub dte: u32,
}
