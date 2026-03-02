//! Shared helper utilities for resource command handlers.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::Instant;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use cho_sdk::error::{ChoSdkError, Result};
use chrono::{DateTime, NaiveDate};
use serde_json::{Map, Value};

use crate::context::CliContext;

use super::resources::{DefaultAdditionalTextCommands, ListArgs};
use super::utils::{parse_query_pairs, read_json_file};

pub(super) fn first_bank_transaction_explanation_id(transaction: &Value) -> Option<String> {
    if let Some(single) = transaction
        .get("bank_transaction_explanation")
        .and_then(infer_item_identifier)
    {
        return Some(single);
    }

    transaction
        .get("bank_transaction_explanations")
        .and_then(Value::as_array)
        .and_then(|items| items.iter().find_map(infer_item_identifier))
}

pub(super) fn attachment_payload_from_path(path: &Path) -> Result<Value> {
    const MAX_ATTACHMENT_SIZE_BYTES: u64 = 5 * 1024 * 1024;

    let metadata = std::fs::metadata(path).map_err(|e| ChoSdkError::Config {
        message: format!("Failed reading attachment metadata {}: {e}", path.display()),
    })?;
    if metadata.len() > MAX_ATTACHMENT_SIZE_BYTES {
        return Err(ChoSdkError::Config {
            message: format!(
                "Attachment {} exceeds FreeAgent 5MB limit ({} bytes)",
                path.display(),
                metadata.len()
            ),
        });
    }

    let bytes = std::fs::read(path).map_err(|e| ChoSdkError::Config {
        message: format!("Failed reading attachment file {}: {e}", path.display()),
    })?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ChoSdkError::Config {
            message: format!(
                "Attachment path '{}' has no valid file name",
                path.display()
            ),
        })?;

    let content_type = match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "pdf" => "application/x-pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "csv" => "text/csv",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    };

    Ok(serde_json::json!({
        "content_src": BASE64_STANDARD.encode(bytes),
        "file_name": file_name,
        "content_type": content_type
    }))
}

pub(super) fn read_optional_json_file(file: &Option<PathBuf>) -> Result<Value> {
    match file {
        Some(path) => read_json_file(path),
        None => Ok(Value::Object(Map::new())),
    }
}

pub(super) async fn run_default_additional_text(
    api_path: &str,
    command: &DefaultAdditionalTextCommands,
    tool_prefix: &str,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    let path = format!("{api_path}/default_additional_text");
    match command {
        DefaultAdditionalTextCommands::Get => {
            let value = ctx.client().get_json(&path, &[]).await?;
            ctx.emit_success(
                &format!("{tool_prefix}.default-additional-text.get"),
                &value,
                start,
            )
        }
        DefaultAdditionalTextCommands::Update { file } => {
            ctx.require_writes_allowed()?;
            let payload = read_json_file(file)?;
            ctx.log_input(
                &format!("{tool_prefix}.default-additional-text.update"),
                &payload,
            );
            let value = ctx.client().put_json(&path, &payload, true).await?;
            ctx.emit_success(
                &format!("{tool_prefix}.default-additional-text.update"),
                &value,
                start,
            )
        }
        DefaultAdditionalTextCommands::Delete => {
            ctx.require_writes_allowed()?;
            let value = ctx.client().delete_json(&path, true).await?;
            ctx.emit_success(
                &format!("{tool_prefix}.default-additional-text.delete"),
                &value,
                start,
            )
        }
    }
}

pub(super) async fn fetch_pdf_resource(
    api_path: &str,
    id: &str,
    output: Option<&Path>,
    tool: &str,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    let path = format!("{}/{}/pdf", api_path, encode_path_segment(id));
    let value = ctx.client().get_json(&path, &[]).await?;
    let pdf = value.get("pdf").cloned().unwrap_or(value);
    let encoded = pdf
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| ChoSdkError::Parse {
            message: format!("Expected '{tool}' response to contain pdf.content"),
        })?;
    let bytes = BASE64_STANDARD
        .decode(encoded)
        .map_err(|e| ChoSdkError::Parse {
            message: format!("Invalid base64 in {tool} response: {e}"),
        })?;

    if let Some(path) = output {
        std::fs::write(path, &bytes).map_err(|e| ChoSdkError::Config {
            message: format!("Failed writing PDF output {}: {e}", path.display()),
        })?;
        let payload = serde_json::json!({
            "id": id,
            "bytes": bytes.len(),
            "output": path.display().to_string(),
            "file_name": pdf.get("file_name"),
            "content_type": pdf.get("content_type"),
        });
        return ctx.emit_success(tool, &payload, start);
    }

    let payload = serde_json::json!({
        "id": id,
        "bytes": bytes.len(),
        "pdf": pdf,
    });
    ctx.emit_success(tool, &payload, start)
}

