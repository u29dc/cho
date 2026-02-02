//! CLI integration tests.
//!
//! Tests run the `cho` binary as a subprocess and verify:
//! - Help text and version output
//! - Argument validation and exit codes
//! - Output format selection (JSON, table, CSV)
//! - Error formatting (JSON on stderr vs human-readable)
//!
//! Note: Tests that require a live Xero connection are not included here.
//! These tests only verify CLI argument parsing and error handling behavior.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

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
