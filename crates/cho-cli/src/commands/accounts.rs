//! Account commands: list.

use clap::Subcommand;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Account subcommands.
#[derive(Debug, Subcommand)]
pub enum AccountCommands {
    /// List chart of accounts.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,
    },
}

/// Runs an account subcommand.
pub async fn run(cmd: &AccountCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        AccountCommands::List { r#where } => {
            warn_if_suspicious_filter(r#where.as_ref());
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let accounts = ctx.client().accounts().list(&params).await?;
            let output = ctx.format_list_output(&accounts)?;
            println!("{output}");
            Ok(())
        }
    }
}
