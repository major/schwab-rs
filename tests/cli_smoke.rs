#![cfg(feature = "cli")]

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn agent() -> Command {
    let mut command = Command::cargo_bin("schwab-agent").expect("schwab-agent binary exists");
    command.env("CLAP_COLOR", "never").env("NO_COLOR", "1");
    command
}

fn help_contains(args: &[&str], expected: &[&str]) {
    let mut assert = agent().args(args).assert().success();
    for item in expected {
        assert = assert.stdout(predicate::str::contains(*item));
    }
}

fn json_usage_error(args: &[&str]) -> Value {
    let output = agent()
        .env("SCHWAB_AGENT_JSON_ERRORS", "1")
        .args(args)
        .output()
        .expect("command runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("stdout is JSON")
}

#[test]
fn help_lists_command_groups() {
    agent()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("market"))
        .stdout(predicate::str::contains("quote"))
        .stdout(predicate::str::contains("history"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("order"))
        .stdout(predicate::str::contains("orders"))
        .stdout(predicate::str::contains("positions"))
        .stdout(predicate::str::contains("schema"))
        .stdout(predicate::str::contains("completions"))
        .stdout(predicate::str::contains("analyze"));
}

#[test]
fn top_level_help_documents_setup_environment_and_debug() {
    help_contains(
        &["--help"],
        &[
            "schwab-agent config status",
            "schwab-agent config show",
            "schwab-agent doctor",
            "schwab-agent schema",
            "schwab-agent completions bash",
            "schwab-agent completion zsh",
            "SCHWAB_CLIENT_ID",
            "SCHWAB_CLIENT_SECRET",
            "SCHWAB_CALLBACK_URL",
            "SCHWAB_TOKEN_PATH",
            "XDG_CONFIG_HOME",
            "XDG_STATE_HOME",
            "RUST_LOG",
            "Precedence: command flags > environment variables > config file > defaults",
        ],
    );
}

#[test]
fn config_status_reports_sanitized_setup_without_secret_values() {
    let tempdir = tempfile::tempdir().expect("temporary directory");
    let config_dir = tempdir.path().join("schwab-agent");
    std::fs::create_dir_all(&config_dir).expect("config directory");
    std::fs::write(
        config_dir.join("config.json"),
        r#"{
            "client_id": "file-client-id-secret",
            "client_secret": "file-client-secret-secret",
            "callback_url": "https://127.0.0.1:8182/callback",
            "i-also-like-to-live-dangerously": true
        }"#,
    )
    .expect("config file");
    let token_path = tempdir.path().join("token.json");
    std::fs::write(&token_path, "{}").expect("token marker");

    let output = agent()
        .env("XDG_CONFIG_HOME", tempdir.path())
        .env("SCHWAB_TOKEN_PATH", &token_path)
        .env("SCHWAB_CLIENT_ID", "env-client-id-secret")
        .env_remove("SCHWAB_CLIENT_SECRET")
        .env_remove("SCHWAB_CALLBACK_URL")
        .env("RUST_LOG", "schwab=debug")
        .args(["config", "status"])
        .output()
        .expect("command runs");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout utf8");
    assert!(output.stderr.is_empty());
    assert!(!stdout.contains("env-client-id-secret"));
    assert!(!stdout.contains("file-client-id-secret"));
    assert!(!stdout.contains("file-client-secret-secret"));
    assert!(!stdout.contains("127.0.0.1:8182/callback"));

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["config_present"], true);
    assert_eq!(body["token_present"], true);
    assert_eq!(body["credential_sources"]["client_id"], "environment");
    assert_eq!(body["credential_sources"]["client_secret"], "config_file");
    assert_eq!(body["callback_url_source"], "config_file");
    assert_eq!(body["token_path_source"], "environment");
    assert_eq!(body["mutable_operations_enabled"], true);
    assert_eq!(body["debug"]["rust_log_enabled"], true);
    assert_eq!(body["debug"]["stderr_only"], true);
    assert!(
        body["environment_variables"]
            .as_array()
            .expect("env var list")
            .iter()
            .any(|item| item == "RUST_LOG")
    );
    assert!(
        body["environment_variables"]
            .as_array()
            .expect("env var list")
            .iter()
            .any(|item| item == "SCHWAB_AGENT_JSON_ERRORS")
    );
}