pub(super) fn list_query(args: &ListArgs) -> Result<Vec<(String, String)>> {
    let mut query = parse_query_pairs(&args.query)?;

    push_if_some(&mut query, "view", args.view.as_ref());
    push_if_some(&mut query, "sort", args.sort.as_ref());
    push_if_some(&mut query, "from_date", args.from_date.as_ref());
    push_if_some(&mut query, "to_date", args.to_date.as_ref());
    push_if_some(&mut query, "updated_since", args.updated_since.as_ref());
    push_if_some(&mut query, "contact", args.contact.as_ref());
    push_if_some(&mut query, "project", args.project.as_ref());
    push_if_some(&mut query, "bank_account", args.bank_account.as_ref());
    push_if_some(&mut query, "user", args.user.as_ref());

    Ok(query)
}

fn push_if_some(query: &mut Vec<(String, String)>, key: &str, value: Option<&String>) {
    if let Some(value) = value
        && !value.trim().is_empty()
    {
        query.push((key.to_string(), value.to_string()));
    }
}

pub(super) fn encode_path_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

pub(super) fn infer_item_identifier(value: &Value) -> Option<String> {
    if let Some(url) = value.get("url").and_then(Value::as_str) {
        return Some(url.to_string());
    }

    if let Some(id) = value.get("id").and_then(Value::as_str) {
        return Some(id.to_string());
    }

    value
        .get("id")
        .and_then(Value::as_i64)
        .map(|id| id.to_string())
}

pub(super) fn bank_account_display_name(item: &Value) -> String {
    if let Some(name) = item.get("name").and_then(Value::as_str)
        && !name.trim().is_empty()
    {
        return name.to_string();
    }

    let bank_name = item
        .get("bank_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let account_number = item
        .get("account_number")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if bank_name.is_empty() && account_number.is_empty() {
        "Bank Account".to_string()
    } else if account_number.is_empty() {
        bank_name.to_string()
    } else if bank_name.is_empty() {
        account_number.to_string()
    } else {
        format!("{bank_name} ({account_number})")
    }
}

pub(super) fn annotate_bank_account_fields(
    item: &mut Value,
    bank_account_url: &str,
    bank_account_name: &str,
) {
    let Value::Object(map) = item else {
        return;
    };

    map.entry("_bank_account_url".to_string())
        .or_insert_with(|| Value::String(bank_account_url.to_string()));
    map.entry("_bank_account_name".to_string())
        .or_insert_with(|| Value::String(bank_account_name.to_string()));
}

pub(super) fn sort_items_by_latest_date(items: &mut [Value]) {
    let Some(date_key) = infer_date_key(items) else {
        return;
    };

    items.sort_by(|left, right| {
        compare_date_values(
            left.get(date_key).and_then(parse_date_value),
            right.get(date_key).and_then(parse_date_value),
        )
    });
}

fn infer_date_key(items: &[Value]) -> Option<&'static str> {
    const DATE_KEYS: &[&str] = &[
        "dated_on",
        "date",
        "created_at",
        "updated_at",
        "period_ends_on",
        "period_end",
        "starts_on",
        "ends_on",
        "due_on",
        "paid_on",
        "submitted_on",
        "filed_on",
        "payment_date",
        "statement_date",
    ];

    for key in DATE_KEYS {
        let count = items
            .iter()
            .filter_map(|item| item.get(*key).and_then(parse_date_value))
            .take(2)
            .count();
        if count >= 2 {
            return Some(*key);
        }
    }
    None
}

fn compare_date_values(left: Option<i64>, right: Option<i64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => right.cmp(&left),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn parse_date_value(value: &Value) -> Option<i64> {
    match value {
        Value::String(text) => parse_date_text(text),
        Value::Number(number) => number.as_i64(),
        _ => None,
    }
}

fn parse_date_text(text: &str) -> Option<i64> {
    if let Ok(datetime) = DateTime::parse_from_rfc3339(text) {
        return Some(datetime.timestamp());
    }

    if let Ok(date) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        return date
            .and_hms_opt(0, 0, 0)
            .map(|datetime| datetime.and_utc().timestamp());
    }

    None
}

