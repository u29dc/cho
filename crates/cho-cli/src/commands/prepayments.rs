//! Prepayment commands: list, get.

use std::time::Instant;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Prepayment subcommands.
#[derive(Debug, Subcommand)]
pub enum PrepaymentCommands {
    /// List prepayments.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,

        /// Order expression (e.g. "Date DESC").
        #[arg(long)]
        order: Option<String>,
    },
    /// Get a single prepayment by ID.
    Get {
        /// Prepayment UUID.
        id: Uuid,
    },
}

/// Returns the tool name for the given subcommand.
pub fn tool_name(cmd: &PrepaymentCommands) -> &'static str {
    match cmd {
        PrepaymentCommands::List { .. } => "prepayments.list",
        PrepaymentCommands::Get { .. } => "prepayments.get",
    }
}

/// Runs a prepayment subcommand.
pub async fn run(
    cmd: &PrepaymentCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        PrepaymentCommands::List { r#where, order } => {
            warn_if_suspicious_filter(r#where.as_ref());
            warn_if_suspicious_filter(order.as_ref());
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
                .prepayments()
                .list(&params, &pagination)
                .await?;
            ctx.emit_list("prepayments.list", &items, start)?;
            Ok(())
        }
        PrepaymentCommands::Get { id } => {
            let item = ctx.client().prepayments().get(*id).await?;
            ctx.emit_success("prepayments.get", &item, start)?;
            Ok(())
        }
    }
}