#[test]
fn config_show_alias_reports_sanitized_setup_without_auth() {
    let tempdir = tempfile::tempdir().expect("temporary directory");
    let output = agent()
        .env("XDG_CONFIG_HOME", tempdir.path())
        .env_remove("SCHWAB_CLIENT_ID")
        .env_remove("SCHWAB_CLIENT_SECRET")
        .env_remove("SCHWAB_CALLBACK_URL")
        .args(["config", "show"])
        .output()
        .expect("command runs");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["credential_sources"]["client_id"], "missing");
    assert_eq!(body["credential_sources"]["client_secret"], "missing");
}

#[test]
fn doctor_reports_sanitized_health_without_auth_or_accounts() {
    let tempdir = tempfile::tempdir().expect("temporary directory");
    let output = agent()
        .env("XDG_CONFIG_HOME", tempdir.path())
        .env_remove("SCHWAB_CLIENT_ID")
        .env_remove("SCHWAB_CLIENT_SECRET")
        .env_remove("SCHWAB_CALLBACK_URL")
        .args(["doctor"])
        .output()
        .expect("command runs");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["status"], "ok");
    assert_eq!(body["config"]["credential_sources"]["client_id"], "missing");
    assert!(
        body["summary"]
            .as_str()
            .is_some_and(|summary| { summary.contains("without reading account data") })
    );
}

#[test]
fn schema_reports_agent_discovery_without_auth_or_accounts() {
    let output = agent()
        .env_remove("SCHWAB_CLIENT_ID")
        .env_remove("SCHWAB_CLIENT_SECRET")
        .env_remove("SCHWAB_CALLBACK_URL")
        .args(["schema"])
        .output()
        .expect("command runs");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["name"], "schwab-agent");
    assert!(body["version"].as_str().is_some());
    assert!(
        body["docs_url"]
            .as_str()
            .is_some_and(|url| { url.contains("github.com/major/schwab-rs") })
    );
    assert!(body["commands"].as_array().is_some_and(|commands| {
        commands.iter().any(|command| {
            command["name"] == "order cancel" && command["classification"] == "mutating"
        }) && commands
            .iter()
            .any(|command| command["name"] == "schema" && command["classification"] == "local_only")
            && commands.iter().any(|command| {
                command["name"] == "quote" && command["classification"] == "read_only"
            })
            && commands.iter().any(|command| {
                command["name"] == "orders" && command["classification"] == "read_only"
            })
            && commands.iter().any(|command| {
                command["name"] == "stock buy" && command["classification"] == "local_only"
            })
            && commands.iter().any(|command| {
                command["name"] == "completion" && command["classification"] == "local_only"
            })
    }));
    assert!(
        body["environment_variables"]
            .as_array()
            .is_some_and(|vars| {
                vars.iter()
                    .any(|var| var["name"] == "SCHWAB_CLIENT_SECRET" && var["sensitive"] == true)
            })
    );
    assert!(
        body["output_formats"]
            .as_array()
            .is_some_and(|formats| { formats.iter().any(|format| format["name"] == "usage_json") })
    );
    assert!(body["exit_codes"].as_array().is_some_and(|codes| {
        codes
            .iter()
            .any(|code| code["code"] == 2 && code["category"] == "usage")
    }));
    assert!(body["field_selectors"].as_array().is_some_and(|selectors| {
        selectors.iter().any(|selector| {
            selector["command"] == "market quote"
                && selector["default_fields"]
                    .as_array()
                    .is_some_and(|fields| fields.iter().any(|field| field == "sym"))
        }) && selectors.iter().any(|selector| {
            selector["command"] == "option chain"
                && selector["available_fields"]
                    .as_array()
                    .is_some_and(|fields| fields.iter().any(|field| field == "delta"))
        })
    }));
}

