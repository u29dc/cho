//! Item commands: list, get.

use std::time::Instant;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Item subcommands.
#[derive(Debug, Subcommand)]
pub enum ItemCommands {
    /// List items.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,
    },
    /// Get a single item by ID.
    Get {
        /// Item UUID.
        id: Uuid,
    },
}

/// Returns the tool name for an item subcommand.
pub fn tool_name(cmd: &ItemCommands) -> &'static str {
    match cmd {
        ItemCommands::List { .. } => "items.list",
        ItemCommands::Get { .. } => "items.get",
    }
}

/// Runs an item subcommand.
pub async fn run(
    cmd: &ItemCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        ItemCommands::List { r#where } => {
            warn_if_suspicious_filter(r#where.as_ref());
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let items = ctx.client().items().list(&params).await?;
            ctx.emit_items("items.list", &items, start)?;
            Ok(())
        }
        ItemCommands::Get { id } => {
            let item = ctx.client().items().get(*id).await?;
            ctx.emit_success("items.get", &item, start)?;
            Ok(())
        }
    }
}
