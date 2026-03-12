use std::fs;
use std::path::Path;

use serde_json::Value;
use tempfile::TempDir;

fn run_raw(home: &Path, args: &[&str]) -> std::process::Output {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("cho");
    cmd.args(args)
        .env("CHO_HOME", home)
        .env_remove("CHO_CLIENT_ID")
        .env_remove("CHO_CLIENT_SECRET")
        .env_remove("TOOLS_HOME");

    cmd.output().expect("command must execute")
}

fn run_json(home: &Path, args: &[&str]) -> (i32, Value) {
    let output = run_raw(home, args);
    let code = output.status.code().unwrap_or(-1);
    let json = serde_json::from_slice::<Value>(&output.stdout)
        .expect("stdout must contain the default JSON envelope");

    (code, json)
}

#[test]
fn tools_json_contains_core_contract_commands() {
    let home = TempDir::new().expect("temp home");
    let (code, json) = run_json(home.path(), &["tools"]);

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);

    let tools = json["data"]["tools"]
        .as_array()
        .expect("tools must be an array");

    let names = tools
        .iter()
        .filter_map(|item| item.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert!(names.contains(&"tools.list"));
    assert!(names.contains(&"health.check"));
    assert!(names.contains(&"auth.login"));
    assert!(names.contains(&"company.get"));
    assert!(names.contains(&"invoices.list"));

    assert!(json["data"]["globalFlags"].is_array());
}

#[test]
fn tools_text_success_writes_to_stdout() {
    let home = TempDir::new().expect("temp home");
    let output = run_raw(home.path(), &["--text", "tools"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(
        String::from_utf8(output.stdout)
            .expect("stdout should be valid utf8")
            .contains("TOOLS")
    );
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr should be valid utf8")
            .is_empty()
    );
}

#[test]
fn tools_csv_format_writes_csv_to_stdout() {
    let home = TempDir::new().expect("temp home");
    let output = run_raw(home.path(), &["--format", "csv", "tools"]);

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid utf8");
    assert!(stdout.starts_with("category,command,description,example"));
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr should be valid utf8")
            .is_empty()
    );
}

#[test]
fn health_table_format_writes_table_to_stdout() {
    let home = TempDir::new().expect("temp home");
    let output = run_raw(home.path(), &["--format", "table", "health"]);

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid utf8");
    assert!(stdout.contains("overall_status"));
    assert!(!stdout.trim_start().starts_with('{'));
}

#[test]
fn config_show_csv_format_writes_csv_to_stdout() {
    let home = TempDir::new().expect("temp home");
    let output = run_raw(home.path(), &["--format", "csv", "config", "show"]);

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid utf8");
    assert!(stdout.starts_with("auth,defaults,safety,sdk"));
    assert!(!stdout.trim_start().starts_with('{'));
}

#[test]
fn legacy_json_flag_is_rejected() {
    let home = TempDir::new().expect("temp home");
    let output = run_raw(home.path(), &["tools", "--json"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(
        output.stdout.is_empty(),
        "stdout should stay empty on clap errors"
    );
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr should be valid utf8")
            .contains("--json")
    );
}

#[test]
fn tools_get_unknown_returns_not_found_error_envelope() {
    let home = TempDir::new().expect("temp home");
    let (code, json) = run_json(home.path(), &["tools", "does.not.exist"]);

    assert_eq!(code, 1);
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "NOT_FOUND");
}

#[test]
fn health_json_reports_blocked_without_credentials() {
    let home = TempDir::new().expect("temp home");
    let (code, json) = run_json(home.path(), &["health"]);

    assert_eq!(code, 2);
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["status"], "blocked");

    let checks = json["data"]["checks"]
        .as_array()
        .expect("checks must be an array");

    let has_credentials_failure = checks.iter().any(|check| {
        check["id"] == "credentials" && check["status"] == "fail" && check["severity"] == "blocking"
    });
    assert!(has_credentials_failure);
}

#[test]
fn auth_login_without_client_credentials_returns_auth_required() {
    let home = TempDir::new().expect("temp home");
    let (code, json) = run_json(home.path(), &["auth", "login"]);

    assert_eq!(code, 2);
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "AUTH_REQUIRED");
}

#[test]
fn config_show_redacts_client_secret() {
    let home = TempDir::new().expect("temp home");

    let (code_id, id_result) = run_json(
        home.path(),
        &["config", "set", "auth.client_id", "client-id-123"],
    );
    assert_eq!(code_id, 0);
    assert_eq!(id_result["ok"], true);

    let (code_secret, secret_result) = run_json(
        home.path(),
        &["config", "set", "auth.client_secret", "super-secret-value"],
    );
    assert_eq!(code_secret, 0);
    assert_eq!(secret_result["ok"], true);

    let (code_show, show_result) = run_json(home.path(), &["config", "show"]);
    assert_eq!(code_show, 0);
    assert_eq!(show_result["ok"], true);
    assert_eq!(show_result["data"]["auth"]["client_id"], "client-id-123");
    assert_eq!(show_result["data"]["auth"]["client_secret"], "[REDACTED]");
}

#[test]
fn command_execution_writes_history_log_entries() {
    let home = TempDir::new().expect("temp home");
    let (code, json) = run_json(home.path(), &["tools"]);
    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);

    let history_path = home.path().join("history.log");
    let history = fs::read_to_string(&history_path).expect("history.log must exist");

    assert!(history.contains("event=command.start"));
    assert!(history.contains("event=command.input"));
    assert!(history.contains("event=command.output"));
    assert!(history.contains("event=command.end"));
    assert!(history.contains("tool=tools.list"));
}