#[test]
fn stock_buy_reports_migration_replacement_as_json() {
    let output = agent()
        .args(["stock", "buy", "AAPL", "-q", "1", "--price", "100"])
        .output()
        .expect("command runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["code"], "usage.migration");
    assert_eq!(body["category"], "usage");
    assert_eq!(body["retryable"], false);
    assert!(body["message"].as_str().is_some_and(|message| {
        message.contains("stock buy") && message.contains("order equity buy")
    }));
    assert!(body["hint"].as_str().is_some_and(|hint| {
        hint.contains("schwab-agent order equity buy SYMBOL -q QUANTITY --price PRICE")
    }));
}

#[test]
fn stock_sell_reports_migration_replacement_as_json() {
    let output = agent()
        .args(["stock", "sell", "AAPL", "-q", "1", "--price", "100"])
        .output()
        .expect("command runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["code"], "usage.migration");
    assert!(body["hint"].as_str().is_some_and(|hint| {
        hint.contains("schwab-agent order equity sell SYMBOL -q QUANTITY --price PRICE")
    }));
}

#[test]
fn completions_output_supported_shell_scripts() {
    for (args, marker) in [
        (["completions", "bash"], "_schwab-agent"),
        (["completion", "zsh"], "#compdef schwab-agent"),
        (["completions", "fish"], "complete -c schwab-agent"),
        (["completions", "powershell"], "Register-ArgumentCompleter"),
    ] {
        let output = agent()
            .args(args)
            .output()
            .expect("completion command runs");

        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8(output.stdout).expect("completion script is utf8");
        assert!(stdout.contains(marker));
        assert!(stdout.contains("order"));
        assert!(stdout.contains("equity"));
        if args == ["completions", "bash"] {
            assert!(stdout.contains("--order-id"));
            assert!(stdout.contains("bash"));
            assert!(stdout.contains("zsh"));
        }
    }
}

#[test]
fn invalid_flag_reports_clap_usage_error() {
    agent()
        .arg("--definitely-invalid")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unexpected argument"))
        .stderr(predicate::str::contains("Usage:"));
}

#[test]
fn unknown_subcommand_reports_json_usage_error_when_requested() {
    let body = json_usage_error(&["stonk", "buy", "AAPL", "-q", "1"]);

    assert_eq!(body["code"], "usage.unknown_command");
    assert_eq!(body["category"], "usage");
    assert_eq!(body["retryable"], false);
    assert!(body["message"].as_str().is_some_and(|message| {
        message.contains("unrecognized subcommand") && message.contains("stonk")
    }));
    assert!(
        body["hint"]
            .as_str()
            .is_some_and(|hint| { hint.contains("command") && hint.contains("--help") })
    );
}

#[test]
fn invalid_value_reports_json_usage_error_when_requested() {
    let body = json_usage_error(&["completions", "not-a-shell"]);

    assert_eq!(body["code"], "usage.invalid_value");
    assert_eq!(body["category"], "usage");
    assert_eq!(body["retryable"], false);
    assert!(body["message"].as_str().is_some_and(|message| {
        message.contains("invalid value") && message.contains("not-a-shell")
    }));
    assert!(
        body["hint"]
            .as_str()
            .is_some_and(|hint| { hint.contains("accepted values") || hint.contains("--help") })
    );
}

#[test]
fn missing_required_argument_reports_json_usage_error_when_requested() {
    let body = json_usage_error(&["market", "quote"]);

    assert_eq!(body["code"], "usage.missing_required");
    assert_eq!(body["category"], "usage");
    assert_eq!(body["retryable"], false);
    assert!(
        body["message"]
            .as_str()
            .is_some_and(|message| { message.contains("required") && message.contains("SYMBOLS") })
    );
    assert!(
        body["hint"]
            .as_str()
            .is_some_and(|hint| { hint.contains("required") && hint.contains("--help") })
    );
}

#[test]
fn argument_conflict_reports_json_usage_error_when_requested() {
    let body = json_usage_error(&["market", "quote", "AAPL", "--fields", "sym", "--all-fields"]);

    assert_eq!(body["code"], "usage.argument_conflict");
    assert_eq!(body["category"], "usage");
    assert_eq!(body["retryable"], false);
    assert!(body["message"].as_str().is_some_and(|message| {
        message.contains("cannot be used") && message.contains("--fields")
    }));
    assert!(body["hint"].as_str().is_some_and(|hint| {
        hint.contains("conflicting") || hint.contains("valid combinations")
    }));
}

#[test]
fn missing_token_error_is_structured_json() {
    let tempdir = tempfile::tempdir().expect("temporary directory");
    let token_path = tempdir.path().join("missing-token.json");

    let output = agent()
        .env("SCHWAB_TOKEN_PATH", &token_path)
        .env("XDG_CONFIG_HOME", tempdir.path())
        .env_remove("SCHWAB_CLIENT_ID")
        .env_remove("SCHWAB_CLIENT_SECRET")
        .env_remove("SCHWAB_CALLBACK_URL")
        .args(["market", "quote", "AAPL"])
        .output()
        .expect("command runs");

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["code"], "auth.token_missing");
    assert_eq!(body["category"], "auth");
    assert_eq!(body["retryable"], false);
    assert!(body["message"].as_str().is_some_and(|message| {
        message.contains("token file not found") && message.contains("missing-token.json")
    }));
    assert!(
        body["hint"].as_str().is_some_and(|hint| {
            hint.contains("auth login-url") && hint.contains("auth exchange")
        })
    );
}

#[test]
fn dry_run_equity_order_outputs_order_json_without_auth() {
    let tempdir = tempfile::tempdir().expect("temporary directory");
    let token_path = tempdir.path().join("unused-token.json");

    let output = agent()
        .env("SCHWAB_TOKEN_PATH", &token_path)
        .env("XDG_CONFIG_HOME", tempdir.path())
        .env_remove("SCHWAB_CLIENT_ID")
        .env_remove("SCHWAB_CLIENT_SECRET")
        .env_remove("SCHWAB_CALLBACK_URL")
        .args([
            "order", "equity", "buy", "AAPL", "-q", "10", "--price", "150.00",
        ])
        .output()
        .expect("command runs");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["orderType"], "LIMIT");
    assert_eq!(body["session"], "NORMAL");
    assert_eq!(body["duration"], "DAY");
    assert_eq!(body["orderStrategyType"], "SINGLE");
    assert_eq!(body["orderLegCollection"][0]["instruction"], "BUY");
    assert_eq!(
        body["orderLegCollection"][0]["instrument"]["symbol"],
        "AAPL"
    );
    assert_eq!(
        body["orderLegCollection"][0]["instrument"]["assetType"],
        "EQUITY"
    );
}

