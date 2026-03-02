use std::collections::HashSet;
use std::fs;
use std::path::Path;

use chrono::{Duration, Utc};
use serde_json::{Value, json};
use tempfile::TempDir;
use wiremock::matchers::{body_partial_json, method, path, query_param};
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

fn enable_writes(home: &Path) {
    fs::write(home.join("config.toml"), "[safety]\nallow_writes = true\n")
        .expect("config file should be written");
}

fn run_help(home: &Path, args: &[&str]) -> (i32, String) {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("cho");
    cmd.args(args)
        .env("CHO_HOME", home)
        .env_remove("TOOLS_HOME")
        .env_remove("CHO_CLIENT_ID")
        .env_remove("CHO_CLIENT_SECRET")
        .env_remove("CHO_BASE_URL");

    let output = cmd.output().expect("command must execute");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8(output.stdout).expect("stdout must be valid utf8");
    (code, stdout)
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

#[tokio::test]
async fn bank_transactions_list_without_filter_merges_accounts_sorted_newest_first() {
    let home = TempDir::new().expect("temp home");
    seed_tokens(home.path(), "seed-access", "seed-refresh");
    let server = MockServer::start().await;

    let bank_account_a = "https://api.freeagent.com/v2/bank_accounts/11";
    let bank_account_b = "https://api.freeagent.com/v2/bank_accounts/22";

    Mock::given(method("GET"))
        .and(path("/v2/bank_accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "bank_accounts": [
                { "url": bank_account_a, "name": "Wise GBP" },
                { "url": bank_account_b, "name": "Monzo GBP" }
            ]
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v2/bank_transactions"))
        .and(query_param("bank_account", bank_account_a))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "bank_transactions": [
                { "url": "btx-a-1", "dated_on": "2026-02-20", "description": "Older tx" }
            ]
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v2/bank_transactions"))
        .and(query_param("bank_account", bank_account_b))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "bank_transactions": [
                { "url": "btx-b-1", "dated_on": "2026-03-01", "description": "Newest tx" }
            ]
        })))
        .mount(&server)
        .await;

    let (code, json, _) = run_json(
        home.path(),
        &["bank-transactions", "list", "--json"],
        true,
        Some(&format!("{}/v2/", server.uri())),
    );

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);

    let data = json["data"].as_array().expect("data should be an array");
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["url"], "btx-b-1");
    assert_eq!(data[0]["_bank_account_name"], "Monzo GBP");
    assert_eq!(data[1]["url"], "btx-a-1");
    assert_eq!(data[1]["_bank_account_name"], "Wise GBP");
}

#[tokio::test]
async fn bank_transactions_for_approval_uses_marked_for_review_view() {
    let home = TempDir::new().expect("temp home");
    seed_tokens(home.path(), "seed-access", "seed-refresh");
    let server = MockServer::start().await;

    let bank_account = "https://api.freeagent.com/v2/bank_accounts/11";
    Mock::given(method("GET"))
        .and(path("/v2/bank_accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "bank_accounts": [
                { "url": bank_account, "name": "Monzo GBP" }
            ]
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v2/bank_transactions"))
        .and(query_param("bank_account", bank_account))
        .and(query_param("view", "marked_for_review"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "bank_transactions": [
                { "url": "btx-1", "dated_on": "2026-03-01", "description": "Needs review" }
            ]
        })))
        .mount(&server)
        .await;

    let (code, json, _) = run_json(
        home.path(),
        &["bank-transactions", "for-approval", "--json"],
        true,
        Some(&format!("{}/v2/", server.uri())),
    );

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    let data = json["data"].as_array().expect("data should be an array");
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["url"], "btx-1");
}

