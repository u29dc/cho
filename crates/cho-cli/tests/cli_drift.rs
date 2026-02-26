use std::collections::HashSet;
use std::fs;
use std::path::Path;

use chrono::{Duration, Utc};
use serde_json::{Value, json};
use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn run_json(
    home: &Path,
    args: &[&str],
    with_auth: bool,
    base_url: Option<&str>,
) -> (i32, Value, String) {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("cho");
    cmd.args(args)
        .env("CHO_HOME", home)
        .env("CHO_DISABLE_KEYRING", "1")
        .env_remove("TOOLS_HOME");

    if with_auth {
        cmd.env("CHO_CLIENT_ID", "test-client-id")
            .env("CHO_CLIENT_SECRET", "test-client-secret");
    } else {
        cmd.env_remove("CHO_CLIENT_ID")
            .env_remove("CHO_CLIENT_SECRET");
    }

    if let Some(base_url) = base_url {
        cmd.env("CHO_BASE_URL", base_url);
    } else {
        cmd.env_remove("CHO_BASE_URL");
    }

    let output = cmd.output().expect("command must execute");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8(output.stdout).expect("stdout must be valid utf8");
    let json = serde_json::from_str::<Value>(&stdout)
        .expect("stdout must contain JSON envelope in --json mode");

    (code, json, stdout)
}

fn seed_tokens(home: &Path, access_token: &str, refresh_token: &str) {
    let tokens = json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "expires_at": (Utc::now() + Duration::minutes(30)).to_rfc3339(),
        "refresh_expires_at": (Utc::now() + Duration::hours(1)).to_rfc3339()
    });

    let path = home.join("tokens.json");
    fs::write(
        &path,
        serde_json::to_string(&tokens).expect("tokens json should serialize"),
    )
    .expect("token file should be written");
}

#[test]
fn tools_registry_has_unique_names_and_json_examples() {
    let home = TempDir::new().expect("temp home");
    let (code, json, stdout) = run_json(home.path(), &["tools", "--json"], false, None);

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    assert!(!stdout.contains("\n  "), "json output should stay compact");

    let tools = json["data"]["tools"]
        .as_array()
        .expect("tools must be an array");

    let mut names = HashSet::new();
    for tool in tools {
        let name = tool["name"].as_str().expect("tool name should be a string");
        assert!(
            names.insert(name.to_string()),
            "duplicate tool name: {name}"
        );

        let command = tool["command"]
            .as_str()
            .expect("tool command should be a string");
        assert!(
            command.contains("--json"),
            "tool command should advertise json mode: {command}"
        );
    }

    let global_flags = json["data"]["globalFlags"]
        .as_array()
        .expect("globalFlags should be an array")
        .iter()
        .map(|item| item["name"].as_str().unwrap_or_default())
        .collect::<HashSet<_>>();

    for required in [
        "--json",
        "--format",
        "--limit",
        "--all",
        "--verbose",
        "--raw",
        "--precise",
    ] {
        assert!(
            global_flags.contains(required),
            "missing global flag metadata for {required}"
        );
    }
}

#[test]
fn config_set_secret_redacts_value_in_audit_log() {
    let home = TempDir::new().expect("temp home");

    let (code, json, _) = run_json(
        home.path(),
        &[
            "config",
            "set",
            "auth.client_secret",
            "super-secret-value",
            "--json",
        ],
        false,
        None,
    );

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);

    let history =
        fs::read_to_string(home.path().join("history.log")).expect("history log should be written");

    assert!(history.contains("auth.client_secret [REDACTED]"));
    assert!(!history.contains("super-secret-value"));
}

#[test]
fn bank_transactions_list_requires_bank_account_filter() {
    let home = TempDir::new().expect("temp home");

    let (code, json, _) = run_json(
        home.path(),
        &["bank-transactions", "list", "--json"],
        true,
        None,
    );

    assert_eq!(code, 1);
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "CONFIG_ERROR");
}

#[test]
fn mutating_commands_are_blocked_when_write_gate_is_disabled() {
    let home = TempDir::new().expect("temp home");

    let (code, json, _) = run_json(
        home.path(),
        &[
            "invoices",
            "create",
            "--file",
            "/tmp/does-not-matter.json",
            "--json",
        ],
        true,
        None,
    );

    assert_eq!(code, 2);
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "WRITE_NOT_ALLOWED");
}

#[tokio::test]
async fn categories_list_handles_grouped_freeagent_response_shape() {
    let home = TempDir::new().expect("temp home");
    seed_tokens(home.path(), "seed-access", "seed-refresh");
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "admin_expenses_categories": [
                {
                    "url": "https://api.freeagent.com/v2/categories/285",
                    "nominal_code": "285",
                    "description": "Accommodation and Meals"
                }
            ],
            "general_categories": [
                {
                    "url": "https://api.freeagent.com/v2/categories/051",
                    "nominal_code": "051",
                    "description": "Interest Received"
                }
            ]
        })))
        .mount(&server)
        .await;

    let (code, json, _) = run_json(
        home.path(),
        &["categories", "list", "--limit", "1", "--json"],
        true,
        Some(&format!("{}/v2/", server.uri())),
    );

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);

    let data = json["data"].as_array().expect("data should be an array");
    assert_eq!(data.len(), 1);
    assert_eq!(json["meta"]["total"], 2);
    assert_eq!(json["meta"]["hasMore"], true);
    assert!(data[0].get("category_group").is_some());
}

#[tokio::test]
async fn reports_cashflow_defaults_to_months_12_when_no_dates_are_provided() {
    let home = TempDir::new().expect("temp home");
    seed_tokens(home.path(), "seed-access", "seed-refresh");
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/cashflow"))
        .and(query_param("months", "12"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "cashflow": {
                "balance": "123.45",
                "from": "2025-01-01",
                "to": "2025-12-31"
            }
        })))
        .mount(&server)
        .await;

    let (code, json, _) = run_json(
        home.path(),
        &["reports", "cashflow", "--json"],
        true,
        Some(&format!("{}/v2/", server.uri())),
    );

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["cashflow"]["balance"], "123.45");
}
