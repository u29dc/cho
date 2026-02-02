//! Payment commands: list, get, create, delete.

use std::path::PathBuf;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;
use cho_sdk::models::payment::Payment;

use crate::context::CliContext;

/// Payment subcommands.
#[derive(Debug, Subcommand)]
pub enum PaymentCommands {
    /// List payments.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,
    },
    /// Get a single payment by ID.
    Get {
        /// Payment ID (UUID).
        id: Uuid,
    },
    /// Create a new payment from a JSON file.
    Create {
        /// Path to JSON file containing the payment data.
        #[arg(long)]
        file: PathBuf,
        /// Idempotency key for safe retries.
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    /// Delete (void) an existing payment.
    Delete {
        /// Payment ID (UUID) to delete.
        id: Uuid,
        /// Idempotency key for safe retries.
        #[arg(long)]
        idempotency_key: Option<String>,
    },
}

/// Runs a payment subcommand.
pub async fn run(cmd: &PaymentCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        PaymentCommands::List { r#where } => {
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let pagination = ctx.pagination_params();
            let payments = ctx.client().payments().list(&params, &pagination).await?;
            let output = ctx.format_list_output(&payments)?;
            println!("{output}");
            Ok(())
        }
        PaymentCommands::Get { id } => {
            let payment = ctx.client().payments().get(*id).await?;
            let output = ctx.format_output(&payment)?;
            println!("{output}");
            Ok(())
        }
        PaymentCommands::Create {
            file,
            idempotency_key,
        } => {
            ctx.require_writes_allowed()?;
            let payment: Payment = crate::commands::invoices::read_json_file(file)?;
            let result = ctx
                .client()
                .payments()
                .create(&payment, idempotency_key.as_deref())
                .await?;
            let output = ctx.format_output(&result)?;
            println!("{output}");
            Ok(())
        }
        PaymentCommands::Delete {
            id,
            idempotency_key,
        } => {
            ctx.require_writes_allowed()?;
            let result = ctx
                .client()
                .payments()
                .delete(*id, idempotency_key.as_deref())
                .await?;
            let output = ctx.format_output(&result)?;
            println!("{output}");
            Ok(())
        }
    }
}
