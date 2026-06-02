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

#[test]
fn help_lists_command_groups() {
    agent()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("market"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("order"))
        .stdout(predicate::str::contains("completions"))
        .stdout(predicate::str::contains("analyze"));
}

#[test]
fn top_level_help_documents_setup_environment_and_debug() {
    help_contains(
        &["--help"],
        &[
            "schwab-agent config status",
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
}

#[test]
fn completions_outputs_shell_script() {
    agent()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains("_schwab-agent"))
        .stdout(predicate::str::contains("complete"));
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
fn ta_dashboard_help_shows_examples() {
    help_contains(
        &["ta", "dashboard", "--help"],
        &[
            "schwab-agent ta dashboard AAPL",
            "default 20 points",
            "schwab-agent ta dashboard SPY --interval weekly --points 10",
        ],
    );
}

#[test]
fn analyze_help_shows_examples_and_compact_default_reason() {
    help_contains(
        &["analyze", "--help"],
        &[
            "schwab-agent analyze AAPL",
            "schwab-agent analyze AAPL MSFT GOOG",
            "schwab-agent analyze AAPL --interval weekly --points 10",
            "The default is 1 point, while ta dashboard defaults to 20",
            "optimized for compact multi-symbol output",
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