#[test]
fn explicit_dry_run_equity_order_outputs_order_json_without_auth() {
    let tempdir = tempfile::tempdir().expect("temporary directory");
    let token_path = tempdir.path().join("unused-token.json");

    let output = agent()
        .env("SCHWAB_TOKEN_PATH", &token_path)
        .env("XDG_CONFIG_HOME", tempdir.path())
        .env_remove("SCHWAB_CLIENT_ID")
        .env_remove("SCHWAB_CLIENT_SECRET")
        .env_remove("SCHWAB_CALLBACK_URL")
        .args([
            "order",
            "equity",
            "buy",
            "AAPL",
            "-q",
            "1",
            "--price",
            "100.00",
            "--dry-run",
        ])
        .output()
        .expect("command runs");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["orderType"], "LIMIT");
    assert_eq!(body["orderLegCollection"][0]["instruction"], "BUY");
    assert_eq!(
        body["orderLegCollection"][0]["instrument"]["symbol"],
        "AAPL"
    );
}

#[test]
fn preview_alias_option_order_outputs_order_json_without_auth() {
    let tempdir = tempfile::tempdir().expect("temporary directory");
    let token_path = tempdir.path().join("unused-token.json");

    let output = agent()
        .env("SCHWAB_TOKEN_PATH", &token_path)
        .env("XDG_CONFIG_HOME", tempdir.path())
        .env_remove("SCHWAB_CLIENT_ID")
        .env_remove("SCHWAB_CLIENT_SECRET")
        .env_remove("SCHWAB_CALLBACK_URL")
        .args([
            "order",
            "option",
            "buy-to-open",
            "AAPL  250117C00150000",
            "-q",
            "1",
            "--price",
            "5.00",
            "--preview",
        ])
        .output()
        .expect("command runs");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["orderType"], "LIMIT");
    assert_eq!(body["orderLegCollection"][0]["instruction"], "BUY_TO_OPEN");
    assert_eq!(
        body["orderLegCollection"][0]["instrument"]["symbol"],
        "AAPL  250117C00150000"
    );
}

