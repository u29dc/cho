//! Bank transaction commands: list, get, create, update.

use std::path::PathBuf;
use std::time::Instant;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;
use cho_sdk::models::bank_transaction::BankTransaction;

use crate::commands::utils::validate_date;
use crate::context::{CliContext, warn_if_suspicious_filter};

/// Transaction subcommands.
#[derive(Debug, Subcommand)]
pub enum TransactionCommands {
    /// List bank transactions.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,

        /// Filter from date (YYYY-MM-DD).
        #[arg(long)]
        from: Option<String>,

        /// Filter to date (YYYY-MM-DD).
        #[arg(long)]
        to: Option<String>,
    },
    /// Get a single bank transaction by ID.
    Get {
        /// Bank transaction ID (UUID).
        id: Uuid,
    },
    /// Create a new bank transaction from a JSON file.
    Create {
        /// Path to JSON file containing the bank transaction data.
        #[arg(long)]
        file: PathBuf,
        /// Idempotency key for safe retries.
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    /// Update an existing bank transaction from a JSON file.
    Update {
        /// Bank transaction ID (UUID) to update.
        id: Uuid,
        /// Path to JSON file containing the bank transaction update data.
        #[arg(long)]
        file: PathBuf,
        /// Idempotency key for safe retries.
        #[arg(long)]
        idempotency_key: Option<String>,
    },
}

/// Returns the tool name for a transaction subcommand.
pub fn tool_name(cmd: &TransactionCommands) -> &'static str {
    match cmd {
        TransactionCommands::List { .. } => "transactions.list",
        TransactionCommands::Get { .. } => "transactions.get",
        TransactionCommands::Create { .. } => "transactions.create",
        TransactionCommands::Update { .. } => "transactions.update",
    }
}

/// Runs a transaction subcommand.
pub async fn run(
    cmd: &TransactionCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        TransactionCommands::List { r#where, from, to } => {
            warn_if_suspicious_filter(r#where.as_ref());
            let mut params = ListParams::new();
            // Validate date formats before OData interpolation
            if let Some(d) = from {
                validate_date(d, "--from")?;
            }
            if let Some(d) = to {
                validate_date(d, "--to")?;
            }

            let mut where_parts = Vec::new();
            if let Some(w) = r#where {
                where_parts.push(w.clone());
            }
            if let Some(from_date) = from {
                where_parts.push(format!("Date >= DateTime({from_date})"));
            }
            if let Some(to_date) = to {
                where_parts.push(format!("Date <= DateTime({to_date})"));
            }
            if !where_parts.is_empty() {
                params = params.with_where(where_parts.join(" AND "));
            }

            let pagination = ctx.pagination_params();
            let txns = ctx
                .client()
                .bank_transactions()
                .list(&params, &pagination)
                .await?;
            ctx.emit_list("transactions.list", &txns, start)?;
            Ok(())
        }
        TransactionCommands::Get { id } => {
            let txn = ctx.client().bank_transactions().get(*id).await?;
            ctx.emit_success("transactions.get", &txn, start)?;
            Ok(())
        }
        TransactionCommands::Create {
            file,
            idempotency_key,
        } => {
            ctx.require_writes_allowed()?;
            let txn: BankTransaction = crate::commands::utils::read_json_file(file)?;
            let result = ctx
                .client()
                .bank_transactions()
                .create(&txn, idempotency_key.as_deref())
                .await?;
            ctx.emit_success("transactions.create", &result, start)?;
            Ok(())
        }
        TransactionCommands::Update {
            id,
            file,
            idempotency_key,
        } => {
            ctx.require_writes_allowed()?;
            let txn: BankTransaction = crate::commands::utils::read_json_file(file)?;
            let result = ctx
                .client()
                .bank_transactions()
                .update(*id, &txn, idempotency_key.as_deref())
                .await?;
            ctx.emit_success("transactions.update", &result, start)?;
            Ok(())
        }
    }
}
