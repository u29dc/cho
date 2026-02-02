//! Invoice commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

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
            let output = ctx.format_list_output(&invoices)?;
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
    }
}
