//! Generic resource command handlers.

use std::path::{Path, PathBuf};
use std::time::Instant;

use cho_sdk::api::specs::{ResourceSpec, by_name};
use cho_sdk::error::{ChoSdkError, Result};
use cho_sdk::models::{ListResult, Pagination};
use clap::{Args, Subcommand};
use serde_json::{Map, Value};

use crate::context::CliContext;

use super::resources_helpers::{
    annotate_bank_account_fields, attachment_payload_from_path, bank_account_display_name,
    encode_path_segment, first_bank_transaction_explanation_id, flatten_category_groups,
    has_bank_account_filter, infer_item_identifier, list_query, sort_items_by_latest_date,
};
pub use super::resources_sales::{
    credit_notes_tool_name, estimates_tool_name, invoices_tool_name, run_credit_notes,
    run_estimates, run_invoices,
};
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
        /// Additional query pairs (`key=value`), can be repeated.
        #[arg(long = "query", value_name = "KEY=VALUE")]
        query: Vec<String>,
    },
    /// Update resource item.
    Update {
        /// Identifier/path key.
        id: String,
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
        /// Additional query pairs (`key=value`), can be repeated.
        #[arg(long = "query", value_name = "KEY=VALUE")]
        query: Vec<String>,
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

/// List-only resource subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum ListOnlyResourceCommands {
    /// List resource items.
    List(Box<ListArgs>),
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

/// Create/update/delete resource subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum WriteOnlyResourceCommands {
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

/// Default additional text operations for sales documents.
#[derive(Debug, Clone, Subcommand)]
pub enum DefaultAdditionalTextCommands {
    /// Get default additional text.
    Get,
    /// Update default additional text using JSON payload.
    Update {
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete default additional text.
    Delete,
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
        /// Optional JSON payload file path.
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Duplicate invoice.
    Duplicate {
        /// Invoice identifier.
        id: String,
    },
    /// Request direct debit payment.
    DirectDebit {
        /// Invoice identifier.
        id: String,
        /// Optional JSON payload file path.
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Fetch invoice timeline.
    Timeline,
    /// Get invoice as PDF.
    GetPdf {
        /// Invoice identifier.
        id: String,
        /// Optional output file path for decoded PDF bytes.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Manage invoice default additional text.
    DefaultAdditionalText {
        #[command(subcommand)]
        command: DefaultAdditionalTextCommands,
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

/// Credit note resource commands.
#[derive(Debug, Clone, Subcommand)]
pub enum CreditNoteCommands {
    /// List credit notes.
    List(Box<ListArgs>),
    /// Get one credit note.
    Get { id: String },
    /// Create credit note.
    Create {
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Update credit note.
    Update {
        /// Identifier/path key.
        id: String,
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete credit note.
    Delete { id: String },
    /// Trigger credit note status transition.
    Transition {
        /// Credit note identifier.
        id: String,
        /// Transition action.
        action: CreditNoteTransition,
    },
    /// Trigger credit note email send.
    SendEmail {
        /// Credit note identifier.
        id: String,
        /// Optional JSON payload file path.
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Get credit note as PDF.
    GetPdf {
        /// Credit note identifier.
        id: String,
        /// Optional output file path for decoded PDF bytes.
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

/// Supported credit note transitions.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum CreditNoteTransition {
    /// Mark credit note as draft.
    MarkAsDraft,
    /// Mark credit note as sent.
    MarkAsSent,
}

/// Estimate resource commands.
#[derive(Debug, Clone, Subcommand)]
pub enum EstimateCommands {
    /// List estimates.
    List(Box<ListArgs>),
    /// Get one estimate.
    Get { id: String },
    /// Create estimate.
    Create {
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Update estimate.
    Update {
        /// Identifier/path key.
        id: String,
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete estimate.
    Delete { id: String },
    /// Trigger estimate status transition.
    Transition {
        /// Estimate identifier.
        id: String,
        /// Transition action.
        action: EstimateTransition,
    },
    /// Trigger estimate email send.
    SendEmail {
        /// Estimate identifier.
        id: String,
        /// Optional JSON payload file path.
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Duplicate estimate.
    Duplicate {
        /// Estimate identifier.
        id: String,
    },
    /// Get estimate as PDF.
    GetPdf {
        /// Estimate identifier.
        id: String,
        /// Optional output file path for decoded PDF bytes.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Manage estimate default additional text.
    DefaultAdditionalText {
        #[command(subcommand)]
        command: DefaultAdditionalTextCommands,
    },
}

/// Supported estimate transitions.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum EstimateTransition {
    /// Mark estimate as draft.
    MarkAsDraft,
    /// Mark estimate as sent.
    MarkAsSent,
    /// Mark estimate as approved.
    MarkAsApproved,
    /// Mark estimate as rejected.
    MarkAsRejected,
    /// Convert estimate to invoice.
    ConvertToInvoice,
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
    /// Delete one bank transaction.
    Delete { id: String },
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

/// Timeslip commands.
#[derive(Debug, Clone, Subcommand)]
pub enum TimeslipCommands {
    /// List timeslips.
    List(Box<ListArgs>),
    /// Get one timeslip.
    Get { id: String },
    /// Create timeslip.
    Create {
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Update timeslip.
    Update {
        /// Identifier/path key.
        id: String,
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete timeslip.
    Delete { id: String },
    /// Start timer for a timeslip.
    StartTimer {
        /// Timeslip identifier.
        id: String,
    },
    /// Stop timer for a timeslip.
    StopTimer {
        /// Timeslip identifier.
        id: String,
    },
}

/// User commands.
#[derive(Debug, Clone, Subcommand)]
pub enum UserCommands {
    /// List users.
    List(Box<ListArgs>),
    /// Get one user.
    Get { id: String },
    /// Create user.
    Create {
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Update user.
    Update {
        /// Identifier/path key.
        id: String,
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete user.
    Delete { id: String },
    /// Get the authenticated user.
    Me,
    /// Update the authenticated user.
    UpdateMe {
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
}

/// Journal set commands.
#[derive(Debug, Clone, Subcommand)]
pub enum JournalSetCommands {
    /// List journal sets.
    List(Box<ListArgs>),
    /// Get one journal set.
    Get { id: String },
    /// Create journal set.
    Create {
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Update journal set.
    Update {
        /// Identifier/path key.
        id: String,
        /// JSON payload file path.
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete journal set.
    Delete { id: String },
    /// Get journal set opening balances.
    OpeningBalances,
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

/// Returns tool name for list-only resource command.
pub fn tool_name_list_only(resource: &str, command: &ListOnlyResourceCommands) -> String {
    let action = match command {
        ListOnlyResourceCommands::List(_) => "list",
    };

    format!("{resource}.{action}")
}

/// Returns tool name for write-only resource command.
pub fn tool_name_write_only(resource: &str, command: &WriteOnlyResourceCommands) -> String {
    let action = match command {
        WriteOnlyResourceCommands::Create { .. } => "create",
        WriteOnlyResourceCommands::Update { .. } => "update",
        WriteOnlyResourceCommands::Delete { .. } => "delete",
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

/// Executes list-only resource command.
pub async fn run_list_only_resource(
    resource: &str,
    command: &ListOnlyResourceCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        ListOnlyResourceCommands::List(args) => {
            run_resource(
                resource,
                &ResourceCommands::List((**args).clone()),
                ctx,
                start,
            )
            .await
        }
    }
}

/// Executes create/update/delete resource command.
pub async fn run_write_only_resource(
    resource: &str,
    command: &WriteOnlyResourceCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        WriteOnlyResourceCommands::Create { file } => {
            run_resource(
                resource,
                &ResourceCommands::Create {
                    file: file.clone(),
                    query: vec![],
                },
                ctx,
                start,
            )
            .await
        }
        WriteOnlyResourceCommands::Update { id, file } => {
            run_resource(
                resource,
                &ResourceCommands::Update {
                    id: id.clone(),
                    file: file.clone(),
                    query: vec![],
                },
                ctx,
                start,
            )
            .await
        }
        WriteOnlyResourceCommands::Delete { id } => {
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
                &ResourceCommands::Create {
                    file: file.clone(),
                    query: vec![],
                },
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
                    query: vec![],
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
        BankTransactionCommands::Delete { id } => {
            ctx.require_writes_allowed()?;
            let value = ctx
                .client()
                .delete_json(
                    &format!("bank_transaction/{}", encode_path_segment(id)),
                    true,
                )
                .await?;
            ctx.emit_success("bank-transactions.delete", &value, start)
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
        BankTransactionCommands::Delete { .. } => "bank-transactions.delete".to_string(),
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
                &ResourceCommands::Create {
                    file: file.clone(),
                    query: vec![],
                },
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
                    query: vec![],
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

/// Executes timeslip command.
pub async fn run_timeslips(
    command: &TimeslipCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        TimeslipCommands::List(args) => {
            run_resource(
                "timeslips",
                &ResourceCommands::List((**args).clone()),
                ctx,
                start,
            )
            .await
        }
        TimeslipCommands::Get { id } => {
            run_resource(
                "timeslips",
                &ResourceCommands::Get { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        TimeslipCommands::Create { file } => {
            run_resource(
                "timeslips",
                &ResourceCommands::Create {
                    file: file.clone(),
                    query: vec![],
                },
                ctx,
                start,
            )
            .await
        }
        TimeslipCommands::Update { id, file } => {
            run_resource(
                "timeslips",
                &ResourceCommands::Update {
                    id: id.clone(),
                    file: file.clone(),
                    query: vec![],
                },
                ctx,
                start,
            )
            .await
        }
        TimeslipCommands::Delete { id } => {
            run_resource(
                "timeslips",
                &ResourceCommands::Delete { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        TimeslipCommands::StartTimer { id } => {
            ctx.require_writes_allowed()?;
            let value = ctx
                .client()
                .post_json(
                    &format!("timeslips/{}/timer", encode_path_segment(id)),
                    &serde_json::json!({}),
                    true,
                )
                .await?;
            ctx.emit_success("timeslips.start-timer", &value, start)
        }
        TimeslipCommands::StopTimer { id } => {
            ctx.require_writes_allowed()?;
            let value = ctx
                .client()
                .delete_json(
                    &format!("timeslips/{}/timer", encode_path_segment(id)),
                    true,
                )
                .await?;
            ctx.emit_success("timeslips.stop-timer", &value, start)
        }
    }
}

/// Returns tool name for timeslip command.
pub fn timeslips_tool_name(command: &TimeslipCommands) -> String {
    match command {
        TimeslipCommands::List(_) => "timeslips.list".to_string(),
        TimeslipCommands::Get { .. } => "timeslips.get".to_string(),
        TimeslipCommands::Create { .. } => "timeslips.create".to_string(),
        TimeslipCommands::Update { .. } => "timeslips.update".to_string(),
        TimeslipCommands::Delete { .. } => "timeslips.delete".to_string(),
        TimeslipCommands::StartTimer { .. } => "timeslips.start-timer".to_string(),
        TimeslipCommands::StopTimer { .. } => "timeslips.stop-timer".to_string(),
    }
}

/// Executes user command.
pub async fn run_users(command: &UserCommands, ctx: &CliContext, start: Instant) -> Result<()> {
    match command {
        UserCommands::List(args) => {
            run_resource(
                "users",
                &ResourceCommands::List((**args).clone()),
                ctx,
                start,
            )
            .await
        }
        UserCommands::Get { id } => {
            run_resource(
                "users",
                &ResourceCommands::Get { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        UserCommands::Create { file } => {
            run_resource(
                "users",
                &ResourceCommands::Create {
                    file: file.clone(),
                    query: vec![],
                },
                ctx,
                start,
            )
            .await
        }
        UserCommands::Update { id, file } => {
            run_resource(
                "users",
                &ResourceCommands::Update {
                    id: id.clone(),
                    file: file.clone(),
                    query: vec![],
                },
                ctx,
                start,
            )
            .await
        }
        UserCommands::Delete { id } => {
            run_resource(
                "users",
                &ResourceCommands::Delete { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        UserCommands::Me => {
            let value = ctx.client().get_json("users/me", &[]).await?;
            ctx.emit_success("users.me", &value, start)
        }
        UserCommands::UpdateMe { file } => {
            ctx.require_writes_allowed()?;
            let payload = read_json_file(file)?;
            ctx.log_input("users.update-me", &payload);
            let value = ctx.client().put_json("users/me", &payload, true).await?;
            ctx.emit_success("users.update-me", &value, start)
        }
    }
}

/// Returns tool name for user command.
pub fn users_tool_name(command: &UserCommands) -> String {
    match command {
        UserCommands::List(_) => "users.list".to_string(),
        UserCommands::Get { .. } => "users.get".to_string(),
        UserCommands::Create { .. } => "users.create".to_string(),
        UserCommands::Update { .. } => "users.update".to_string(),
        UserCommands::Delete { .. } => "users.delete".to_string(),
        UserCommands::Me => "users.me".to_string(),
        UserCommands::UpdateMe { .. } => "users.update-me".to_string(),
    }
}

/// Executes journal set command.
pub async fn run_journal_sets(
    command: &JournalSetCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        JournalSetCommands::List(args) => {
            run_resource(
                "journal-sets",
                &ResourceCommands::List((**args).clone()),
                ctx,
                start,
            )
            .await
        }
        JournalSetCommands::Get { id } => {
            run_resource(
                "journal-sets",
                &ResourceCommands::Get { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        JournalSetCommands::Create { file } => {
            run_resource(
                "journal-sets",
                &ResourceCommands::Create {
                    file: file.clone(),
                    query: vec![],
                },
                ctx,
                start,
            )
            .await
        }
        JournalSetCommands::Update { id, file } => {
            run_resource(
                "journal-sets",
                &ResourceCommands::Update {
                    id: id.clone(),
                    file: file.clone(),
                    query: vec![],
                },
                ctx,
                start,
            )
            .await
        }
        JournalSetCommands::Delete { id } => {
            run_resource(
                "journal-sets",
                &ResourceCommands::Delete { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        JournalSetCommands::OpeningBalances => {
            let value = ctx
                .client()
                .get_json("journal_sets/opening_balances", &[])
                .await?;
            ctx.emit_success("journal-sets.opening-balances", &value, start)
        }
    }
}

/// Returns tool name for journal set command.
pub fn journal_sets_tool_name(command: &JournalSetCommands) -> String {
    match command {
        JournalSetCommands::List(_) => "journal-sets.list".to_string(),
        JournalSetCommands::Get { .. } => "journal-sets.get".to_string(),
        JournalSetCommands::Create { .. } => "journal-sets.create".to_string(),
        JournalSetCommands::Update { .. } => "journal-sets.update".to_string(),
        JournalSetCommands::Delete { .. } => "journal-sets.delete".to_string(),
        JournalSetCommands::OpeningBalances => "journal-sets.opening-balances".to_string(),
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
        ResourceCommands::Create { file, query } => {
            if !spec.capabilities.create {
                return Err(ChoSdkError::Config {
                    message: format!("Resource '{}' does not support create", spec.name),
                });
            }

            ctx.require_writes_allowed()?;
            let payload = read_json_file(file)?;
            ctx.log_input(&format!("{}.create", tool_prefix), &payload);
            let query_pairs = parse_query_pairs(query)?;
            let value = if query_pairs.is_empty() {
                api.create(&payload).await?
            } else {
                let wrapped = normalize_resource_payload(&payload, spec.singular_key);
                let path = path_with_query(spec.path, &query_pairs);
                let response = ctx.client().post_json(&path, &wrapped, true).await?;
                unwrap_resource_response(&response, spec.singular_key, spec.collection_key)?
            };
            ctx.emit_success(&format!("{}.create", tool_prefix), &value, start)
        }
        ResourceCommands::Update { id, file, query } => {
            if !spec.capabilities.update {
                return Err(ChoSdkError::Config {
                    message: format!("Resource '{}' does not support update", spec.name),
                });
            }

            ctx.require_writes_allowed()?;
            let payload = read_json_file(file)?;
            ctx.log_input(&format!("{}.update", tool_prefix), &payload);
            let query_pairs = parse_query_pairs(query)?;
            let value = if query_pairs.is_empty() {
                api.update(id, &payload).await?
            } else {
                let wrapped = normalize_resource_payload(&payload, spec.singular_key);
                let target_path = resource_target_path(spec.path, id);
                let path = path_with_query(&target_path, &query_pairs);
                let response = ctx.client().put_json(&path, &wrapped, true).await?;
                unwrap_resource_response(&response, spec.singular_key, spec.collection_key)?
            };
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

fn normalize_resource_payload(body: &Value, singular_key: &str) -> Value {
    if let Value::Object(map) = body
        && map.contains_key(singular_key)
    {
        return body.clone();
    }

    serde_json::json!({
        singular_key: body,
    })
}

fn unwrap_resource_response(
    response: &Value,
    singular_key: &str,
    collection_key: &str,
) -> Result<Value> {
    if let Some(value) = response.get(singular_key) {
        return Ok(value.clone());
    }

    if let Some(array) = response.get(collection_key).and_then(Value::as_array)
        && let Some(first) = array.first()
    {
        return Ok(first.clone());
    }

    if response.is_object() {
        return Ok(response.clone());
    }

    Err(ChoSdkError::Parse {
        message: format!(
            "Response did not contain expected keys '{singular_key}' or '{collection_key}'"
        ),
    })
}

fn resource_target_path(resource_path: &str, id: &str) -> String {
    let trimmed = id.trim();
    if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
        return trimmed.trim_end_matches('/').to_string();
    }

    format!(
        "{}/{}",
        resource_path.trim_end_matches('/'),
        encode_path_segment(trimmed)
    )
}

fn path_with_query(path: &str, query: &[(String, String)]) -> String {
    if query.is_empty() {
        return path.to_string();
    }

    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in query {
        serializer.append_pair(key, value);
    }
    let encoded = serializer.finish();

    if path.contains('?') {
        format!("{path}&{encoded}")
    } else {
        format!("{path}?{encoded}")
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
