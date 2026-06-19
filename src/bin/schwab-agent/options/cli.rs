use clap::{ArgGroup, Args, Subcommand};

/// Option-chain, screening, and contract lookup commands.
#[derive(Debug, Subcommand)]
pub enum OptionCommand {
    /// Get expiration dates for an option symbol.
    Expirations(OptionExpirationsArgs),
    /// Get an option chain for a symbol.
    Chain(ChainArgs),
    /// Screen option chains with liquidity and pricing filters.
    Screen(ScreenArgs),
    /// Look up a single option contract.
    Contract(ContractArgs),
}

/// Arguments for `option expirations`.
#[derive(Debug, Args)]
pub struct OptionExpirationsArgs {
    /// Underlying symbol, for example AAPL.
    #[arg(required = true)]
    pub symbol: String,
}

/// Arguments for `option chain`.
#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  \
    schwab-agent option chain AAPL\n      \
    Get the full option chain with all contract types, expirations, and strikes.\n\n  \
    schwab-agent option chain AAPL --type call --dte 30 --fields strike,delta,bid,ask,volume,oi\n      \
    Get calls near 30 DTE with selected row fields.\n\n  \
    schwab-agent option chain AMD --type put --strike-min 140 --strike-max 160 --delta-min -0.30 --delta-max -0.15\n      \
    Get puts in a strike range with delta filters.\n\nValid --type values: call, put, all. Input is case-insensitive.")]
pub struct ChainArgs {
    /// Underlying symbol, for example AAPL.
    #[arg(required = true)]
    pub symbol: String,

    /// Contract type filter, call, put, or all.
    #[arg(long = "type")]
    pub contract_type: Option<String>,

    /// Nearest expiration by days to expiration.
    #[arg(long, conflicts_with = "expiration")]
    pub dte: Option<i32>,

    /// Exact expiration date in YYYY-MM-DD format.
    #[arg(long, conflicts_with = "dte")]
    pub expiration: Option<String>,

    /// Minimum delta filter.
    #[arg(long)]
    pub delta_min: Option<f64>,

    /// Maximum delta filter.
    #[arg(long)]
    pub delta_max: Option<f64>,

    /// Comma-separated field list.
    #[arg(long)]
    pub fields: Option<String>,

    /// Number of strikes around at-the-money to include.
    #[arg(long)]
    pub strike_count: Option<u32>,

    /// Exact strike price.
    #[arg(long, conflicts_with_all = ["strike_min", "strike_max", "strike_count"])]
    pub strike: Option<f64>,

    /// Minimum strike price.
    #[arg(long)]
    pub strike_min: Option<f64>,

    /// Maximum strike price.
    #[arg(long)]
    pub strike_max: Option<f64>,

    /// Schwab strike range filter.
    #[arg(long)]
    pub strike_range: Option<String>,
}

/// Arguments for `option screen`.
#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  \
    schwab-agent option screen AAPL --type call --dte-min 20 --dte-max 45 --min-volume 100 --min-oi 500 --max-spread-pct 10\n      \
    Screen liquid calls with tighter spreads across a DTE window.\n\n  \
    schwab-agent option screen SPY --type put --min-premium 1.00 --max-premium 5.00 --limit 20\n      \
    Screen puts by premium range and cap the result count.\n\nValid --type values: call, put, all. Numeric filters must be finite values.")]
pub struct ScreenArgs {
    /// Underlying symbol, for example AAPL.
    #[arg(required = true)]
    pub symbol: String,

    /// Contract type filter, call, put, or all.
    #[arg(long = "type")]
    pub contract_type: Option<String>,

    /// Minimum days to expiration.
    #[arg(long = "dte-min")]
    pub dte_min: Option<i32>,

    /// Maximum days to expiration.
    #[arg(long = "dte-max")]
    pub dte_max: Option<i32>,

    /// Exact expiration date in YYYY-MM-DD format.
    #[arg(long)]
    pub expiration: Option<String>,

    /// Minimum delta filter.
    #[arg(long)]
    pub delta_min: Option<f64>,

    /// Maximum delta filter.
    #[arg(long)]
    pub delta_max: Option<f64>,

    /// Comma-separated field list.
    #[arg(long)]
    pub fields: Option<String>,

    /// Number of strikes around at-the-money to include.
    #[arg(long)]
    pub strike_count: Option<u32>,

    /// Exact strike price.
    #[arg(long, conflicts_with_all = ["strike_min", "strike_max", "strike_count"])]
    pub strike: Option<f64>,

    /// Minimum strike price.
    #[arg(long)]
    pub strike_min: Option<f64>,

    /// Maximum strike price.
    #[arg(long)]
    pub strike_max: Option<f64>,

    /// Schwab strike range filter.
    #[arg(long)]
    pub strike_range: Option<String>,

    /// Minimum bid price.
    #[arg(long = "min-bid")]
    pub min_bid: Option<f64>,

    /// Maximum ask price.
    #[arg(long = "max-ask")]
    pub max_ask: Option<f64>,

    /// Minimum volume.
    #[arg(long = "min-volume")]
    pub min_volume: Option<u64>,

    /// Minimum open interest.
    #[arg(long = "min-oi")]
    pub min_oi: Option<u64>,

    /// Maximum spread percent.
    #[arg(long = "max-spread-pct")]
    pub max_spread_pct: Option<f64>,

    /// Minimum premium.
    #[arg(long = "min-premium")]
    pub min_premium: Option<f64>,

    /// Maximum premium.
    #[arg(long = "max-premium")]
    pub max_premium: Option<f64>,

    /// Sort field.
    #[arg(long)]
    pub sort: Option<String>,

    /// Maximum number of results.
    #[arg(long)]
    pub limit: Option<usize>,
}

/// Arguments for `option contract`.
#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("contract-side")
        .required(true)
        .args(["call", "put"])
))]
pub struct ContractArgs {
    /// Underlying symbol, for example AAPL.
    #[arg(required = true)]
    pub symbol: String,

    /// Exact expiration date in YYYY-MM-DD format.
    #[arg(long)]
    pub expiration: String,

    /// Exact strike price.
    #[arg(long)]
    pub strike: f64,

    /// Select a call contract.
    #[arg(long, conflicts_with = "put")]
    pub call: bool,

    /// Select a put contract.
    #[arg(long, conflicts_with = "call")]
    pub put: bool,
}