#[test]
fn market_history_help_shows_accepted_date_formats() {
    help_contains(
        &["market", "history", "--help"],
        &[
            "YYYY-MM-DD",
            "RFC3339",
            "epoch milliseconds",
            "2026-01-01",
            "2026-01-01T09:30:00Z",
            "1767225600000",
        ],
    );
}

#[test]
fn market_quote_help_shows_examples() {
    help_contains(
        &["market", "quote", "--help"],
        &[
            "schwab-agent market quote AAPL",
            "schwab-agent market quote AAPL MSFT GOOG --fields sym,last,pct,vol",
            "schwab-agent market quote AAPL --all-fields",
        ],
    );
}

#[test]
fn market_history_help_shows_examples() {
    help_contains(
        &["market", "history", "--help"],
        &[
            "schwab-agent market history SPY",
            "schwab-agent market history SPY --from 2026-01-01 --to 2026-01-31 --fields ts,close,vol",
            "schwab-agent market history AAPL --period-type day --period 5 --frequency-type minute --frequency 5 --extended-hours",
        ],
    );
}

#[test]
fn option_chain_help_shows_examples_and_type_values() {
    help_contains(
        &["option", "chain", "--help"],
        &[
            "schwab-agent option chain AAPL",
            "schwab-agent option chain AAPL --type call --dte 30 --fields strike,delta,bid,ask,volume,oi",
            "schwab-agent option chain AMD --type put --strike-min 140 --strike-max 160 --delta-min -0.30 --delta-max -0.15",
            "Valid --type values: call, put, all",
        ],
    );
}

#[test]
fn option_screen_help_shows_examples_and_type_values() {
    help_contains(
        &["option", "screen", "--help"],
        &[
            "schwab-agent option screen AAPL --type call --dte-min 20 --dte-max 45 --min-volume 100 --min-oi 500 --max-spread-pct 10",
            "schwab-agent option screen SPY --type put --min-premium 1.00 --max-premium 5.00 --limit 20",
            "Valid --type values: call, put, all",
            "Numeric filters must be finite values",
        ],
    );
}

#[test]
fn analyze_help_shows_examples_and_expected_move() {
    help_contains(
        &["analyze", "--help"],
        &[
            "schwab-agent analyze AAPL",
            "schwab-agent analyze AAPL MSFT GOOG",
            "schwab-agent analyze AAPL --expected-move --dte 45",
            "--points <POINTS>",
            "--expected-move",
        ],
    );
}

#[test]
fn market_history_invalid_date_reports_structured_error_without_auth() {
    let tempdir = tempfile::tempdir().expect("temporary directory");
    let token_path = tempdir.path().join("unused-token.json");

    let output = agent()
        .env("SCHWAB_TOKEN_PATH", &token_path)
        .env("XDG_CONFIG_HOME", tempdir.path())
        .env_remove("SCHWAB_CLIENT_ID")
        .env_remove("SCHWAB_CLIENT_SECRET")
        .env_remove("SCHWAB_CALLBACK_URL")
        .args(["market", "history", "SPY", "--from", "not-a-date"])
        .output()
        .expect("command runs");

    assert_eq!(output.status.code(), Some(10));
    assert!(output.stderr.is_empty());

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["code"], "market.validation_failed");
    assert_eq!(body["category"], "market");
    assert!(body["message"].as_str().is_some_and(|message| {
        message.contains("not-a-date") && message.contains("expected YYYY-MM-DD")
    }));
}

#[test]
fn market_history_date_inputs_pass_validation_before_auth() {
    for value in ["2026-01-01", "2026-01-01T09:30:00Z", "1767225600000"] {
        let tempdir = tempfile::tempdir().expect("temporary directory");
        let token_path = tempdir.path().join("missing-token.json");

        let output = agent()
            .env("SCHWAB_TOKEN_PATH", &token_path)
            .env("XDG_CONFIG_HOME", tempdir.path())
            .env_remove("SCHWAB_CLIENT_ID")
            .env_remove("SCHWAB_CLIENT_SECRET")
            .env_remove("SCHWAB_CALLBACK_URL")
            .args(["market", "history", "SPY", "--from", value, "--to", value])
            .output()
            .expect("command runs");

        assert_eq!(output.status.code(), Some(3));
        assert!(output.stderr.is_empty());

        let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
        assert_eq!(body["category"], "auth");
    }
}
