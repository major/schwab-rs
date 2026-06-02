//! Command-line argument definitions for the `schwab-agent` JSON CLI.

use clap::{ArgGroup, Args, Parser, Subcommand};
use clap_complete::Shell;

/// Agent-oriented JSON CLI porcelain for Charles Schwab workflows.
#[derive(Debug, Parser)]
#[command(
    name = "schwab-agent",
    version,
    about = "Agent-oriented JSON CLI porcelain for Charles Schwab workflows",
    long_about = "All normal command output is compact JSON. Use --help on any command for examples and flags. Trading commands intentionally start with draft and validate workflows before placement.\n\nSetup and agent discovery:\n  schwab-agent schema\n      Print machine-readable command capabilities, safety classifications, output formats, environment variables, exit codes, and field selectors.\n  schwab-agent doctor\n      Print sanitized config, auth, token, and debug health without reading account data.\n  schwab-agent config show\n      Produces the same sanitized output as config status.\n  schwab-agent config status\n      Print sanitized config, token, credential-source, precedence, and debug status without exposing secrets.\n  schwab-agent completions bash > schwab-agent.bash\n      Generate a raw bash completion script for installation or sourcing.\n  schwab-agent completion zsh > _schwab-agent\n      Singular alias for completion generation, useful for zsh function installs.\n\nEnvironment variables:\n  SCHWAB_CLIENT_ID, SCHWAB_CLIENT_SECRET, SCHWAB_CALLBACK_URL\n      Auth credentials. Environment values override ~/.config/schwab-agent/config.json.\n  SCHWAB_TOKEN_PATH\n      Optional token path override. Empty values are ignored.\n  XDG_CONFIG_HOME\n      Base directory for config.json and the default compatibility token path.\n  XDG_STATE_HOME\n      Base directory for saved order previews, falling back to the platform state or local data directory.\n  RUST_LOG\n      Enable tracing diagnostics on stderr, for example RUST_LOG=schwab=debug. JSON stdout remains unchanged.\n  SCHWAB_AGENT_JSON_ERRORS\n      Set to 1 to render clap usage errors as JSON on stdout with code, message, category, retryable, and hint fields. Default clap stderr remains unchanged when unset.\n\nPrecedence: command flags > environment variables > config file > defaults. Defaults include https://127.0.0.1:8182 for the callback URL and $XDG_CONFIG_HOME/schwab-agent-rs/token.json for tokens when SCHWAB_TOKEN_PATH is unset.",
    arg_required_else_help = true,
    propagate_version = true,
    help_template = "{name} {version}\n{about-section}\n{usage-heading} {usage}\n\n{all-args}{tab}"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    /// Returns the stable dotted command name used in JSON envelopes.
    #[must_use]
    pub fn command_name(&self) -> &'static str {
        match &self.command {
            Command::Analyze(_) => "analyze",
            Command::Auth(AuthCommand::Status) => "auth.status",
            Command::Auth(AuthCommand::Login(_)) => "auth.login",
            Command::Auth(AuthCommand::LoginUrl(_)) => "auth.login_url",
            Command::Auth(AuthCommand::Exchange(_)) => "auth.exchange",
            Command::Auth(AuthCommand::Refresh) => "auth.refresh",
            Command::Config(ConfigCommand::Show) => "config.show",
            Command::Config(ConfigCommand::Status) => "config.status",
            Command::Doctor => "doctor",
            Command::Option(OptionCommand::Expirations(_)) => "option.expirations",
            Command::Option(OptionCommand::Chain(_)) => "option.chain",
            Command::Option(OptionCommand::Screen(_)) => "option.screen",
            Command::Option(OptionCommand::Contract(_)) => "option.contract",
            Command::Market(MarketCommand::History(_)) => "market.history",
            Command::Market(MarketCommand::Quote(_)) => "market.quote",
            Command::History(_) => "market.history",
            Command::Quote(_) => "market.quote",
            Command::Order(OrderCommand::Get(_)) => "order.get",
            Command::Order(_) => "order",
            Command::Orders(_) => "order.get",
            Command::Positions(_) => "account",
            Command::Schema => "schema",
            Command::Stock(_) => "stock",
            Command::Completions(_) => "completions",
            Command::Ta(TaCommand::Dashboard(_)) => "ta.dashboard",
            Command::Ta(TaCommand::ExpectedMove(_)) => "ta.expected-move",
            Command::Account(_) => "account",
        }
    }
}

/// Top-level command groups.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Multi-symbol analysis combining quote and technical analysis dashboard.
    Analyze(AnalyzeArgs),
    /// Authentication commands for token setup and inspection.
    #[command(subcommand)]
    Auth(AuthCommand),
    /// Configuration, environment, and debug discovery commands.
    #[command(subcommand)]
    Config(ConfigCommand),
    /// Inspect sanitized auth, config, and environment health.
    Doctor,
    /// Market-data workflows with compact JSON summaries.
    #[command(subcommand)]
    Market(MarketCommand),
    /// Alias for `market quote`.
    Quote(QuoteArgs),
    /// Alias for `market history`.
    History(HistoryArgs),
    /// Option chain, screening, and contract lookup workflows.
    #[command(subcommand)]
    Option(OptionCommand),
    /// Unified order construction, preview, placement, and lifecycle workflows.
    #[command(subcommand)]
    Order(OrderCommand),
    /// Alias for `order get`.
    Orders(crate::order::lifecycle::OrderGetArgs),
    /// Alias for `account --positions`.
    Positions(PositionsArgs),
    /// Legacy stock command namespace with migration hints.
    #[command(hide = true, subcommand)]
    Stock(StockCommand),
    /// Generate shell completion scripts.
    #[command(alias = "completion")]
    Completions(CompletionsArgs),
    /// Technical analysis indicator workflows.
    #[command(subcommand)]
    Ta(TaCommand),
    /// Account discovery, balances, positions, and resolution workflows.
    ///
    /// Without a selector, returns account hashes, nicknames, balance summaries
    /// (margin or cash), day-trader and closing-only flags, and optionally open
    /// positions. With a selector alone, resolves an account hash or nickname to
    /// its canonical account hash. With a selector plus `--positions`, returns
    /// the matching account summary.
    #[command(after_help = "LLM workflow:\n  \
        schwab-agent account\n      \
        List account hashes, nicknames, account types, balance summaries, and account flags for every linked account.\n\n  \
        schwab-agent account --positions\n      \
        Include holdings as compact position objects for every linked account. Use this before allocation or order-planning decisions.\n\n  \
        schwab-agent account HASH_OR_NICKNAME\n      \
        Resolve a raw account hash or unique nickname to the canonical hash for --account on stock and order commands.\n\n  \
        schwab-agent account HASH_OR_NICKNAME --positions\n      \
        Return the selected account summary plus compact position objects instead of only resolving the hash.\n\n\
        Position objects include symbol, description, asset_type, long_quantity, short_quantity, average_price, market_value, current_day_profit_loss, and current_day_profit_loss_percentage when Schwab provides them.")]
    Account(AccountArgs),
    /// Emit machine-readable CLI capabilities for agents.
    Schema,
}

