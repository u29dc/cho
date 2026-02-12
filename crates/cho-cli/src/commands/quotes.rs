//! Quote commands: list, get.

use std::time::Instant;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Quote subcommands.
#[derive(Debug, Subcommand)]
pub enum QuoteCommands {
    /// List quotes.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,

        /// Order expression (e.g. "Date DESC").
        #[arg(long)]
        order: Option<String>,
    },
    /// Get a single quote by ID.
    Get {
        /// Quote UUID.
        id: Uuid,
    },
}

/// Returns the tool name for a quote subcommand.
pub fn tool_name(cmd: &QuoteCommands) -> &'static str {
    match cmd {
        QuoteCommands::List { .. } => "quotes.list",
        QuoteCommands::Get { .. } => "quotes.get",
    }
}

/// Runs a quote subcommand.
pub async fn run(
    cmd: &QuoteCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        QuoteCommands::List { r#where, order } => {
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
            let items = ctx.client().quotes().list(&params, &pagination).await?;
            ctx.emit_list("quotes.list", &items, start)?;
            Ok(())
        }
        QuoteCommands::Get { id } => {
            let item = ctx.client().quotes().get(*id).await?;
            ctx.emit_success("quotes.get", &item, start)?;
            Ok(())
        }
    }
}