#[tokio::test]
async fn bank_transactions_delete_uses_documented_singular_endpoint_path() {
    let home = TempDir::new().expect("temp home");
    enable_writes(home.path());
    seed_tokens(home.path(), "seed-access", "seed-refresh");
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v2/bank_transaction/tx-44"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "deleted"
        })))
        .mount(&server)
        .await;

    let (code, json, _) = run_json(
        home.path(),
        &["bank-transactions", "delete", "tx-44", "--json"],
        true,
        Some(&format!("{}/v2/", server.uri())),
    );

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["status"], "deleted");
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
async fn update_explanation_accepts_local_attachment_path_and_partial_fields() {
    let home = TempDir::new().expect("temp home");
    enable_writes(home.path());
    seed_tokens(home.path(), "seed-access", "seed-refresh");
    let server = MockServer::start().await;

    let receipt_path = home.path().join("receipt.pdf");
    fs::write(&receipt_path, b"%PDF-1.4\nmock").expect("pdf fixture should be written");

    Mock::given(method("GET"))
        .and(path("/v2/bank_transactions/tx-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "bank_transaction": {
                "url": "tx-1",
                "bank_transaction_explanations": [
                    { "url": "exp-1" }
                ]
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/v2/bank_transaction_explanations/exp-1"))
        .and(body_partial_json(json!({
            "bank_transaction_explanation": {
                "description": "Expense: MyMind Subscription",
                "marked_for_review": false,
                "attachment": {
                    "file_name": "receipt.pdf",
                    "content_type": "application/x-pdf"
                }
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "bank_transaction_explanation": {
                "url": "exp-1",
                "description": "Expense: MyMind Subscription"
            }
        })))
        .mount(&server)
        .await;

    let receipt_arg = receipt_path.to_string_lossy().to_string();
    let args = vec![
        "bank-transactions",
        "update-explanation",
        "tx-1",
        "--description",
        "Expense: MyMind Subscription",
        "--mark-for-review",
        "false",
        "--attachment",
        &receipt_arg,
        "--json",
    ];
    let (code, json, _) = run_json(
        home.path(),
        &args,
        true,
        Some(&format!("{}/v2/", server.uri())),
    );

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["description"], "Expense: MyMind Subscription");
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

#[test]
fn help_for_read_only_resources_hides_mutating_commands() {
    let home = TempDir::new().expect("temp home");

    let (recurring_code, recurring_help) = run_help(home.path(), &["recurring-invoices", "--help"]);
    assert_eq!(recurring_code, 0);
    assert!(recurring_help.contains("\n  list"));
    assert!(recurring_help.contains("\n  get"));
    assert!(!recurring_help.contains("\n  create"));
    assert!(!recurring_help.contains("\n  update"));
    assert!(!recurring_help.contains("\n  delete"));

    let (capital_code, capital_help) = run_help(home.path(), &["capital-assets", "--help"]);
    assert_eq!(capital_code, 0);
    assert!(capital_help.contains("\n  list"));
    assert!(capital_help.contains("\n  get"));
    assert!(!capital_help.contains("\n  create"));
    assert!(!capital_help.contains("\n  update"));
    assert!(!capital_help.contains("\n  delete"));
}

#[test]
fn help_for_get_delete_resources_exposes_only_supported_commands() {
    let home = TempDir::new().expect("temp home");
    let (code, help) = run_help(home.path(), &["attachments", "--help"]);

    assert_eq!(code, 0);
    assert!(help.contains("\n  get"));
    assert!(help.contains("\n  delete"));
    assert!(!help.contains("\n  list"));
    assert!(!help.contains("\n  create"));
    assert!(!help.contains("\n  update"));
}

#[tokio::test]
async fn payroll_mark_payment_paid_uses_put_endpoint() {
    let home = TempDir::new().expect("temp home");
    enable_writes(home.path());
    seed_tokens(home.path(), "seed-access", "seed-refresh");
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/v2/payroll/2026/payments/2026-04-30/mark_as_paid"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "paid"
        })))
        .mount(&server)
        .await;

    let (code, json, _) = run_json(
        home.path(),
        &[
            "payroll",
            "mark-payment-paid",
            "2026",
            "2026-04-30",
            "--json",
        ],
        true,
        Some(&format!("{}/v2/", server.uri())),
    );

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["status"], "paid");
}

