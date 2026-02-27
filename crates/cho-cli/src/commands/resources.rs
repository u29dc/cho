//! Generic resource command handlers.

use std::path::PathBuf;
use std::time::Instant;

use cho_sdk::api::specs::{ResourceSpec, by_name};
use cho_sdk::error::{ChoSdkError, Result};
use cho_sdk::models::ListResult;
use clap::{Args, Subcommand};

use crate::context::CliContext;

use super::utils::{parse_query_pairs, read_json_file};

/// Generic list args shared by list commands.
#[derive(Debug, Clone, Args)]
pub struct ListArgs {
    /// Built-in `view` filter.
    #[arg(long)]
    pub view: Option<String>,
    /// Sorting expression.
    #[arg(long)]
    pub sort: Option<String>,
    /// From-date filter (`YYYY-MM-DD`).
    #[arg(long)]
    pub from_date: Option<String>,
    /// To-date filter (`YYYY-MM-DD`).
    #[arg(long)]
    pub to_date: Option<String>,
    /// Updated-since timestamp.
    #[arg(long)]
    pub updated_since: Option<String>,
    /// Contact URL filter.
    #[arg(long)]
    pub contact: Option<String>,
    /// Project URL filter.
    #[arg(long)]
    pub project: Option<String>,
    /// Bank account URL filter.
    #[arg(long)]
    pub bank_account: Option<String>,
    /// User URL filter.
    #[arg(long)]
    pub user: Option<String>,
    /// Override page size (`1..=100`).
    #[arg(long)]
    pub per_page: Option<u32>,
    /// Additional query pairs (`key=value`), can be repeated.
    #[arg(long = "query", value_name = "KEY=VALUE")]
    pub query: Vec<String>,
}

