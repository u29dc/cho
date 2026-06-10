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

fn run_toon(home: &Path, args: &[&str]) -> (i32, Value) {
    let output = run_raw(home, args);
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8(output.stdout).expect("toon stdout should be valid utf8");
    let json = toon_format::decode_default(stdout.trim_end()).expect("stdout must decode as Toon");

    (code, json)
}

fn without_elapsed(mut value: Value) -> Value {
    if let Some(meta) = value["meta"].as_object_mut() {
        meta.remove("elapsed");
    }
    value
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
    assert_eq!(json["data"]["defaultOutputFormat"], "json");
    assert_eq!(
        json["data"]["outputFormats"],
        serde_json::json!(["json", "toon"])
    );
}

#[test]
fn bare_invocation_prints_root_help() {
    let home = TempDir::new().expect("temp home");
    let output = run_raw(home.path(), &[]);

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid utf8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Commands:"));
    assert!(!stdout.trim_start().starts_with('{'));
}

#[test]
fn group_invocation_prints_group_help() {
    let home = TempDir::new().expect("temp home");
    let output = run_raw(home.path(), &["invoices"]);

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid utf8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Commands:"));
    assert!(!stdout.trim_start().starts_with('{'));
    assert!(output.stderr.is_empty());
}

#[test]
fn missing_leaf_argument_uses_native_clap_error() {
    let home = TempDir::new().expect("temp home");
    let output = run_raw(home.path(), &["invoices", "get"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid utf8");
    assert!(stderr.contains("required"));
    assert!(stderr.contains("Usage:"));
}

#[test]
fn toon_orientation_commands_match_default_json_shape() {
    let home = TempDir::new().expect("temp home");

    for args in [vec!["tools"], vec!["health"]] {
        let (json_code, json) = run_json(home.path(), &args);
        let mut toon_args = args.clone();
        toon_args.push("--toon");
        let (toon_code, toon) = run_toon(home.path(), &toon_args);

        assert_eq!(
            toon_code,
            json_code,
            "exit code drift for cho {}",
            args.join(" ")
        );
        assert_eq!(
            without_elapsed(toon),
            without_elapsed(json),
            "toon parity drift for cho {}",
            args.join(" ")
        );
    }
}

#[test]
fn removed_text_and_format_flags_are_rejected() {
    let home = TempDir::new().expect("temp home");

    for args in [vec!["--text", "tools"], vec!["--format", "csv", "tools"]] {
        let output = run_raw(home.path(), &args);
        assert_eq!(output.status.code(), Some(2));
        assert!(
            output.stdout.is_empty(),
            "stdout should stay empty on clap errors"
        );
        let stderr = String::from_utf8(output.stderr).expect("stderr should be valid utf8");
        assert!(stderr.contains(args[0]));
    }
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
    assert_eq!(json["error"]["code"], "not_found");
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
    assert_eq!(json["error"]["code"], "auth_required");
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

#[cfg(unix)]
#[test]
fn existing_unwritable_history_log_fails_closed_before_dispatch() {
    use std::os::unix::fs::PermissionsExt;

    let home = TempDir::new().expect("temp home");
    let history_path = home.path().join("history.log");
    fs::write(&history_path, "seed\n").expect("history.log seed should be written");

    let mut readonly = fs::metadata(&history_path)
        .expect("history.log metadata should be readable")
        .permissions();
    readonly.set_mode(0o444);
    fs::set_permissions(&history_path, readonly).expect("history.log should be made read-only");

    let output = run_raw(home.path(), &["tools"]);

    let mut writable = fs::metadata(&history_path)
        .expect("history.log metadata should still be readable")
        .permissions();
    writable.set_mode(0o600);
    let _ = fs::set_permissions(&history_path, writable);

    assert_eq!(output.status.code(), Some(2));
    let json = serde_json::from_slice::<Value>(&output.stdout)
        .expect("stdout must contain an audit error envelope");
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "audit_log_unavailable");
}