/// Configuration and environment discovery commands.
#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Show sanitized config, auth, path, precedence, and debug status.
    Show,
    /// Show sanitized config, auth, path, precedence, and debug status.
    Status,
}

/// Arguments for shell completion generation.
#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for.
    #[arg(value_enum)]
    pub shell: Shell,
}

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

/// Arguments for the top-level `analyze` command.
#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  \
    schwab-agent analyze AAPL\n      \
    Analyze one symbol with a compact one-point TA summary plus quote data.\n\n  \
    schwab-agent analyze AAPL MSFT GOOG\n      \
    Analyze multiple public tickers and keep partial per-symbol errors in the JSON output.\n\n  \
    schwab-agent analyze AAPL --interval weekly --points 10\n      \
    Request a weekly dashboard depth similar to ta dashboard. The default is 1 point, while ta dashboard defaults to 20, because analyze is optimized for compact multi-symbol output.")]
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

/// Authentication commands.
#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Show local token state without printing secrets.
    Status,
    /// Full interactive login: open browser, wait for a complete callback, exchange and save token.
    Login(LoginArgs),
    /// Build a browser authorization URL and open it in the default browser.
    LoginUrl(LoginUrlArgs),
    /// Exchange a pasted browser redirect URL for a saved token file.
    Exchange(AuthExchangeArgs),
    /// Force-refresh the saved token file.
    Refresh,
}

/// Arguments for `auth login`.
#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Skip opening the authorization URL in the default browser.
    #[arg(long)]
    pub no_browser: bool,

    /// Seconds to wait for the callback before timing out.
    #[arg(long, default_value = "300")]
    pub timeout: u64,
}

/// Arguments for `auth login-url`.
#[derive(Debug, Args)]
pub struct LoginUrlArgs {
    /// Skip opening the authorization URL in the default browser.
    #[arg(long)]
    pub no_browser: bool,
}

/// Arguments for `auth exchange`.
#[derive(Debug, Args)]
pub struct AuthExchangeArgs {
    /// CSRF state returned by `auth login-url`.
    #[arg(long)]
    pub state: String,

    /// Full redirect URL copied from the browser address bar.
    #[arg(long)]
    pub redirect_url: String,
}

/// Market-data commands.
#[derive(Debug, Subcommand)]
pub enum MarketCommand {
    /// Get price history candles for a symbol.
    History(HistoryArgs),
    /// Get compact quote summaries for one or more symbols.
    Quote(QuoteArgs),
}

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

/// Arguments for `account`.
#[derive(Debug, Args)]
pub struct AccountArgs {
    /// Account hash or nickname to resolve. Omit to list account summaries.
    pub selector: Option<String>,

    /// Include individual positions in each account summary.
    #[arg(long)]
    pub positions: bool,
}

/// Arguments for the top-level `positions` alias.
#[derive(Debug, Args)]
pub struct PositionsArgs {
    /// Account hash or nickname to inspect. Omit to list positions for all accounts.
    pub selector: Option<String>,
}

impl From<&PositionsArgs> for AccountArgs {
    fn from(args: &PositionsArgs) -> Self {
        Self {
            selector: args.selector.clone(),
            positions: true,
        }
    }
}

/// Legacy stock commands that now point to `order equity`.
#[derive(Debug, Subcommand)]
pub enum StockCommand {
    /// Use `order equity buy` instead.
    Buy(EquityOrderArgs),
    /// Use `order equity sell` instead.
    Sell(EquityOrderArgs),
}

impl AccountArgs {
    /// Whether position data should be fetched from the API.
    ///
    /// Returns `true` when position data is explicitly requested.
    #[must_use]
    pub fn include_positions(&self) -> bool {
        self.positions
    }

    /// Whether the selector should return an account summary instead of a hash resolution.
    #[must_use]
    pub fn requests_summary(&self) -> bool {
        self.include_positions()
    }
}

// ---------------------------------------------------------------------------
// Unified order command types
// ---------------------------------------------------------------------------