/// Generic CRUD subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum ResourceCommands {
    /// List resource items.
    List(ListArgs),
    /// Get one resource item.
    Get {
        /// Identifier/path key.
        id: String,
    },
    /// Create resource item.
    Create {
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Update resource item.
    Update {
        /// Identifier/path key.
        id: String,
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete resource item.
    Delete {
        /// Identifier/path key.
        id: String,
    },
}

/// Contact resource commands.
#[derive(Debug, Clone, Subcommand)]
pub enum ContactCommands {
    /// List contacts.
    List(Box<ListArgs>),
    /// Get one contact.
    Get { id: String },
    /// Create contact.
    Create {
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Update contact.
    Update {
        /// Identifier/path key.
        id: String,
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete contact.
    Delete { id: String },
    /// Search contacts by term (name/email).
    Search {
        /// Case-insensitive search term.
        term: String,
        /// Optional custom per-page.
        #[arg(long)]
        per_page: Option<u32>,
    },
}

/// Invoice resource commands.
#[derive(Debug, Clone, Subcommand)]
pub enum InvoiceCommands {
    /// List invoices.
    List(Box<ListArgs>),
    /// Get one invoice.
    Get { id: String },
    /// Create invoice.
    Create {
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Update invoice.
    Update {
        /// Identifier/path key.
        id: String,
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete invoice.
    Delete { id: String },
    /// Trigger invoice status transition.
    Transition {
        /// Invoice identifier.
        id: String,
        /// Transition action.
        action: InvoiceTransition,
    },
    /// Trigger invoice email send.
    SendEmail {
        /// Invoice identifier.
        id: String,
    },
}

/// Supported invoice transitions.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum InvoiceTransition {
    /// Mark invoice as draft.
    MarkAsDraft,
    /// Mark invoice as sent.
    MarkAsSent,
    /// Mark invoice as scheduled.
    MarkAsScheduled,
    /// Mark invoice as cancelled.
    MarkAsCancelled,
    /// Convert invoice to credit note.
    ConvertToCreditNote,
}

/// Bank transaction commands.
#[derive(Debug, Clone, Subcommand)]
pub enum BankTransactionCommands {
    /// List bank transactions.
    List(Box<ListArgs>),
    /// Get one bank transaction.
    Get { id: String },
    /// Upload statement file for bank account.
    UploadStatement {
        /// Bank account URL.
        #[arg(long)]
        bank_account: String,
        /// Statement file path.
        #[arg(long)]
        file: PathBuf,
    },
}

/// Expense commands.
#[derive(Debug, Clone, Subcommand)]
pub enum ExpenseCommands {
    /// List expenses.
    List(Box<ListArgs>),
    /// Get one expense.
    Get { id: String },
    /// Create expense.
    Create {
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Update expense.
    Update {
        /// Identifier/path key.
        id: String,
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete expense.
    Delete { id: String },
    /// Get mileage settings.
    MileageSettings,
}

/// Returns tool name for generic resource command.
pub fn tool_name(resource: &str, command: &ResourceCommands) -> String {
    let action = match command {
        ResourceCommands::List(_) => "list",
        ResourceCommands::Get { .. } => "get",
        ResourceCommands::Create { .. } => "create",
        ResourceCommands::Update { .. } => "update",
        ResourceCommands::Delete { .. } => "delete",
    };

    format!("{resource}.{action}")
}

/// Executes generic resource command.
pub async fn run_resource(
    resource: &str,
    command: &ResourceCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    if resource == "categories" {
        return run_categories_resource(command, ctx, start).await;
    }
    if resource == "bank-transaction-explanations"
        && let ResourceCommands::List(args) = command
        && !has_bank_account_filter(args)
    {
        return Err(ChoSdkError::Config {
            message: "bank-transaction-explanations list requires --bank-account <url>".to_string(),
        });
    }

    let spec = by_name(resource).ok_or_else(|| ChoSdkError::Config {
        message: format!("Unsupported resource '{resource}'"),
    })?;

    run_resource_with_spec(spec, command, ctx, start).await
}

/// Executes contact command.
pub async fn run_contacts(
    command: &ContactCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        ContactCommands::List(args) => {
            run_resource(
                "contacts",
                &ResourceCommands::List((**args).clone()),
                ctx,
                start,
            )
            .await
        }
        ContactCommands::Get { id } => {
            run_resource(
                "contacts",
                &ResourceCommands::Get { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        ContactCommands::Create { file } => {
            run_resource(
                "contacts",
                &ResourceCommands::Create { file: file.clone() },
                ctx,
                start,
            )
            .await
        }
        ContactCommands::Update { id, file } => {
            run_resource(
                "contacts",
                &ResourceCommands::Update {
                    id: id.clone(),
                    file: file.clone(),
                },
                ctx,
                start,
            )
            .await
        }
        ContactCommands::Delete { id } => {
            run_resource(
                "contacts",
                &ResourceCommands::Delete { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        ContactCommands::Search { term, per_page } => {
            search_contacts(term, *per_page, ctx, start).await
        }
    }
}

/// Returns tool name for contact command.
pub fn contacts_tool_name(command: &ContactCommands) -> String {
    match command {
        ContactCommands::List(_) => "contacts.list".to_string(),
        ContactCommands::Get { .. } => "contacts.get".to_string(),
        ContactCommands::Create { .. } => "contacts.create".to_string(),
        ContactCommands::Update { .. } => "contacts.update".to_string(),
        ContactCommands::Delete { .. } => "contacts.delete".to_string(),
        ContactCommands::Search { .. } => "contacts.search".to_string(),
    }
}

/// Executes invoice command.
pub async fn run_invoices(
    command: &InvoiceCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        InvoiceCommands::List(args) => {
            run_resource(
                "invoices",
                &ResourceCommands::List((**args).clone()),
                ctx,
                start,
            )
            .await
        }
        InvoiceCommands::Get { id } => {
            run_resource(
                "invoices",
                &ResourceCommands::Get { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        InvoiceCommands::Create { file } => {
            run_resource(
                "invoices",
                &ResourceCommands::Create { file: file.clone() },
                ctx,
                start,
            )
            .await
        }
        InvoiceCommands::Update { id, file } => {
            run_resource(
                "invoices",
                &ResourceCommands::Update {
                    id: id.clone(),
                    file: file.clone(),
                },
                ctx,
                start,
            )
            .await
        }
        InvoiceCommands::Delete { id } => {
            run_resource(
                "invoices",
                &ResourceCommands::Delete { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        InvoiceCommands::Transition { id, action } => {
            ctx.require_writes_allowed()?;
            let spec = by_name("invoices").ok_or_else(|| ChoSdkError::Config {
                message: "Missing invoices resource spec".to_string(),
            })?;
            let api = ctx.client().resource(spec);
            let suffix = match action {
                InvoiceTransition::MarkAsDraft => "transitions/mark_as_draft",
                InvoiceTransition::MarkAsSent => "transitions/mark_as_sent",
                InvoiceTransition::MarkAsScheduled => "transitions/mark_as_scheduled",
                InvoiceTransition::MarkAsCancelled => "transitions/mark_as_cancelled",
                InvoiceTransition::ConvertToCreditNote => "transitions/convert_to_credit_note",
            };
            let value = api
                .action(id, reqwest::Method::PUT, suffix, None, true)
                .await?;
            ctx.emit_success("invoices.transition", &value, start)
        }
        InvoiceCommands::SendEmail { id } => {
            ctx.require_writes_allowed()?;
            let spec = by_name("invoices").ok_or_else(|| ChoSdkError::Config {
                message: "Missing invoices resource spec".to_string(),
            })?;
            let api = ctx.client().resource(spec);
            let value = api
                .action(id, reqwest::Method::POST, "send_email", None, true)
                .await?;
            ctx.emit_success("invoices.send-email", &value, start)
        }
    }
}

/// Returns tool name for invoice command.
pub fn invoices_tool_name(command: &InvoiceCommands) -> String {
    match command {
        InvoiceCommands::List(_) => "invoices.list".to_string(),
        InvoiceCommands::Get { .. } => "invoices.get".to_string(),
        InvoiceCommands::Create { .. } => "invoices.create".to_string(),
        InvoiceCommands::Update { .. } => "invoices.update".to_string(),
        InvoiceCommands::Delete { .. } => "invoices.delete".to_string(),
        InvoiceCommands::Transition { .. } => "invoices.transition".to_string(),
        InvoiceCommands::SendEmail { .. } => "invoices.send-email".to_string(),
    }
}

/// Executes bank transaction command.
pub async fn run_bank_transactions(
    command: &BankTransactionCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        BankTransactionCommands::List(args) => {
            if !has_bank_account_filter(args) {
                return Err(ChoSdkError::Config {
                    message: "bank-transactions list requires --bank-account <url>".to_string(),
                });
            }
            run_resource(
                "bank-transactions",
                &ResourceCommands::List((**args).clone()),
                ctx,
                start,
            )
            .await
        }
        BankTransactionCommands::Get { id } => {
            run_resource(
                "bank-transactions",
                &ResourceCommands::Get { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        BankTransactionCommands::UploadStatement { bank_account, file } => {
            ctx.require_writes_allowed()?;
            let content = std::fs::read_to_string(file).map_err(|e| ChoSdkError::Config {
                message: format!("Failed reading statement file {}: {e}", file.display()),
            })?;
            let statement_bytes = content.len();
            let payload = serde_json::json!({ "statement": content });
            let audit_payload = serde_json::json!({
                "bank_account": bank_account,
                "file": file.display().to_string(),
                "statement_bytes": statement_bytes,
            });
            ctx.log_input("bank-transactions.upload-statement", &audit_payload);
            let value = ctx
                .client()
                .post_json(
                    &format!(
                        "bank_transactions/statement?bank_account={}",
                        encode_path_segment(bank_account)
                    ),
                    &payload,
                    true,
                )
                .await?;
            ctx.emit_success("bank-transactions.upload-statement", &value, start)
        }
    }
}

/// Returns tool name for bank transaction command.
pub fn bank_transactions_tool_name(command: &BankTransactionCommands) -> String {
    match command {
        BankTransactionCommands::List(_) => "bank-transactions.list".to_string(),
        BankTransactionCommands::Get { .. } => "bank-transactions.get".to_string(),
        BankTransactionCommands::UploadStatement { .. } => {
            "bank-transactions.upload-statement".to_string()
        }
    }
}

/// Executes expense command.
pub async fn run_expenses(
    command: &ExpenseCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        ExpenseCommands::List(args) => {
            run_resource(
                "expenses",
                &ResourceCommands::List((**args).clone()),
                ctx,
                start,
            )
            .await
        }
        ExpenseCommands::Get { id } => {
            run_resource(
                "expenses",
                &ResourceCommands::Get { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        ExpenseCommands::Create { file } => {
            run_resource(
                "expenses",
                &ResourceCommands::Create { file: file.clone() },
                ctx,
                start,
            )
            .await
        }
        ExpenseCommands::Update { id, file } => {
            run_resource(
                "expenses",
                &ResourceCommands::Update {
                    id: id.clone(),
                    file: file.clone(),
                },
                ctx,
                start,
            )
            .await
        }
        ExpenseCommands::Delete { id } => {
            run_resource(
                "expenses",
                &ResourceCommands::Delete { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        ExpenseCommands::MileageSettings => {
            let value = ctx
                .client()
                .get_json("expenses/mileage_settings", &[])
                .await?;
            ctx.emit_success("expenses.mileage-settings", &value, start)
        }
    }
}

/// Returns tool name for expense command.
pub fn expenses_tool_name(command: &ExpenseCommands) -> String {
    match command {
        ExpenseCommands::List(_) => "expenses.list".to_string(),
        ExpenseCommands::Get { .. } => "expenses.get".to_string(),
        ExpenseCommands::Create { .. } => "expenses.create".to_string(),
        ExpenseCommands::Update { .. } => "expenses.update".to_string(),
        ExpenseCommands::Delete { .. } => "expenses.delete".to_string(),
        ExpenseCommands::MileageSettings => "expenses.mileage-settings".to_string(),
    }
}

async fn run_resource_with_spec(
    spec: ResourceSpec,
    command: &ResourceCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    let api = ctx.client().resource(spec);
    let tool_prefix = spec.name;

    match command {
        ResourceCommands::List(list_args) => {
            if !spec.capabilities.list {
                return Err(ChoSdkError::Config {
                    message: format!("Resource '{}' does not support list", spec.name),
                });
            }

            let query = list_query(list_args)?;
            let mut pagination = ctx.pagination();
            if let Some(per_page) = list_args.per_page {
                pagination.per_page = per_page.clamp(1, 100);
            }

            let result = api.list(&query, pagination).await?;
            ctx.emit_list(&format!("{}.list", tool_prefix), &result, start)
        }
        ResourceCommands::Get { id } => {
            if !spec.capabilities.get {
                return Err(ChoSdkError::Config {
                    message: format!("Resource '{}' does not support get", spec.name),
                });
            }

            let value = api.get(id).await?;
            ctx.emit_success(&format!("{}.get", tool_prefix), &value, start)
        }
        ResourceCommands::Create { file } => {
            if !spec.capabilities.create {
                return Err(ChoSdkError::Config {
                    message: format!("Resource '{}' does not support create", spec.name),
                });
            }

            ctx.require_writes_allowed()?;
            let payload = read_json_file(file)?;
            ctx.log_input(&format!("{}.create", tool_prefix), &payload);
            let value = api.create(&payload).await?;
            ctx.emit_success(&format!("{}.create", tool_prefix), &value, start)
        }
        ResourceCommands::Update { id, file } => {
            if !spec.capabilities.update {
                return Err(ChoSdkError::Config {
                    message: format!("Resource '{}' does not support update", spec.name),
                });
            }

            ctx.require_writes_allowed()?;
            let payload = read_json_file(file)?;
            ctx.log_input(&format!("{}.update", tool_prefix), &payload);
            let value = api.update(id, &payload).await?;
            ctx.emit_success(&format!("{}.update", tool_prefix), &value, start)
        }
        ResourceCommands::Delete { id } => {
            if !spec.capabilities.delete {
                return Err(ChoSdkError::Config {
                    message: format!("Resource '{}' does not support delete", spec.name),
                });
            }

            ctx.require_writes_allowed()?;
            let value = api.delete(id).await?;
            ctx.emit_success(&format!("{}.delete", tool_prefix), &value, start)
        }
    }
}

async fn run_categories_resource(
    command: &ResourceCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        ResourceCommands::List(list_args) => {
            let query = list_query(list_args)?;
            let value = ctx.client().get_json("categories", &query).await?;

            let mut items = flatten_category_groups(&value);
            let total = items.len();
            let mut has_more = false;

            let pagination = ctx.pagination();
            if !pagination.all && pagination.limit > 0 && total > pagination.limit {
                items.truncate(pagination.limit);
                has_more = true;
            }

            let result = ListResult {
                items,
                total: Some(total),
                has_more,
                page: 1,
                per_page: pagination.per_page,
            };

            ctx.emit_list("categories.list", &result, start)
        }
        ResourceCommands::Get { id } => {
            let value = ctx
                .client()
                .get_json(&format!("categories/{}", encode_path_segment(id)), &[])
                .await?;

            let items = flatten_category_groups(&value);
            if let Some(first) = items.into_iter().next() {
                ctx.emit_success("categories.get", &first, start)
            } else {
                ctx.emit_success("categories.get", &value, start)
            }
        }
        ResourceCommands::Create { .. }
        | ResourceCommands::Update { .. }
        | ResourceCommands::Delete { .. } => {
            let spec = by_name("categories").ok_or_else(|| ChoSdkError::Config {
                message: "Missing categories resource spec".to_string(),
            })?;
            run_resource_with_spec(spec, command, ctx, start).await
        }
    }
}

async fn search_contacts(
    term: &str,
    per_page: Option<u32>,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    let spec = by_name("contacts").ok_or_else(|| ChoSdkError::Config {
        message: "Missing contacts resource spec".to_string(),
    })?;

    let api = ctx.client().resource(spec);
    let mut pagination = ctx.pagination();
    if let Some(per_page) = per_page {
        pagination.per_page = per_page.clamp(1, 100);
    }
    pagination.all = true;

    let result = api.list(&[], pagination).await?;
    let lowered = term.to_ascii_lowercase();

    let mut filtered = Vec::new();
    for item in result.items {
        let haystack = [
            item.get("first_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
            item.get("last_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
            item.get("organisation_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
            item.get("email")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
            item.get("contact_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
        ]
        .join(" ")
        .to_ascii_lowercase();

        if haystack.contains(&lowered) {
            filtered.push(item);
        }
    }

    let payload = serde_json::json!({
        "matches": filtered,
        "match_count": filtered.len(),
        "search_term": term,
    });

    ctx.emit_success("contacts.search", &payload, start)
}

fn list_query(args: &ListArgs) -> Result<Vec<(String, String)>> {
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

fn encode_path_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn has_bank_account_filter(args: &ListArgs) -> bool {
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

fn flatten_category_groups(value: &serde_json::Value) -> Vec<serde_json::Value> {
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
}
