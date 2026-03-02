//! Generic resource command handlers.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::Instant;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use cho_sdk::api::specs::{ResourceSpec, by_name};
use cho_sdk::error::{ChoSdkError, Result};
use cho_sdk::models::{ListResult, Pagination};
use chrono::{DateTime, NaiveDate};
use clap::{Args, Subcommand};
use serde_json::{Map, Value};

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

/// Read-only resource subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum ReadOnlyResourceCommands {
    /// List resource items.
    List(Box<ListArgs>),
    /// Get one resource item.
    Get {
        /// Identifier/path key.
        id: String,
    },
}

/// Get/delete resource subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum GetDeleteResourceCommands {
    /// Get one resource item.
    Get {
        /// Identifier/path key.
        id: String,
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
    /// List bank transactions marked for approval/review.
    ForApproval(Box<ListArgs>),
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
    /// Update explanation fields for a bank transaction.
    UpdateExplanation {
        /// Bank transaction id or url.
        transaction: String,
        /// Set a clean description (for example "Expense: MyMind Subscription").
        #[arg(long)]
        description: Option<String>,
        /// Mark or unmark review state on the explanation.
        #[arg(long)]
        mark_for_review: Option<bool>,
        /// Optional local attachment path (PDF/image); encoded automatically.
        #[arg(long)]
        attachment: Option<PathBuf>,
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

/// Returns tool name for read-only resource command.
pub fn tool_name_read_only(resource: &str, command: &ReadOnlyResourceCommands) -> String {
    let action = match command {
        ReadOnlyResourceCommands::List(_) => "list",
        ReadOnlyResourceCommands::Get { .. } => "get",
    };

    format!("{resource}.{action}")
}

/// Returns tool name for get/delete resource command.
pub fn tool_name_get_delete(resource: &str, command: &GetDeleteResourceCommands) -> String {
    let action = match command {
        GetDeleteResourceCommands::Get { .. } => "get",
        GetDeleteResourceCommands::Delete { .. } => "delete",
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
        return list_bank_resource_across_accounts(resource, args, ctx, start).await;
    }

    let spec = by_name(resource).ok_or_else(|| ChoSdkError::Config {
        message: format!("Unsupported resource '{resource}'"),
    })?;

    run_resource_with_spec(spec, command, ctx, start).await
}

/// Executes read-only resource command.
pub async fn run_read_only_resource(
    resource: &str,
    command: &ReadOnlyResourceCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        ReadOnlyResourceCommands::List(args) => {
            run_resource(
                resource,
                &ResourceCommands::List((**args).clone()),
                ctx,
                start,
            )
            .await
        }
        ReadOnlyResourceCommands::Get { id } => {
            run_resource(
                resource,
                &ResourceCommands::Get { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
    }
}

/// Executes get/delete resource command.
pub async fn run_get_delete_resource(
    resource: &str,
    command: &GetDeleteResourceCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        GetDeleteResourceCommands::Get { id } => {
            run_resource(
                resource,
                &ResourceCommands::Get { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        GetDeleteResourceCommands::Delete { id } => {
            run_resource(
                resource,
                &ResourceCommands::Delete { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
    }
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
        BankTransactionCommands::List(args) => run_bank_transactions_list(args, ctx, start).await,
        BankTransactionCommands::ForApproval(args) => {
            let mut list_args = (**args).clone();
            list_args.view = Some("marked_for_review".to_string());
            run_bank_transactions_list(&list_args, ctx, start).await
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
        BankTransactionCommands::UpdateExplanation {
            transaction,
            description,
            mark_for_review,
            attachment,
        } => {
            update_bank_transaction_explanation(
                transaction,
                description.as_deref(),
                *mark_for_review,
                attachment.as_deref(),
                ctx,
                start,
            )
            .await
        }
    }
}

async fn run_bank_transactions_list(
    args: &ListArgs,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    if has_bank_account_filter(args) {
        return run_resource(
            "bank-transactions",
            &ResourceCommands::List(args.clone()),
            ctx,
            start,
        )
        .await;
    }

    list_bank_resource_across_accounts("bank-transactions", args, ctx, start).await
}

async fn update_bank_transaction_explanation(
    transaction: &str,
    description: Option<&str>,
    mark_for_review: Option<bool>,
    attachment: Option<&Path>,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    ctx.require_writes_allowed()?;

    if description.is_none() && mark_for_review.is_none() && attachment.is_none() {
        return Err(ChoSdkError::Config {
            message:
                "No changes requested. Provide at least one of --description, --mark-for-review, or --attachment"
                    .to_string(),
        });
    }

    let transaction_spec = by_name("bank-transactions").ok_or_else(|| ChoSdkError::Config {
        message: "Missing bank-transactions resource spec".to_string(),
    })?;
    let transaction_value = ctx
        .client()
        .resource(transaction_spec)
        .get(transaction)
        .await?;
    let explanation_id =
        first_bank_transaction_explanation_id(&transaction_value).ok_or_else(|| {
            ChoSdkError::Config {
                message: "Selected transaction has no explanation yet. Create one first via bank-transaction-explanations create --file <path>"
                    .to_string(),
            }
        })?;

    let mut patch = Map::new();
    if let Some(description) = description.filter(|value| !value.trim().is_empty()) {
        patch.insert(
            "description".to_string(),
            Value::String(description.to_string()),
        );
    }
    if let Some(mark_for_review) = mark_for_review {
        patch.insert(
            "marked_for_review".to_string(),
            Value::Bool(mark_for_review),
        );
    }
    if let Some(attachment_path) = attachment {
        patch.insert(
            "attachment".to_string(),
            attachment_payload_from_path(attachment_path)?,
        );
    }

    if patch.is_empty() {
        return Err(ChoSdkError::Config {
            message: "No non-empty explanation updates were provided".to_string(),
        });
    }

    let audit_payload = serde_json::json!({
        "transaction": transaction,
        "explanation": explanation_id,
        "description": description.unwrap_or_default(),
        "mark_for_review": mark_for_review,
        "attachment": attachment.map(|path| path.display().to_string()),
    });
    ctx.log_input("bank-transactions.update-explanation", &audit_payload);

    let explanations_spec =
        by_name("bank-transaction-explanations").ok_or_else(|| ChoSdkError::Config {
            message: "Missing bank-transaction-explanations resource spec".to_string(),
        })?;
    let value = ctx
        .client()
        .resource(explanations_spec)
        .update(&explanation_id, &Value::Object(patch))
        .await?;

    ctx.emit_success("bank-transactions.update-explanation", &value, start)
}

/// Returns tool name for bank transaction command.
pub fn bank_transactions_tool_name(command: &BankTransactionCommands) -> String {
    match command {
        BankTransactionCommands::List(_) => "bank-transactions.list".to_string(),
        BankTransactionCommands::ForApproval(_) => "bank-transactions.for-approval".to_string(),
        BankTransactionCommands::Get { .. } => "bank-transactions.get".to_string(),
        BankTransactionCommands::UploadStatement { .. } => {
            "bank-transactions.upload-statement".to_string()
        }
        BankTransactionCommands::UpdateExplanation { .. } => {
            "bank-transactions.update-explanation".to_string()
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

async fn list_bank_resource_across_accounts(
    resource: &str,
    list_args: &ListArgs,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    let spec = by_name(resource).ok_or_else(|| ChoSdkError::Config {
        message: format!("Unsupported resource '{resource}'"),
    })?;
    let bank_accounts_spec = by_name("bank-accounts").ok_or_else(|| ChoSdkError::Config {
        message: "Missing bank-accounts resource spec".to_string(),
    })?;

    let bank_accounts = ctx
        .client()
        .resource(bank_accounts_spec)
        .list(&[], Pagination::all())
        .await?;

    let mut query_base = list_query(list_args)?;
    query_base.retain(|(key, _)| key != "bank_account");

    let api = ctx.client().resource(spec);
    let mut combined = Vec::new();
    for account in bank_accounts.items {
        let Some(bank_account_url) = infer_item_identifier(&account) else {
            continue;
        };
        let account_name = bank_account_display_name(&account);

        let mut query = query_base.clone();
        query.push(("bank_account".to_string(), bank_account_url.clone()));
        let result = api.list(&query, Pagination::all()).await?;

        for mut item in result.items {
            annotate_bank_account_fields(&mut item, &bank_account_url, &account_name);
            combined.push(item);
        }
    }

    sort_items_by_latest_date(&mut combined);
    let total = combined.len();
    let mut has_more = false;

    let pagination = ctx.pagination();
    if !pagination.all && pagination.limit > 0 && total > pagination.limit {
        combined.truncate(pagination.limit);
        has_more = true;
    }

    let result = ListResult {
        items: combined,
        total: Some(total),
        has_more,
        page: 1,
        per_page: pagination.per_page,
    };

    ctx.emit_list(&format!("{resource}.list"), &result, start)
}

fn first_bank_transaction_explanation_id(transaction: &Value) -> Option<String> {
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

fn attachment_payload_from_path(path: &Path) -> Result<Value> {
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

fn infer_item_identifier(value: &Value) -> Option<String> {
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

fn bank_account_display_name(item: &Value) -> String {
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

fn annotate_bank_account_fields(item: &mut Value, bank_account_url: &str, bank_account_name: &str) {
    let Value::Object(map) = item else {
        return;
    };

    map.entry("_bank_account_url".to_string())
        .or_insert_with(|| Value::String(bank_account_url.to_string()));
    map.entry("_bank_account_name".to_string())
        .or_insert_with(|| Value::String(bank_account_name.to_string()));
}

fn sort_items_by_latest_date(items: &mut [Value]) {
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
