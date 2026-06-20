//! Command-line argument definitions for the `schwab-agent` JSON CLI.

use clap::{Parser, Subcommand};

/// Agent-oriented JSON CLI porcelain for Charles Schwab workflows.
#[derive(Debug, Parser)]
#[command(
    name = "schwab-agent",
    version,
    about = "Agent-oriented JSON CLI porcelain for Charles Schwab workflows",
    long_about = "All normal command output is compact JSON. Use --help on any command for examples and flags. Trading commands intentionally start with draft and validate workflows before placement.\n\nSetup and agent discovery:\n  schwab-agent schema\n      Print machine-readable command capabilities, safety classifications, output formats, environment variables, exit codes, and field selectors.\n  schwab-agent doctor\n      Print sanitized config, auth, token, and debug health without reading account data.\n  schwab-agent config show\n      Produces the same sanitized output as config status.\n  schwab-agent config status\n      Print sanitized config, token, credential-source, precedence, and debug status without exposing secrets.\n\nEnvironment variables:\n  SCHWAB_CLIENT_ID, SCHWAB_CLIENT_SECRET, SCHWAB_CALLBACK_URL\n      Auth credentials. Environment values override ~/.config/schwab-agent/config.json.\n  SCHWAB_TOKEN_PATH\n      Optional token path override. Empty values are ignored.\n  XDG_CONFIG_HOME\n      Base directory for config.json and the default compatibility token path.\n  XDG_STATE_HOME\n      Base directory for saved order previews, falling back to the platform state or local data directory.\n  RUST_LOG\n      Enable tracing diagnostics on stderr, for example RUST_LOG=schwab=debug. JSON stdout remains unchanged.\n  SCHWAB_AGENT_JSON_ERRORS\n      Set to 1 to render clap usage errors as JSON on stdout with code, message, category, retryable, and hint fields. Default clap stderr remains unchanged when unset.\n\nPrecedence: command flags > environment variables > config file > defaults. Defaults include https://127.0.0.1:8182 for the callback URL and $XDG_CONFIG_HOME/schwab-agent-rs/token.json for tokens when SCHWAB_TOKEN_PATH is unset.",
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
            Command::Order(OrderCommand::Get(_)) => "order.get",
            Command::Order(_) => "order",
            Command::Transactions(_) => "transactions",
            Command::Schema => "schema",
            Command::Stock(_) => "stock",
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
    /// Option chain, screening, and contract lookup workflows.
    #[command(subcommand)]
    Option(OptionCommand),
    /// Unified order construction, preview, placement, and lifecycle workflows.
    #[command(subcommand)]
    Order(OrderCommand),
    /// Get account transactions. Defaults to the primary account, last 30 days, and TRADE type.
    Transactions(TransactionsArgs),
    /// Legacy stock command namespace with migration hints.
    #[command(hide = true, subcommand)]
    Stock(StockCommand),
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

pub use crate::account::cli::AccountArgs;
pub use crate::analyze::cli::AnalyzeArgs;
pub use crate::auth::cli::{AuthCommand, AuthExchangeArgs, LoginArgs, LoginUrlArgs};
pub use crate::market::cli::{HistoryArgs, MarketCommand, QuoteArgs};
pub use crate::options::cli::{ChainArgs, ContractArgs, OptionCommand, ScreenArgs};
pub use crate::order::cli::{
    CommonOrderArgs, EquityArgs, EquityOrderArgs, OptionArgs, OptionOrderArgs, OrderCommand,
    PlaceFromPreviewArgs, PlaceRawArgs, PreviewRawArgs, ReplaceArgs, ReplaceOrderSpec,
    StockCommand,
};
pub use crate::ta::cli::{DashboardArgs, ExpectedMoveArgs};
pub use crate::transaction::cli::TransactionsArgs;

#[cfg(test)]
mod tests;
