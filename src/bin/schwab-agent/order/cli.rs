use clap::{Args, Subcommand};

/// Legacy stock commands that now point to `order equity`.
#[derive(Debug, Subcommand)]
pub enum StockCommand {
    /// Use `order equity buy` instead.
    Buy(EquityOrderArgs),
    /// Use `order equity sell` instead.
    Sell(EquityOrderArgs),
}

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
    #[arg(long, requires = "account", conflicts_with_all = ["dry_run", "preview", "preview_first"])]
    pub save_preview: bool,
    /// Call Schwab previewOrder first, then place automatically (requires --account).
    #[arg(long, requires = "account", conflicts_with_all = ["dry_run", "preview", "save_preview"])]
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
