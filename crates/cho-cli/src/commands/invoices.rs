//! Invoice commands: list, get, create, update.

use std::path::PathBuf;
use std::time::Instant;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;
use cho_sdk::models::invoice::Invoice;

use crate::commands::utils::{read_json_file, validate_date};
use crate::context::{CliContext, warn_if_suspicious_filter};

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

/// Returns the tool name for an invoice subcommand.
pub fn tool_name(cmd: &InvoiceCommands) -> &'static str {
    match cmd {
        InvoiceCommands::List { .. } => "invoices.list",
        InvoiceCommands::Get { .. } => "invoices.get",
        InvoiceCommands::Create { .. } => "invoices.create",
        InvoiceCommands::Update { .. } => "invoices.update",
    }
}

/// Runs an invoice subcommand.
pub async fn run(
    cmd: &InvoiceCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        InvoiceCommands::List {
            r#where,
            order,
            from,
            to,
            summary,
        } => {
            // Check for suspicious OData patterns in user-provided filter/order
            warn_if_suspicious_filter(r#where.as_ref());
            warn_if_suspicious_filter(order.as_ref());

            let mut params = ListParams::new();

            // Build where filter combining explicit where + date range
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

            if let Some(o) = order {
                params = params.with_order(o.clone());
            }
            if *summary {
                params = params.with_summary_only(true);
            }

            let pagination = ctx.pagination_params();
            let invoices = ctx.client().invoices().list(&params, &pagination).await?;
            ctx.emit_list("invoices.list", &invoices, start)?;
            Ok(())
        }
        InvoiceCommands::Get { id_or_number } => {
            let invoice = if let Ok(uuid) = id_or_number.parse::<Uuid>() {
                ctx.client().invoices().get(uuid).await?
            } else {
                ctx.client().invoices().get_by_number(id_or_number).await?
            };
            ctx.emit_success("invoices.get", &invoice, start)?;
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
            ctx.emit_success("invoices.create", &result, start)?;
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
            ctx.emit_success("invoices.update", &result, start)?;
            Ok(())
        }
    }
}
