use std::fs;
use std::path::Path;

use serde_json::Value;
use tempfile::TempDir;

fn run_json(home: &Path, args: &[&str]) -> (i32, Value) {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("cho");
    cmd.args(args)
        .env("CHO_HOME", home)
        .env_remove("CHO_CLIENT_ID")
        .env_remove("CHO_CLIENT_SECRET")
        .env_remove("TOOLS_HOME");

    let output = cmd.output().expect("command must execute");
    let code = output.status.code().unwrap_or(-1);
    let json = serde_json::from_slice::<Value>(&output.stdout)
        .expect("stdout must contain JSON envelope in --json mode");

    (code, json)
}

#[test]
fn tools_json_contains_core_contract_commands() {
    let home = TempDir::new().expect("temp home");
    let (code, json) = run_json(home.path(), &["tools", "--json"]);

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
fn tools_get_unknown_returns_not_found_error_envelope() {
    let home = TempDir::new().expect("temp home");
    let (code, json) = run_json(home.path(), &["tools", "does.not.exist", "--json"]);

    assert_eq!(code, 1);
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "NOT_FOUND");
}

#[test]
fn health_json_reports_blocked_without_credentials() {
    let home = TempDir::new().expect("temp home");
    let (code, json) = run_json(home.path(), &["health", "--json"]);

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
    let (code, json) = run_json(home.path(), &["auth", "login", "--json"]);

    assert_eq!(code, 2);
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "AUTH_REQUIRED");
}

#[test]
fn config_show_redacts_client_secret() {
    let home = TempDir::new().expect("temp home");

    let (code_id, id_result) = run_json(
        home.path(),
        &["config", "set", "auth.client_id", "client-id-123", "--json"],
    );
    assert_eq!(code_id, 0);
    assert_eq!(id_result["ok"], true);

    let (code_secret, secret_result) = run_json(
        home.path(),
        &[
            "config",
            "set",
            "auth.client_secret",
            "super-secret-value",
            "--json",
        ],
    );
    assert_eq!(code_secret, 0);
    assert_eq!(secret_result["ok"], true);

    let (code_show, show_result) = run_json(home.path(), &["config", "show", "--json"]);
    assert_eq!(code_show, 0);
    assert_eq!(show_result["ok"], true);
    assert_eq!(show_result["data"]["auth"]["client_id"], "client-id-123");
    assert_eq!(show_result["data"]["auth"]["client_secret"], "[REDACTED]");
}

#[test]
fn command_execution_writes_history_log_entries() {
    let home = TempDir::new().expect("temp home");
    let (code, json) = run_json(home.path(), &["tools", "--json"]);
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
