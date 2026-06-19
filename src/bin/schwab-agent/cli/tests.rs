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
        Command::Order(OrderCommand::Equity(super::EquityArgs::Buy(super::EquityOrderArgs {
            common,
            ..
        }))) => common.duration,
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

    let cli = Cli::parse_from(["schwab-agent", "transactions"]);
    assert_eq!(cli.command_name(), "transactions");
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
    let err = Cli::try_parse_from(["schwab-agent", "account", "--positions", "--fields", "sym"])
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

    let Command::Order(OrderCommand::Option(super::OptionArgs::SellToClose(args))) = cli.command
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
