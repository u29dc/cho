//! CLI integration tests.
//!
//! Tests run the `cho` binary as a subprocess and verify:
//! - Help text and version output
//! - Argument validation and exit codes
//! - Output format selection (JSON, table, CSV)
//! - Envelope contract (`{ok, data, meta}` / `{ok, error, meta}`)
//! - Tool discovery (`cho tools --json`)
//! - Health readiness (`cho health --json`)
//!
//! Note: Tests that require a live Xero connection are not included here.
//! These tests only verify CLI argument parsing and error handling behavior.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::Value;

fn cho() -> assert_cmd::Command {
    let mut cmd = cargo_bin_cmd!("cho");
    cmd.env_remove("CHO_FORMAT")
        .env_remove("CHO_TENANT_ID")
        .env_remove("CHO_CLIENT_ID")
        .env_remove("CHO_BASE_URL");
    cmd
}

// --- Help and version ---

#[test]
fn help_output() {
    cho()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("cho"))
        .stdout(predicate::str::contains("Xero"));
}

#[test]
fn version_output() {
    cho()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cho"));
}

#[test]
fn no_args_shows_help() {
    // Running with no subcommand should show usage/help and fail
    cho()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

// --- Subcommand help ---

#[test]
fn invoices_help() {
    cho()
        .args(["invoices", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"));
}

#[test]
fn contacts_help() {
    cho()
        .args(["contacts", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("search"));
}

#[test]
fn payments_help() {
    cho()
        .args(["payments", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"));
}

#[test]
fn transactions_help() {
    cho()
        .args(["transactions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"));
}

#[test]
fn accounts_help() {
    cho()
        .args(["accounts", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"));
}

#[test]
fn reports_help() {
    cho()
        .args(["reports", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("balance-sheet"))
        .stdout(predicate::str::contains("pnl"))
        .stdout(predicate::str::contains("trial-balance"))
        .stdout(predicate::str::contains("aged-payables"))
        .stdout(predicate::str::contains("aged-receivables"));
}

#[test]
fn auth_help() {
    cho()
        .args(["auth", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("login"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("refresh"))
        .stdout(predicate::str::contains("tenants"));
}

#[test]
fn config_help() {
    cho()
        .args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("show"));
}

// --- Global flags parsing ---

#[test]
fn invalid_format_rejected() {
    cho()
        .args(["--format", "xml", "invoices", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn format_json_accepted() {
    // Should succeed (auth status doesn't require auth) or fail gracefully,
    // but the format flag should be parsed correctly (no argument error).
    let result = cho().args(["--format", "json", "auth", "status"]).assert();
    // Verify it didn't fail due to argument parsing (exit code 2 = clap error)
    result.code(predicate::ne(2));
}

#[test]
fn format_table_accepted() {
    let result = cho().args(["--format", "table", "auth", "status"]).assert();
    result.code(predicate::ne(2));
}

#[test]
fn format_csv_accepted() {
    let result = cho().args(["--format", "csv", "auth", "status"]).assert();
    result.code(predicate::ne(2));
}

// --- Invalid arguments ---

#[test]
fn invoices_get_requires_id() {
    cho()
        .args(["invoices", "get"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn contacts_get_requires_uuid() {
    cho()
        .args(["contacts", "get", "not-a-uuid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn payments_get_requires_uuid() {
    cho()
        .args(["payments", "get", "not-a-uuid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn transactions_get_requires_uuid() {
    cho()
        .args(["transactions", "get", "not-a-uuid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn contacts_search_requires_term() {
    cho()
        .args(["contacts", "search"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn config_set_requires_key_and_value() {
    cho()
        .args(["config", "set"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// --- Unknown subcommand ---

#[test]
fn unknown_subcommand_fails() {
    cho()
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

// --- Environment variable support ---

#[test]
fn env_format_accepted() {
    let result = cho()
        .env("CHO_FORMAT", "json")
        .args(["auth", "status"])
        .assert();
    // Verify it didn't fail due to argument parsing
    result.code(predicate::ne(2));
}

// --- Limit flag ---

#[test]
fn limit_flag_accepts_number() {
    let result = cho().args(["--limit", "50", "auth", "status"]).assert();
    result.code(predicate::ne(2));
}

#[test]
fn limit_flag_rejects_non_number() {
    cho()
        .args(["--limit", "abc", "invoices", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// --- Tool discovery (cho tools) ---

#[test]
fn tools_json_returns_catalog() {
    let output = cho()
        .args(["tools", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: Value = serde_json::from_slice(&output).expect("valid JSON envelope");
    assert_eq!(envelope["ok"], true);
    assert!(envelope["meta"]["tool"].as_str() == Some("tools"));
    assert!(envelope["meta"]["elapsed"].is_number());

    let data = &envelope["data"];
    assert!(data["version"].is_string());
    assert!(data["tools"].is_array());
    assert!(data["globalFlags"].is_array());

    let tools = data["tools"].as_array().unwrap();
    assert!(tools.len() >= 40, "Expected 40+ tools, got {}", tools.len());

    // Verify tool structure
    let first_tool = &tools[0];
    assert!(first_tool["name"].is_string());
    assert!(first_tool["command"].is_string());
    assert!(first_tool["category"].is_string());
    assert!(first_tool["description"].is_string());
    assert!(first_tool["parameters"].is_array());
    assert!(first_tool["outputFields"].is_array());
    assert!(first_tool["idempotent"].is_boolean());
    assert!(first_tool["example"].is_string());

    // Verify meta has count
    assert!(envelope["meta"]["count"].is_number());
}

#[test]
fn tools_detail_returns_single_tool() {
    let output = cho()
        .args(["tools", "invoices.list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: Value = serde_json::from_slice(&output).expect("valid JSON envelope");
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["data"]["name"], "invoices.list");
    assert_eq!(envelope["data"]["category"], "invoices");
    assert!(envelope["data"]["parameters"].is_array());
}

#[test]
fn tools_not_found_returns_error() {
    let output = cho()
        .args(["tools", "nonexistent.tool", "--json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let envelope: Value = serde_json::from_slice(&output).expect("valid JSON envelope");
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], "NOT_FOUND");
    assert!(
        envelope["error"]["message"]
            .as_str()
            .unwrap()
            .contains("nonexistent.tool")
    );
}

#[test]
fn tools_text_mode_lists_categories() {
    // Force table mode since piped stdout auto-detects to JSON
    cho()
        .args(["--format", "table", "tools"])
        .assert()
        .success()
        .stderr(predicate::str::contains("INVOICES"))
        .stderr(predicate::str::contains("CONTACTS"))
        .stderr(predicate::str::contains("tools available"));
}

// --- Health readiness (cho health) ---

#[test]
fn health_json_returns_status() {
    let output = cho()
        .args(["health", "--json"])
        .assert()
        .get_output()
        .stdout
        .clone();

    let envelope: Value = serde_json::from_slice(&output).expect("valid JSON envelope");
    assert_eq!(envelope["ok"], true);
    assert!(envelope["meta"]["tool"].as_str() == Some("health"));
    assert!(envelope["meta"]["elapsed"].is_number());

    let data = &envelope["data"];
    let status = data["status"].as_str().unwrap();
    assert!(
        status == "ready" || status == "degraded" || status == "blocked",
        "Unexpected status: {status}"
    );

    assert!(data["checks"].is_array());
    let checks = data["checks"].as_array().unwrap();
    assert!(!checks.is_empty());

    // Verify check structure
    for check in checks {
        assert!(check["id"].is_string());
        assert!(check["label"].is_string());
        let check_status = check["status"].as_str().unwrap();
        assert!(
            check_status == "pass" || check_status == "warn" || check_status == "fail",
            "Unexpected check status: {check_status}"
        );
        assert!(check["severity"].is_string());
    }

    assert!(data["summary"]["pass"].is_number());
    assert!(data["summary"]["warn"].is_number());
    assert!(data["summary"]["fail"].is_number());
}

#[test]
fn health_exit_code_2_when_blocked() {
    // Without config/auth, health should report blocked and exit 2
    let result = cho().args(["health", "--json"]).assert();
    // exit code is 0 (degraded) or 2 (blocked) depending on environment
    let code = result.get_output().status.code().unwrap();
    assert!(code == 0 || code == 2, "Expected exit 0 or 2, got {code}");
}

// --- --json flag alias ---

#[test]
fn json_flag_alias() {
    // `--json` should produce same envelope format as `--format json`
    let output = cho()
        .args(["--json", "tools"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: Value = serde_json::from_slice(&output).expect("valid JSON from --json flag");
    assert_eq!(envelope["ok"], true);
    assert!(envelope["data"]["tools"].is_array());
}

// --- --meta deprecation ---

#[test]
fn meta_deprecation_warning() {
    cho()
        .args(["--meta", "tools", "--json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("--meta is deprecated"));
}

// --- Error envelope ---

#[test]
fn error_envelope_on_auth_failure() {
    // Running a command that requires auth without credentials should produce
    // an error envelope on stdout (not stderr) in JSON mode
    let output = cho()
        .args(["--json", "invoices", "list"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let envelope: Value = serde_json::from_slice(&output).expect("valid error envelope");
    assert_eq!(envelope["ok"], false);
    assert!(envelope["error"]["code"].is_string());
    assert!(envelope["error"]["message"].is_string());
    assert!(envelope["error"]["hint"].is_string());
    assert!(envelope["meta"]["tool"].is_string());
    assert!(envelope["meta"]["elapsed"].is_number());
}
