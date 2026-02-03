//! Purchase order commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Purchase order subcommands.
#[derive(Debug, Subcommand)]
pub enum PurchaseOrderCommands {
    /// List purchase orders.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,

        /// Order expression (e.g. "Date DESC").
        #[arg(long)]
        order: Option<String>,
    },
    /// Get a single purchase order by ID.
    Get {
        /// Purchase order UUID.
        id: Uuid,
    },
}

/// Runs a purchase order subcommand.
pub async fn run(cmd: &PurchaseOrderCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        PurchaseOrderCommands::List { r#where, order } => {
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
                .purchase_orders()
                .list(&params, &pagination)
                .await?;
            let output = ctx.format_paginated_output(&items)?;
            println!("{output}");
            Ok(())
        }
        PurchaseOrderCommands::Get { id } => {
            let item = ctx.client().purchase_orders().get(*id).await?;
            let output = ctx.format_output(&item)?;
            println!("{output}");
            Ok(())
        }
    }
}
