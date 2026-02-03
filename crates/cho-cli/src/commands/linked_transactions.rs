//! Linked transaction commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Linked transaction subcommands.
#[derive(Debug, Subcommand)]
pub enum LinkedTransactionCommands {
    /// List linked transactions.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,

        /// Order expression (e.g. "Date DESC").
        #[arg(long)]
        order: Option<String>,
    },
    /// Get a single linked transaction by ID.
    Get {
        /// Linked transaction UUID.
        id: Uuid,
    },
}

/// Runs a linked transaction subcommand.
pub async fn run(cmd: &LinkedTransactionCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        LinkedTransactionCommands::List { r#where, order } => {
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
                .linked_transactions()
                .list(&params, &pagination)
                .await?;
            let output = ctx.format_paginated_output(&items)?;
            println!("{output}");
            Ok(())
        }
        LinkedTransactionCommands::Get { id } => {
            let item = ctx.client().linked_transactions().get(*id).await?;
            let output = ctx.format_output(&item)?;
            println!("{output}");
            Ok(())
        }
    }
}