/// Unified order command covering equity and option workflows.
///
/// Use `order equity` for stock trades and `order option` for option trades.
/// The `get`, `cancel`, `replace`, `repeat`, `place-from-preview`, `preview-raw`, and
/// `place-raw` subcommands work with orders of any type.
#[derive(Debug, Subcommand)]
pub enum OrderCommand {
    /// Equity (stock) order: buy, sell, sell-short, buy-to-cover.
    #[command(subcommand)]
    Equity(EquityArgs),
    /// Option order: buy-to-open, sell-to-open, buy-to-close, sell-to-close (OCC symbol required).
    #[command(subcommand)]
    Option(OptionArgs),
    /// Get active orders, symbol-filtered orders, or one exact order.
    #[command(
        after_help = "LLM selection guide:\n  schwab-agent order get\n      Get all active/open orders across every linked account. Use this first when you need current open orders and do not already know the account.\n\n  schwab-agent order get --account HASH_OR_NICKNAME\n      Get all active/open orders for one account. Use this when you already know which account to inspect.\n\n  schwab-agent order get --symbol IBM\n      Get active/open orders whose orderLegCollection includes an IBM instrument symbol. Matching is case-insensitive and includes multi-leg orders when any leg matches. Add --account to limit this to one account.\n\n  schwab-agent order get --include-inactive\n      Get active plus inactive orders across every linked account. Any returned status not listed in the active_statuses output field is treated as inactive. Add --account to limit this to one account.\n\n  schwab-agent order get --account HASH_OR_NICKNAME --order-id ORDER_ID\n      Get one exact order. Use this only when both the account and order ID are known. Do not combine --order-id with discovery filters such as --recent, --from, --to, --symbol, or --include-inactive.\n\nActive statuses are exact strings returned in the active_statuses output field."
    )]
    Get(crate::order::lifecycle::OrderGetArgs),
    /// Cancel an order by ID.
    ///
    /// After cancellation, verifies the status via a follow-up GET so
    /// the agent can confirm the order was actually canceled.
    Cancel(crate::order::lifecycle::OrderCancelArgs),
    /// Replace an existing order.
    Replace(ReplaceArgs),
    /// Repeat an existing order by rebuilding it as a new order payload.
    Repeat(crate::order::lifecycle::OrderRepeatArgs),
    /// Place an order from a previously saved preview digest.
    #[command(name = "place-from-preview")]
    PlaceFromPreview(PlaceFromPreviewArgs),
    /// Preview an arbitrary JSON order payload (for brackets, OCO, etc.).
    #[command(name = "preview-raw")]
    PreviewRaw(PreviewRawArgs),
    /// Place an arbitrary JSON order payload (for brackets, OCO, etc.).
    #[command(name = "place-raw")]
    PlaceRaw(PlaceRawArgs),
}

/// Equity (stock) order actions.
#[derive(Debug, Subcommand)]
pub enum EquityArgs {
    /// Buy shares.
    Buy(EquityOrderArgs),
    /// Sell shares.
    Sell(EquityOrderArgs),
    /// Sell shares short.
    #[command(name = "sell-short")]
    SellShort(EquityOrderArgs),
    /// Buy to cover a short position.
    #[command(name = "buy-to-cover")]
    BuyToCover(EquityOrderArgs),
}

/// Arguments for an equity order action.
#[derive(Debug, Args)]
#[command(after_help = "Execution modes:\n  \
    schwab-agent order equity buy AAPL -q 1 --price 100 --dry-run\n      \
    Print local draft JSON only. No account, auth, preview API, or placement is required. --preview is an alias for this local draft mode.\n\n  \
    schwab-agent order equity buy AAPL -q 1 --price 100\n      \
    Compatibility draft mode. Omitting --account still prints local order JSON without any API call.\n\n  \
    schwab-agent order equity buy AAPL -q 1 --price 100 --account HASH --save-preview\n      \
    Call Schwab previewOrder, save a tamper-evident digest, and do not place.\n\n  \
    schwab-agent order place-from-preview --account HASH --digest DIGEST_HEX\n      \
    Place the exact saved preview after review.\n\n  \
    schwab-agent order equity buy AAPL -q 1 --price 100 --account HASH\n      \
    Place immediately. Mutable order config must be enabled.")]
pub struct EquityOrderArgs {
    /// Ticker symbol (e.g. AAPL, SPY).
    pub symbol: String,
    /// Number of shares.
    #[arg(short, long)]
    pub quantity: f64,
    /// Limit price (omit for market order; with --stop becomes stop-limit).
    #[arg(short, long)]
    pub price: Option<f64>,
    /// Stop trigger price (omit for market/limit; with --price becomes stop-limit).
    #[arg(long)]
    pub stop: Option<f64>,
    #[command(flatten)]
    pub common: CommonOrderArgs,
}

/// Option order actions using OCC symbols.
#[derive(Debug, Subcommand)]
pub enum OptionArgs {
    /// Buy to open a new option position.
    #[command(name = "buy-to-open")]
    BuyToOpen(OptionOrderArgs),
    /// Sell to open a new option position.
    #[command(name = "sell-to-open")]
    SellToOpen(OptionOrderArgs),
    /// Buy to close an existing option position.
    #[command(name = "buy-to-close")]
    BuyToClose(OptionOrderArgs),
    /// Sell to close an existing option position.
    #[command(name = "sell-to-close")]
    SellToClose(OptionOrderArgs),
}

/// Arguments for an option order action.
#[derive(Debug, Args)]
#[command(after_help = "Execution modes:\n  \
    schwab-agent order option buy-to-open \"AAPL  250117C00150000\" -q 1 --price 5.00 --dry-run\n      \
    Print local draft JSON only. No account, auth, preview API, or placement is required. --preview is an alias for this local draft mode.\n\n  \
    schwab-agent order option buy-to-open \"AAPL  250117C00150000\" -q 1 --price 5.00\n      \
    Compatibility draft mode. Omitting --account still prints local order JSON without any API call.\n\n  \
    schwab-agent order option buy-to-open \"AAPL  250117C00150000\" -q 1 --price 5.00 --account HASH --save-preview\n      \
    Call Schwab previewOrder, save a tamper-evident digest, and do not place.\n\n  \
    schwab-agent order place-from-preview --account HASH --digest DIGEST_HEX\n      \
    Place the exact saved preview after review.\n\n  \
    schwab-agent order option buy-to-open \"AAPL  250117C00150000\" -q 1 --price 5.00 --account HASH\n      \
    Place immediately. Mutable order config must be enabled.")]
pub struct OptionOrderArgs {
    /// OCC option symbol (e.g. AAPL  250117C00150000).
    pub symbol: String,
    /// Number of contracts.
    #[arg(short, long)]
    pub quantity: f64,
    /// Limit price (omit for market order).
    #[arg(short, long)]
    pub price: Option<f64>,
    #[command(flatten)]
    pub common: CommonOrderArgs,
}