#[tokio::test]
async fn vat_mark_payment_paid_uses_put_endpoint() {
    let home = TempDir::new().expect("temp home");
    enable_writes(home.path());
    seed_tokens(home.path(), "seed-access", "seed-refresh");
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path(
            "/v2/vat_returns/2026-03-31/payments/2026-05-07/mark_as_paid",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "vat_return": {
                "url": "https://api.freeagent.com/v2/vat_returns/2026-03-31"
            }
        })))
        .mount(&server)
        .await;

    let (code, json, _) = run_json(
        home.path(),
        &[
            "vat-returns",
            "mark-payment-paid",
            "2026-03-31",
            "2026-05-07",
            "--json",
        ],
        true,
        Some(&format!("{}/v2/", server.uri())),
    );

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    assert_eq!(
        json["data"]["vat_return"]["url"],
        "https://api.freeagent.com/v2/vat_returns/2026-03-31"
    );
}

#[test]
fn help_for_list_only_and_write_only_resources_shows_expected_commands() {
    let home = TempDir::new().expect("temp home");

    let (cis_code, cis_help) = run_help(home.path(), &["cis-bands", "--help"]);
    assert_eq!(cis_code, 0);
    assert!(cis_help.contains("\n  list"));
    assert!(!cis_help.contains("\n  get"));
    assert!(!cis_help.contains("\n  create"));

    let (email_code, email_help) = run_help(home.path(), &["email-addresses", "--help"]);
    assert_eq!(email_code, 0);
    assert!(email_help.contains("\n  list"));
    assert!(!email_help.contains("\n  get"));
    assert!(!email_help.contains("\n  create"));

    let (estimate_items_code, estimate_items_help) =
        run_help(home.path(), &["estimate-items", "--help"]);
    assert_eq!(estimate_items_code, 0);
    assert!(estimate_items_help.contains("\n  create"));
    assert!(estimate_items_help.contains("\n  update"));
    assert!(estimate_items_help.contains("\n  delete"));
    assert!(!estimate_items_help.contains("\n  list"));
    assert!(!estimate_items_help.contains("\n  get"));
}

#[tokio::test]
async fn invoices_timeline_hits_dedicated_endpoint() {
    let home = TempDir::new().expect("temp home");
    seed_tokens(home.path(), "seed-access", "seed-refresh");
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/invoices/timeline"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "timeline_events": [
                { "type": "sent", "invoice_url": "https://api.freeagent.com/v2/invoices/1" }
            ]
        })))
        .mount(&server)
        .await;

    let (code, json, _) = run_json(
        home.path(),
        &["invoices", "timeline", "--json"],
        true,
        Some(&format!("{}/v2/", server.uri())),
    );

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["timeline_events"][0]["type"], "sent");
}

#[tokio::test]
async fn timeslips_start_timer_uses_post_endpoint() {
    let home = TempDir::new().expect("temp home");
    enable_writes(home.path());
    seed_tokens(home.path(), "seed-access", "seed-refresh");
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v2/timeslips/42/timer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "timeslip": { "url": "https://api.freeagent.com/v2/timeslips/42", "timer_running": true }
        })))
        .mount(&server)
        .await;

    let (code, json, _) = run_json(
        home.path(),
        &["timeslips", "start-timer", "42", "--json"],
        true,
        Some(&format!("{}/v2/", server.uri())),
    );

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    assert_eq!(
        json["data"]["timeslip"]["url"],
        "https://api.freeagent.com/v2/timeslips/42"
    );
}

#[tokio::test]
async fn users_update_me_uses_put_users_me_endpoint() {
    let home = TempDir::new().expect("temp home");
    enable_writes(home.path());
    seed_tokens(home.path(), "seed-access", "seed-refresh");
    let server = MockServer::start().await;

    let payload_path = home.path().join("user-update.json");
    fs::write(
        &payload_path,
        serde_json::to_string(&json!({
            "user": {
                "first_name": "Ada"
            }
        }))
        .expect("payload json"),
    )
    .expect("payload file should be written");

    Mock::given(method("PUT"))
        .and(path("/v2/users/me"))
        .and(body_partial_json(json!({
            "user": {
                "first_name": "Ada"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "user": { "url": "https://api.freeagent.com/v2/users/1", "first_name": "Ada" }
        })))
        .mount(&server)
        .await;

    let payload_arg = payload_path.to_string_lossy().to_string();
    let args = vec!["users", "update-me", "--file", &payload_arg, "--json"];
    let (code, json, _) = run_json(
        home.path(),
        &args,
        true,
        Some(&format!("{}/v2/", server.uri())),
    );

    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["user"]["first_name"], "Ada");
}
