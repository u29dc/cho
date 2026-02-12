//! Tracking category commands: list, get.

use std::time::Instant;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Tracking category subcommands.
#[derive(Debug, Subcommand)]
pub enum TrackingCategoryCommands {
    /// List tracking categories.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,
    },
    /// Get a single tracking category by ID.
    Get {
        /// Tracking category UUID.
        id: Uuid,
    },
}

/// Returns the tool name for the given subcommand.
pub fn tool_name(cmd: &TrackingCategoryCommands) -> &'static str {
    match cmd {
        TrackingCategoryCommands::List { .. } => "tracking-categories.list",
        TrackingCategoryCommands::Get { .. } => "tracking-categories.get",
    }
}

/// Runs a tracking category subcommand.
pub async fn run(
    cmd: &TrackingCategoryCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        TrackingCategoryCommands::List { r#where } => {
            warn_if_suspicious_filter(r#where.as_ref());
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let items = ctx.client().tracking_categories().list(&params).await?;
            ctx.emit_items("tracking-categories.list", &items, start)?;
            Ok(())
        }
        TrackingCategoryCommands::Get { id } => {
            let item = ctx.client().tracking_categories().get(*id).await?;
            ctx.emit_success("tracking-categories.get", &item, start)?;
            Ok(())
        }
    }
}