/// Arguments shared by equity and option order actions.
#[derive(Debug, Args)]
pub struct CommonOrderArgs {
    /// Account hash or nickname. Omit for local draft mode.
    #[arg(short, long)]
    pub account: Option<String>,
    /// Trading session.
    #[arg(long, default_value = "normal")]
    pub session: crate::shared::SessionChoice,
    /// Order duration.
    #[arg(long, default_value = "day")]
    pub duration: crate::shared::DurationChoice,
    /// Print local draft JSON without auth, preview, or placement.
    #[arg(long, conflicts_with_all = ["preview", "save_preview", "preview_first"])]
    pub dry_run: bool,
    /// Alias for --dry-run. This is local payload preview, not Schwab previewOrder.
    #[arg(long, conflicts_with_all = ["dry_run", "save_preview", "preview_first"])]
    pub preview: bool,
    /// Call Schwab previewOrder and save a digest to disk (requires --account).
    #[arg(long, requires = "account", conflicts_with_all = ["dry_run", "preview"])]
    pub save_preview: bool,
    /// Call Schwab previewOrder first, then place automatically (requires --account).
    #[arg(long, requires = "account", conflicts_with_all = ["dry_run", "preview"])]
    pub preview_first: bool,
}

/// Arguments for `order replace`.
#[derive(Debug, Args)]
pub struct ReplaceArgs {
    /// Account hash or nickname.
    #[arg(short, long)]
    pub account: String,
    /// Order ID to replace.
    #[arg(long, value_parser = clap::value_parser!(i64).range(1..))]
    pub order_id: i64,
    /// The replacement order.
    #[command(subcommand)]
    pub order_spec: ReplaceOrderSpec,
}

/// Replacement order payload (equity or option).
#[derive(Debug, Subcommand)]
pub enum ReplaceOrderSpec {
    /// Replace with an equity order.
    #[command(subcommand)]
    Equity(EquityArgs),
    /// Replace with an option order.
    #[command(subcommand)]
    Option(OptionArgs),
}

/// Arguments for `order place-from-preview`.
#[derive(Debug, Args)]
pub struct PlaceFromPreviewArgs {
    /// Account hash or nickname.
    #[arg(short, long)]
    pub account: String,
    /// SHA-256 digest from a previous preview --save-preview run.
    #[arg(short, long)]
    pub digest: String,
}

/// Arguments for `order preview-raw`.
#[derive(Debug, Args)]
pub struct PreviewRawArgs {
    /// Account hash or nickname.
    #[arg(long)]
    pub account: String,
    /// Save preview to disk after previewing.
    #[arg(long)]
    pub save_preview: bool,
    /// Complete order JSON payload (for brackets, OCO, triggers, etc.).
    #[arg(long)]
    pub json: String,
}

/// Arguments for `order place-raw`.
#[derive(Debug, Args)]
pub struct PlaceRawArgs {
    /// Account hash or nickname.
    #[arg(long)]
    pub account: String,
    /// Complete order JSON payload.
    #[arg(long)]
    pub json: String,
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use clap::{CommandFactory, Parser, error::ErrorKind};

    use super::{Cli, Command, MarketCommand, OrderCommand, TaCommand};

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn expect_history_alias(command: Command) -> super::HistoryArgs {
        match command {
            Command::History(args) => args,
            _ => panic!("expected history alias command"),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn expect_quote_alias(command: Command) -> super::QuoteArgs {
        match command {
            Command::Quote(args) => args,
            _ => panic!("expected quote alias command"),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn expect_positions_alias(command: &Command) -> &super::PositionsArgs {
        match command {
            Command::Positions(args) => args,
            _ => panic!("expected positions alias command"),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn expect_orders_alias(command: Command) -> crate::order::lifecycle::OrderGetArgs {
        match command {
            Command::Orders(args) => args,
            _ => panic!("expected orders alias command"),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn expect_equity_buy_duration(command: Command) -> crate::shared::DurationChoice {
        match command {
            Command::Order(OrderCommand::Equity(super::EquityArgs::Buy(
                super::EquityOrderArgs { common, .. },
            ))) => common.duration,
            _ => panic!("expected order equity buy command"),
        }
    }

    #[test]
    fn command_tree_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn order_get_help_renders_llm_guide_once() {
        let mut command = Cli::command();
        let help = command
            .find_subcommand_mut("order")
            .and_then(|order| order.find_subcommand_mut("get"))
            .map(|get| get.render_long_help().to_string())
            .expect("order get command exists");

        assert_eq!(help.matches("LLM selection guide:").count(), 1);
        assert!(help.contains("active_statuses output field"));
        assert!(help.contains("discovery filters"));
        assert!(help.contains("--symbol IBM"));
        assert!(help.contains("Matching is case-insensitive"));
    }

    #[test]
    fn account_help_includes_llm_workflow() {
        let mut command = Cli::command();
        let help = command
            .find_subcommand_mut("account")
            .map(|account| account.render_long_help().to_string())
            .expect("account command exists");

        assert_eq!(help.matches("LLM workflow:").count(), 1);
        assert!(!help.contains("Examples:"));
        assert!(!help.contains("--with-positions-only"));
        assert!(help.contains("schwab-agent account --positions"));
        assert!(help.contains("compact position objects"));
        assert!(help.contains("--account"));
    }

    #[test]
    fn command_name_auth_status() {
        let cli = Cli::parse_from(["schwab-agent", "auth", "status"]);
        assert_eq!(cli.command_name(), "auth.status");
    }

    #[test]
    fn command_name_auth_login() {
        let cli = Cli::parse_from(["schwab-agent", "auth", "login"]);
        assert_eq!(cli.command_name(), "auth.login");
    }

    #[test]
    fn command_name_analyze() {
        let cli = Cli::parse_from(["schwab-agent", "analyze", "AAPL"]);
        assert_eq!(cli.command_name(), "analyze");
    }

    #[test]
    fn command_name_auth_login_url() {
        let cli = Cli::parse_from(["schwab-agent", "auth", "login-url"]);
        assert_eq!(cli.command_name(), "auth.login_url");
    }

    #[test]
    fn command_name_auth_exchange() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "auth",
            "exchange",
            "--state",
            "abc",
            "--redirect-url",
            "https://example.com",
        ]);
        assert_eq!(cli.command_name(), "auth.exchange");
    }

    #[test]
    fn command_name_auth_refresh() {
        let cli = Cli::parse_from(["schwab-agent", "auth", "refresh"]);
        assert_eq!(cli.command_name(), "auth.refresh");
    }

    #[test]
    fn command_name_config_status() {
        let cli = Cli::parse_from(["schwab-agent", "config", "status"]);
        assert_eq!(cli.command_name(), "config.status");
    }

    #[test]
    fn command_name_config_show() {
        let cli = Cli::parse_from(["schwab-agent", "config", "show"]);
        assert_eq!(cli.command_name(), "config.show");
    }

    #[test]
    fn command_name_doctor() {
        let cli = Cli::parse_from(["schwab-agent", "doctor"]);
        assert_eq!(cli.command_name(), "doctor");
    }

    #[test]
    fn command_name_schema() {
        let cli = Cli::parse_from(["schwab-agent", "schema"]);
        assert_eq!(cli.command_name(), "schema");
    }

    #[test]
    fn command_name_market_history() {
        let cli = Cli::parse_from(["schwab-agent", "market", "history", "AAPL"]);
        assert_eq!(cli.command_name(), "market.history");
    }

    #[test]
    fn command_name_history_alias() {
        let cli = Cli::parse_from(["schwab-agent", "history", "AAPL"]);
        assert_eq!(cli.command_name(), "market.history");
    }

    #[test]
    fn command_name_market_history_with_all_flags() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "market",
            "history",
            "AAPL",
            "--fields",
            "ts,close,vol",
            "--period-type",
            "month",
            "--period",
            "3",
            "--frequency-type",
            "daily",
            "--frequency",
            "1",
            "--from",
            "1735689600000",
            "--to",
            "1743379200000",
            "--extended-hours",
        ]);
        assert_eq!(cli.command_name(), "market.history");
    }

    #[test]
    fn market_history_fields_parse_output_fields() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "market",
            "history",
            "AAPL",
            "--fields",
            "ts,close,vol",
        ]);

