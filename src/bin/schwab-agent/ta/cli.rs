use clap::{Args, Subcommand};

/// Technical analysis commands.
#[derive(Debug, Subcommand)]
pub enum TaCommand {
    /// Run all indicators for a symbol and return a category-grouped dashboard.
    Dashboard(DashboardArgs),
    /// Compute expected move from the option chain's ATM straddle.
    #[command(name = "expected-move")]
    ExpectedMove(ExpectedMoveArgs),
}

/// Arguments for `ta dashboard`.
#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  \
    schwab-agent ta dashboard AAPL\n      \
    Run a daily dashboard with the default 20 points per indicator series.\n\n  \
    schwab-agent ta dashboard SPY --interval weekly --points 10\n      \
    Run a weekly dashboard and cap each indicator series at 10 points.")]
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

/// Arguments for `ta expected-move`.
#[derive(Debug, Args)]
pub struct ExpectedMoveArgs {
    /// Ticker symbol, for example AAPL.
    #[arg(required = true)]
    pub symbol: String,
    /// Target days to expiration for the option chain.
    #[arg(long, default_value = "30")]
    pub dte: u32,
}
