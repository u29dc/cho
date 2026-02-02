//! Invoice commands: list, get, create, update.

use std::path::PathBuf;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;
use cho_sdk::models::invoice::Invoice;

use crate::context::CliContext;

/// Invoice subcommands.
#[derive(Debug, Subcommand)]
pub enum InvoiceCommands {
    /// List invoices.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,

        /// Order expression (e.g. "Date DESC").
        #[arg(long)]
        order: Option<String>,

        /// Filter from date (YYYY-MM-DD, adds where DateFrom>=DATE).
        #[arg(long)]
        from: Option<String>,

        /// Filter to date (YYYY-MM-DD, adds where DateTo<=DATE).
        #[arg(long)]
        to: Option<String>,

        /// Return summary only (fewer fields).
        #[arg(long)]
        summary: bool,
    },
    /// Get a single invoice by ID or invoice number.
    Get {
        /// Invoice ID (UUID) or invoice number.
        id_or_number: String,
    },
    /// Create a new invoice from a JSON file.
    Create {
        /// Path to JSON file containing the invoice data.
        #[arg(long)]
        file: PathBuf,
        /// Idempotency key for safe retries.
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    /// Update an existing invoice from a JSON file.
    Update {
        /// Invoice ID (UUID) to update.
        id: Uuid,
        /// Path to JSON file containing the invoice update data.
        #[arg(long)]
        file: PathBuf,
        /// Idempotency key for safe retries.
        #[arg(long)]
        idempotency_key: Option<String>,
    },
}

/// Runs an invoice subcommand.
pub async fn run(cmd: &InvoiceCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        InvoiceCommands::List {
            r#where,
            order,
            from,
            to,
            summary,
        } => {
            let mut params = ListParams::new();

            // Build where filter combining explicit where + date range
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

            if let Some(o) = order {
                params = params.with_order(o.clone());
            }
            if *summary {
                params = params.with_summary_only(true);
            }

            let pagination = ctx.pagination_params();
            let invoices = ctx.client().invoices().list(&params, &pagination).await?;
            let output = ctx.format_paginated_output(&invoices)?;
            println!("{output}");
            Ok(())
        }
        InvoiceCommands::Get { id_or_number } => {
            let invoice = if let Ok(uuid) = id_or_number.parse::<Uuid>() {
                ctx.client().invoices().get(uuid).await?
            } else {
                ctx.client().invoices().get_by_number(id_or_number).await?
            };
            let output = ctx.format_output(&invoice)?;
            println!("{output}");
            Ok(())
        }
        InvoiceCommands::Create {
            file,
            idempotency_key,
        } => {
            ctx.require_writes_allowed()?;
            let invoice: Invoice = read_json_file(file)?;
            let result = ctx
                .client()
                .invoices()
                .create(&invoice, idempotency_key.as_deref())
                .await?;
            let output = ctx.format_output(&result)?;
            println!("{output}");
            Ok(())
        }
        InvoiceCommands::Update {
            id,
            file,
            idempotency_key,
        } => {
            ctx.require_writes_allowed()?;
            let invoice: Invoice = read_json_file(file)?;
            let result = ctx
                .client()
                .invoices()
                .update(*id, &invoice, idempotency_key.as_deref())
                .await?;
            let output = ctx.format_output(&result)?;
            println!("{output}");
            Ok(())
        }
    }
}

/// Reads and parses a JSON file into the specified type.
pub fn read_json_file<T: serde::de::DeserializeOwned>(
    path: &std::path::Path,
) -> cho_sdk::error::Result<T> {
    let content =
        std::fs::read_to_string(path).map_err(|e| cho_sdk::error::ChoSdkError::Config {
            message: format!("Failed to read file {}: {e}", path.display()),
        })?;
    serde_json::from_str(&content).map_err(|e| cho_sdk::error::ChoSdkError::Parse {
        message: format!("Failed to parse JSON from {}: {e}", path.display()),
    })
}