        let Command::Market(MarketCommand::History(args)) = cli.command else {
            panic!("expected market history command");
        };
        assert_eq!(args.fields.as_deref(), Some("ts,close,vol"));
        assert!(!args.all_fields);
    }

    #[test]
    fn history_alias_parses_market_history_args() {
        let cli = Cli::parse_from(["schwab-agent", "history", "SPY", "--fields", "ts,close"]);

        let args = expect_history_alias(cli.command);
        assert_eq!(args.symbol, "SPY");
        assert_eq!(args.fields.as_deref(), Some("ts,close"));
    }

    #[test]
    fn market_history_all_fields_parses() {
        let cli = Cli::parse_from(["schwab-agent", "market", "history", "AAPL", "--all-fields"]);

        let Command::Market(MarketCommand::History(args)) = cli.command else {
            panic!("expected market history command");
        };
        assert!(args.all_fields);
        assert!(args.fields.is_none());
    }

    #[test]
    fn command_name_market_quote() {
        let cli = Cli::parse_from(["schwab-agent", "market", "quote", "AAPL"]);
        assert_eq!(cli.command_name(), "market.quote");
    }

    #[test]
    fn command_name_quote_alias() {
        let cli = Cli::parse_from(["schwab-agent", "quote", "AAPL"]);
        assert_eq!(cli.command_name(), "market.quote");
    }

    #[test]
    fn market_quote_fields_parse_output_and_api_fields() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "market",
            "quote",
            "AAPL",
            "--fields",
            "sym,last",
            "--api-fields",
            "quote,reference",
        ]);

        let Command::Market(MarketCommand::Quote(args)) = cli.command else {
            panic!("expected market quote command");
        };
        assert_eq!(args.fields.as_deref(), Some("sym,last"));
        assert_eq!(args.api_fields.as_deref(), Some("quote,reference"));
        assert!(!args.all_fields);
    }

    #[test]
    fn quote_alias_parses_market_quote_args() {
        let cli = Cli::parse_from(["schwab-agent", "quote", "AAPL", "--fields", "sym,last"]);

        let args = expect_quote_alias(cli.command);
        assert_eq!(args.symbols, ["AAPL"]);
        assert_eq!(args.fields.as_deref(), Some("sym,last"));
    }

    #[test]
    fn market_quote_all_fields_parses() {
        let cli = Cli::parse_from(["schwab-agent", "market", "quote", "AAPL", "--all-fields"]);

        let Command::Market(MarketCommand::Quote(args)) = cli.command else {
            panic!("expected market quote command");
        };
        assert!(args.all_fields);
        assert!(args.fields.is_none());
    }

    #[test]
    fn command_name_option_expirations() {
        let cli = Cli::parse_from(["schwab-agent", "option", "expirations", "AAPL"]);
        assert_eq!(cli.command_name(), "option.expirations");
    }

    #[test]
    fn command_name_option_chain() {
        let cli = Cli::parse_from(["schwab-agent", "option", "chain", "AAPL"]);
        assert_eq!(cli.command_name(), "option.chain");
    }

    #[test]
    fn command_name_option_screen() {
        let cli = Cli::parse_from(["schwab-agent", "option", "screen", "AAPL"]);
        assert_eq!(cli.command_name(), "option.screen");
    }

    #[test]
    fn command_name_option_contract() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "option",
            "contract",
            "AAPL",
            "--expiration",
            "2026-01-17",
            "--strike",
            "200",
            "--call",
        ]);
        assert_eq!(cli.command_name(), "option.contract");
    }

    #[test]
    fn command_name_account() {
        let cli = Cli::parse_from(["schwab-agent", "account"]);
        assert_eq!(cli.command_name(), "account");
    }

    #[test]
    fn command_name_account_with_positions() {
        let cli = Cli::parse_from(["schwab-agent", "account", "--positions"]);
        assert_eq!(cli.command_name(), "account");
    }

    #[test]
    fn command_name_positions_alias() {
        let cli = Cli::parse_from(["schwab-agent", "positions"]);
        assert_eq!(cli.command_name(), "account");
    }

    #[test]
    fn command_name_account_with_selector() {
        let cli = Cli::parse_from(["schwab-agent", "account", "Trading"]);
        assert_eq!(cli.command_name(), "account");
    }

    #[test]
    fn command_name_ta_dashboard() {
        let cli = Cli::parse_from(["schwab-agent", "ta", "dashboard", "AAPL"]);
        assert_eq!(cli.command_name(), "ta.dashboard");
    }

    #[test]
    fn command_name_ta_expected_move() {
        let cli = Cli::parse_from(["schwab-agent", "ta", "expected-move", "AAPL"]);
        assert_eq!(cli.command_name(), "ta.expected-move");
    }

    #[test]
    fn command_name_completions() {
        let cli = Cli::parse_from(["schwab-agent", "completions", "bash"]);
        assert_eq!(cli.command_name(), "completions");
    }

    #[test]
    fn command_name_completion_alias() {
        let cli = Cli::parse_from(["schwab-agent", "completion", "zsh"]);
        assert_eq!(cli.command_name(), "completions");
    }

    #[test]
    fn parse_account_no_flags() {
        let cli = Cli::parse_from(["schwab-agent", "account"]);

        let Command::Account(args) = cli.command else {
            panic!("expected account command");
        };
        assert!(args.selector.is_none());
        assert!(!args.positions);
    }

    #[test]
    fn parse_account_positions() {
        let cli = Cli::parse_from(["schwab-agent", "account", "--positions"]);

        let Command::Account(args) = cli.command else {
            panic!("expected account command");
        };
        assert!(args.selector.is_none());
        assert!(args.positions);
        assert!(args.include_positions());
    }

    #[test]
    fn parse_account_with_positions_only_is_rejected() {
        let err = Cli::try_parse_from(["schwab-agent", "account", "--with-positions-only"])
            .expect_err("removed account flag should be rejected");

        assert!(err.to_string().contains("--with-positions-only"));
    }

    #[test]
    fn parse_account_fields_is_rejected() {
        let err =
            Cli::try_parse_from(["schwab-agent", "account", "--positions", "--fields", "sym"])
                .expect_err("removed account flag should be rejected");

        assert!(err.to_string().contains("--fields"));
    }

    #[test]
    fn parse_account_all_fields_is_rejected() {
        let err = Cli::try_parse_from(["schwab-agent", "account", "--positions", "--all-fields"])
            .expect_err("removed account flag should be rejected");

        assert!(err.to_string().contains("--all-fields"));
    }

    #[test]
    fn parse_account_no_flags_include_positions_false() {
        let cli = Cli::parse_from(["schwab-agent", "account"]);

        let Command::Account(args) = cli.command else {
            panic!("expected account command");
        };
        assert!(!args.include_positions());
    }

    #[test]
    fn parse_account_selector() {
        let cli = Cli::parse_from(["schwab-agent", "account", "Trading"]);

        let Command::Account(args) = cli.command else {
            panic!("expected account command");
        };
        assert_eq!(args.selector.as_deref(), Some("Trading"));
    }

    #[test]
    fn parse_account_selector_with_positions() {
        let cli = Cli::parse_from(["schwab-agent", "account", "--positions", "Trading"]);

        let Command::Account(args) = cli.command else {
            panic!("expected account command");
        };
        assert_eq!(args.selector.as_deref(), Some("Trading"));
        assert!(args.positions);
        assert!(args.include_positions());
        assert!(args.requests_summary());
    }

    #[test]
    fn positions_alias_requests_positions_for_all_accounts() {
        let cli = Cli::parse_from(["schwab-agent", "positions"]);

        let args = expect_positions_alias(&cli.command);
        let account_args = super::AccountArgs::from(args);
        assert!(account_args.selector.is_none());
        assert!(account_args.positions);
        assert!(account_args.include_positions());
    }

    #[test]
    fn positions_alias_accepts_selector() {
        let cli = Cli::parse_from(["schwab-agent", "positions", "Trading"]);

        let args = expect_positions_alias(&cli.command);
        let account_args = super::AccountArgs::from(args);
        assert_eq!(account_args.selector.as_deref(), Some("Trading"));
        assert!(account_args.positions);
        assert!(account_args.requests_summary());
    }

    #[test]
    fn parse_account_selector_before_positions() {
        let cli = Cli::parse_from(["schwab-agent", "account", "Trading", "--positions"]);

        let Command::Account(args) = cli.command else {
            panic!("expected account command");
        };
        assert_eq!(args.selector.as_deref(), Some("Trading"));
        assert!(args.positions);
        assert!(args.requests_summary());
    }

    #[test]
    fn parse_ta_dashboard_defaults() {
        let cli = Cli::parse_from(["schwab-agent", "ta", "dashboard", "AAPL"]);

        let Command::Ta(TaCommand::Dashboard(args)) = cli.command else {
            panic!("expected ta dashboard command");
        };
        assert_eq!(args.symbol, "AAPL");
        assert_eq!(args.interval, "daily");
        assert_eq!(args.points, 20);
    }

    #[test]
    fn parse_ta_dashboard_custom_interval_and_points() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "ta",
            "dashboard",
            "AAPL",
            "--interval",
            "weekly",
            "--points",
            "10",
        ]);

        let Command::Ta(TaCommand::Dashboard(args)) = cli.command else {
            panic!("expected ta dashboard command");
        };
        assert_eq!(args.symbol, "AAPL");
        assert_eq!(args.interval, "weekly");
        assert_eq!(args.points, 10);
    }

    #[test]
    fn parse_ta_expected_move_defaults() {
        let cli = Cli::parse_from(["schwab-agent", "ta", "expected-move", "AAPL"]);

        let Command::Ta(TaCommand::ExpectedMove(args)) = cli.command else {
            panic!("expected ta expected-move command");
        };
        assert_eq!(args.symbol, "AAPL");
        assert_eq!(args.dte, 30);
    }

    #[test]
    fn parse_ta_expected_move_custom_dte() {
        let cli = Cli::parse_from(["schwab-agent", "ta", "expected-move", "AAPL", "--dte", "45"]);

        let Command::Ta(TaCommand::ExpectedMove(args)) = cli.command else {
            panic!("expected ta expected-move command");
        };
        assert_eq!(args.symbol, "AAPL");
        assert_eq!(args.dte, 45);
    }

    #[test]
    fn parse_analyze_multiple_symbols() {
        let cli = Cli::parse_from(["schwab-agent", "analyze", "AAPL", "MSFT"]);

        let Command::Analyze(args) = cli.command else {
            panic!("expected analyze command");
        };
        assert_eq!(args.symbols, ["AAPL", "MSFT"]);
        assert_eq!(args.interval, "daily");
        assert_eq!(args.points, 1);
    }

    #[test]
    fn parse_analyze_custom_interval_and_points() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "analyze",
            "AAPL",
            "--interval",
            "daily",
            "--points",
            "5",
        ]);

        let Command::Analyze(args) = cli.command else {
            panic!("expected analyze command");
        };
        assert_eq!(args.symbols, ["AAPL"]);
        assert_eq!(args.interval, "daily");
        assert_eq!(args.points, 5);
    }

    #[test]
    fn parse_order_equity_buy_dry_run() {
        let cli = Cli::parse_from(["schwab-agent", "order", "equity", "buy", "AAPL", "-q", "10"]);

        assert_eq!(cli.command_name(), "order");

        let Command::Order(OrderCommand::Equity(super::EquityArgs::Buy(args))) = cli.command else {
            panic!("expected order equity buy command");
        };
        assert_eq!(args.symbol, "AAPL");
        assert_eq!(args.quantity, 10.0);
        assert!(args.price.is_none());
        assert!(args.stop.is_none());
        assert!(args.common.account.is_none());
        assert!(!args.common.dry_run);
        assert!(!args.common.preview);
        assert!(!args.common.save_preview);
        assert!(!args.common.preview_first);
    }

    #[test]
    fn parse_order_equity_buy_explicit_dry_run() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "equity",
            "buy",
            "AAPL",
            "-q",
            "10",
            "--dry-run",
        ]);

        assert_matches!(
            cli.command,
            Command::Order(OrderCommand::Equity(super::EquityArgs::Buy(
                super::EquityOrderArgs {
                    common: super::CommonOrderArgs {
                        dry_run: true,
                        preview: false,
                        account: None,
                        ..
                    },
                    ..
                }
            )))
        );
    }

    #[test]
    fn parse_order_equity_buy_preview_alias() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "equity",
            "buy",
            "AAPL",
            "-q",
            "10",
            "--preview",
        ]);

        assert_matches!(
            cli.command,
            Command::Order(OrderCommand::Equity(super::EquityArgs::Buy(
                super::EquityOrderArgs {
                    common: super::CommonOrderArgs {
                        dry_run: false,
                        preview: true,
                        account: None,
                        ..
                    },
                    ..
                }
            )))
        );
    }

    #[test]
    fn parse_order_dry_run_conflicts_with_save_preview() {
        let err = Cli::try_parse_from([
            "schwab-agent",
            "order",
            "equity",
            "buy",
            "AAPL",
            "-q",
            "10",
            "--dry-run",
            "--account",
            "HASH123",
            "--save-preview",
        ])
        .expect_err("draft mode should conflict with account-backed preview");

        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn parse_order_preview_conflicts_with_preview_first() {
        let err = Cli::try_parse_from([
            "schwab-agent",
            "order",
            "option",
            "buy-to-open",
            "AAPL  250117C00150000",
            "-q",
            "1",
            "--preview",
            "--account",
            "HASH123",
            "--preview-first",
        ])
        .expect_err("local preview should conflict with preview-first");

        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn parse_order_dry_run_conflicts_with_preview_alias() {
        let err = Cli::try_parse_from([
            "schwab-agent",
            "order",
            "equity",
            "buy",
            "AAPL",
            "-q",
            "10",
            "--dry-run",
            "--preview",
        ])
        .expect_err("draft aliases should conflict with each other");

        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn command_name_orders_alias() {
        let cli = Cli::parse_from(["schwab-agent", "orders", "--symbol", "AAPL"]);
        assert_eq!(cli.command_name(), "order.get");
    }

    #[test]
    fn command_name_order_get() {
        let cli = Cli::parse_from(["schwab-agent", "order", "get", "--symbol", "AAPL"]);
        assert_eq!(cli.command_name(), "order.get");
    }

    #[test]
    fn orders_alias_parses_order_get_args() {
        let cli = Cli::parse_from(["schwab-agent", "orders", "--symbol", "AAPL"]);

        let args = expect_orders_alias(cli.command);
        assert_eq!(args.symbol.as_deref(), Some("AAPL"));
        assert!(args.account.is_none());
    }

    #[test]
    fn uppercase_duration_aliases_parse() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "equity",
            "buy",
            "AAPL",
            "-q",
            "10",
            "--duration",
            "GTC",
        ]);

        assert_eq!(
            std::mem::discriminant(&expect_equity_buy_duration(cli.command)),
            std::mem::discriminant(&crate::shared::DurationChoice::GoodTillCancel)
        );

        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "equity",
            "buy",
            "AAPL",
            "-q",
            "10",
            "--duration",
            "DAY",
        ]);

        assert_eq!(
            std::mem::discriminant(&expect_equity_buy_duration(cli.command)),
            std::mem::discriminant(&crate::shared::DurationChoice::Day)
        );
    }

    #[test]
    fn human_session_aliases_parse() {
        for (alias, expected) in [
            ("regular", crate::shared::SessionChoice::Normal),
            ("pre", crate::shared::SessionChoice::Am),
            ("post", crate::shared::SessionChoice::Pm),
            ("extended", crate::shared::SessionChoice::Seamless),
        ] {
            let cli = Cli::parse_from([
                "schwab-agent",
                "order",
                "equity",
                "buy",
                "AAPL",
                "-q",
                "10",
                "--session",
                alias,
            ]);

            assert_matches!(
                cli.command,
                Command::Order(OrderCommand::Equity(super::EquityArgs::Buy(
                    super::EquityOrderArgs {
                        common: super::CommonOrderArgs { session, .. },
                        ..
                    }
                ))) if matches!((session, expected),
                    (crate::shared::SessionChoice::Normal, crate::shared::SessionChoice::Normal)
                        | (crate::shared::SessionChoice::Am, crate::shared::SessionChoice::Am)
                        | (crate::shared::SessionChoice::Pm, crate::shared::SessionChoice::Pm)
                        | (crate::shared::SessionChoice::Seamless, crate::shared::SessionChoice::Seamless))
            );
        }
    }

    #[test]
    fn parse_order_equity_buy_with_account_and_price() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "equity",
            "buy",
            "AAPL",
            "-q",
            "10",
            "-p",
            "150.00",
            "-a",
            "HASH123",
        ]);

        let Command::Order(OrderCommand::Equity(super::EquityArgs::Buy(args))) = cli.command else {
            panic!("expected order equity buy command");
        };
        assert_eq!(args.symbol, "AAPL");
        assert_eq!(args.quantity, 10.0);
        assert_eq!(args.price, Some(150.0));
        assert_eq!(args.common.account.as_deref(), Some("HASH123"));
        assert!(!args.common.dry_run);
        assert!(!args.common.preview);
    }

    #[test]
    fn parse_order_equity_sell_short() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "equity",
            "sell-short",
            "TSLA",
            "-q",
            "5",
            "--stop",
            "200.00",
        ]);

        let Command::Order(OrderCommand::Equity(super::EquityArgs::SellShort(args))) = cli.command
        else {
            panic!("expected order equity sell-short command");
        };
        assert_eq!(args.symbol, "TSLA");
        assert_eq!(args.quantity, 5.0);
        assert_eq!(args.stop, Some(200.0));
    }

    #[test]
    fn parse_order_option_buy_to_open() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "option",
            "buy-to-open",
            "AAPL  250117C00150000",
            "-q",
            "1",
            "-p",
            "3.50",
            "-a",
            "HASH123",
            "--save-preview",
        ]);

        let Command::Order(OrderCommand::Option(super::OptionArgs::BuyToOpen(args))) = cli.command
        else {
            panic!("expected order option buy-to-open command");
        };
        assert_eq!(args.symbol, "AAPL  250117C00150000");
        assert_eq!(args.quantity, 1.0);
        assert_eq!(args.price, Some(3.50));
        assert_eq!(args.common.account.as_deref(), Some("HASH123"));
        assert!(!args.common.dry_run);
        assert!(!args.common.preview);
        assert!(args.common.save_preview);
        assert!(!args.common.preview_first);
    }

    #[test]
    fn parse_order_option_sell_to_close() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "option",
            "sell-to-close",
            "AAPL  250117C00150000",
            "-q",
            "2",
        ]);

        let Command::Order(OrderCommand::Option(super::OptionArgs::SellToClose(args))) =
            cli.command
        else {
            panic!("expected order option sell-to-close command");
        };
        assert_eq!(args.symbol, "AAPL  250117C00150000");
        assert_eq!(args.quantity, 2.0);
        assert!(args.price.is_none());
        assert!(args.common.account.is_none());
        assert!(!args.common.dry_run);
        assert!(!args.common.preview);
    }

    #[test]
    fn parse_order_replace() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "replace",
            "-a",
            "HASH123",
            "--order-id",
            "12345",
            "equity",
            "buy",
            "AAPL",
            "-q",
            "10",
            "-p",
            "150.00",
        ]);

        let Command::Order(OrderCommand::Replace(args)) = cli.command else {
            panic!("expected order replace command");
        };
        assert_eq!(args.account, "HASH123");
        assert_eq!(args.order_id, 12345);
    }

    #[test]
    fn parse_order_place_from_preview() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "place-from-preview",
            "-a",
            "HASH123",
            "-d",
            "abc123",
        ]);

        let Command::Order(OrderCommand::PlaceFromPreview(args)) = cli.command else {
            panic!("expected order place-from-preview command");
        };
        assert_eq!(args.account, "HASH123");
        assert_eq!(args.digest, "abc123");
    }

    #[test]
    fn parse_order_preview_raw() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "preview-raw",
            "--account",
            "HASH123",
            "--json",
            "{\"orderType\":\"LIMIT\"}",
            "--save-preview",
        ]);

        let Command::Order(OrderCommand::PreviewRaw(args)) = cli.command else {
            panic!("expected order preview-raw command");
        };
        assert_eq!(args.account, "HASH123");
        assert!(args.save_preview);
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn parse_order_place_raw() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "place-raw",
            "--account",
            "HASH123",
            "--json",
            "{\"orderType\":\"MARKET\"}",
        ]);

        let Command::Order(OrderCommand::PlaceRaw(args)) = cli.command else {
            panic!("expected order place-raw command");
        };
        assert_eq!(args.account, "HASH123");
    }

    #[test]
    fn stock_subcommand_parses_for_migration_hint() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "stock",
            "buy",
            "AAPL",
            "-q",
            "10",
            "--price",
            "100",
        ]);

        assert_eq!(cli.command_name(), "stock");
        assert_matches!(cli.command, Command::Stock(super::StockCommand::Buy(_)));
    }

    #[test]
    fn removed_global_credential_flags_are_unknown() {
        for flag in [
            "--token",
            "--client-id",
            "--client-secret",
            "--callback-url",
        ] {
            let result = Cli::try_parse_from(["schwab-agent", flag, "value", "auth", "status"]);
            assert_eq!(result.unwrap_err().kind(), ErrorKind::UnknownArgument);
        }
    }
}