pub(super) fn has_bank_account_filter(args: &ListArgs) -> bool {
    if args
        .bank_account
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        return true;
    }

    args.query.iter().any(|entry| {
        entry
            .split_once('=')
            .is_some_and(|(key, value)| key == "bank_account" && !value.trim().is_empty())
    })
}

pub(super) fn flatten_category_groups(value: &serde_json::Value) -> Vec<serde_json::Value> {
    let Some(object) = value.as_object() else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for (group_name, group_value) in object {
        if let Some(items) = group_value.as_array() {
            for item in items {
                let mut item_value = item.clone();
                if let serde_json::Value::Object(map) = &mut item_value
                    && !map.contains_key("category_group")
                {
                    map.insert(
                        "category_group".to_string(),
                        serde_json::Value::String(group_name.clone()),
                    );
                }
                out.push(item_value);
            }
        } else if group_value.is_object() {
            let mut item_value = group_value.clone();
            if let serde_json::Value::Object(map) = &mut item_value
                && !map.contains_key("category_group")
            {
                map.insert(
                    "category_group".to_string(),
                    serde_json::Value::String(group_name.clone()),
                );
            }
            out.push(item_value);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn has_bank_account_filter_detects_direct_flag() {
        let args = ListArgs {
            view: None,
            sort: None,
            from_date: None,
            to_date: None,
            updated_since: None,
            contact: None,
            project: None,
            bank_account: Some("https://api.freeagent.com/v2/bank_accounts/1".to_string()),
            user: None,
            per_page: None,
            query: vec![],
        };

        assert!(has_bank_account_filter(&args));
    }

    #[test]
    fn has_bank_account_filter_detects_query_pair() {
        let args = ListArgs {
            view: None,
            sort: None,
            from_date: None,
            to_date: None,
            updated_since: None,
            contact: None,
            project: None,
            bank_account: None,
            user: None,
            per_page: None,
            query: vec!["bank_account=https://api.freeagent.com/v2/bank_accounts/1".to_string()],
        };

        assert!(has_bank_account_filter(&args));
    }

    #[test]
    fn flatten_category_groups_flattens_array_groups() {
        let value = serde_json::json!({
            "general_categories": [
                {
                    "nominal_code": "051",
                    "description": "Interest Received"
                }
            ]
        });

        let items = flatten_category_groups(&value);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["nominal_code"], "051");
        assert_eq!(items[0]["category_group"], "general_categories");
    }

    #[test]
    fn flatten_category_groups_flattens_single_object_groups() {
        let value = serde_json::json!({
            "general_categories": {
                "nominal_code": "051",
                "description": "Interest Received"
            }
        });

        let items = flatten_category_groups(&value);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["nominal_code"], "051");
        assert_eq!(items[0]["category_group"], "general_categories");
    }

    #[test]
    fn first_bank_transaction_explanation_id_reads_array_entries() {
        let transaction = serde_json::json!({
            "bank_transaction_explanations": [
                { "url": "exp-42" }
            ]
        });

        assert_eq!(
            first_bank_transaction_explanation_id(&transaction).as_deref(),
            Some("exp-42")
        );
    }

    #[test]
    fn attachment_payload_from_path_encodes_pdf() {
        let dir = tempdir().expect("temp dir");
        let pdf_path = dir.path().join("receipt.pdf");
        std::fs::write(&pdf_path, b"%PDF-1.4 mock").expect("fixture write");
        let expected_name = pdf_path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("file name")
            .to_string();
        let payload = attachment_payload_from_path(&pdf_path).expect("payload");

        assert_eq!(payload["file_name"], expected_name);
        assert_eq!(payload["content_type"], "application/x-pdf");
        assert!(payload["content_src"].as_str().is_some());
    }

    #[test]
    fn sort_items_by_latest_date_orders_descending() {
        let mut items = vec![
            serde_json::json!({ "dated_on": "2026-02-01", "url": "a" }),
            serde_json::json!({ "dated_on": "2026-03-01", "url": "b" }),
            serde_json::json!({ "dated_on": "2026-01-01", "url": "c" }),
        ];

        sort_items_by_latest_date(&mut items);

        assert_eq!(items[0]["url"], "b");
        assert_eq!(items[1]["url"], "a");
        assert_eq!(items[2]["url"], "c");
    }
}
