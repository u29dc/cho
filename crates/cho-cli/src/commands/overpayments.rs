//! Overpayment commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Overpayment subcommands.
#[derive(Debug, Subcommand)]
pub enum OverpaymentCommands {
    /// List overpayments.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,

        /// Order expression (e.g. "Date DESC").
        #[arg(long)]
        order: Option<String>,
    },
    /// Get a single overpayment by ID.
    Get {
        /// Overpayment UUID.
        id: Uuid,
    },
}

/// Runs an overpayment subcommand.
pub async fn run(cmd: &OverpaymentCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        OverpaymentCommands::List { r#where, order } => {
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
                .overpayments()
                .list(&params, &pagination)
                .await?;
            let output = ctx.format_paginated_output(&items)?;
            println!("{output}");
            Ok(())
        }
        OverpaymentCommands::Get { id } => {
            let item = ctx.client().overpayments().get(*id).await?;
            let output = ctx.format_output(&item)?;
            println!("{output}");
            Ok(())
        }
    }
}
