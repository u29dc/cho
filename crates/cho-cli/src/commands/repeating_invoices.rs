//! Repeating invoice commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Repeating invoice subcommands.
#[derive(Debug, Subcommand)]
pub enum RepeatingInvoiceCommands {
    /// List repeating invoices.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,

        /// Order expression (e.g. "Date DESC").
        #[arg(long)]
        order: Option<String>,
    },
    /// Get a single repeating invoice by ID.
    Get {
        /// Repeating invoice UUID.
        id: Uuid,
    },
}

/// Runs a repeating invoice subcommand.
pub async fn run(cmd: &RepeatingInvoiceCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        RepeatingInvoiceCommands::List { r#where, order } => {
            warn_if_suspicious_filter(r#where.as_ref());
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            if let Some(o) = order {
                params = params.with_order(o.clone());
            }
            let pagination = ctx.pagination_params();
            let items = ctx
                .client()
                .repeating_invoices()
                .list(&params, &pagination)
                .await?;
            let output = ctx.format_paginated_output(&items)?;
            println!("{output}");
            Ok(())
        }
        RepeatingInvoiceCommands::Get { id } => {
            let item = ctx.client().repeating_invoices().get(*id).await?;
            let output = ctx.format_output(&item)?;
            println!("{output}");
            Ok(())
        }
    }
}
