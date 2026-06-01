use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn agent() -> Command {
    let mut command = Command::cargo_bin("schwab-agent").expect("schwab-agent binary exists");
    command.env("CLAP_COLOR", "never").env("NO_COLOR", "1");
    command
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
        .stdout(predicate::str::contains("order"))
        .stdout(predicate::str::contains("completions"))
        .stdout(predicate::str::contains("analyze"));
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
    let output = agent()
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
