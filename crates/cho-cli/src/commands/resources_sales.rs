//! Sales resource command handlers.

use std::time::Instant;

use cho_sdk::api::specs::by_name;
use cho_sdk::error::{ChoSdkError, Result};

use crate::context::CliContext;

use super::resources::{
    CreditNoteCommands, CreditNoteTransition, DefaultAdditionalTextCommands, EstimateCommands,
    EstimateTransition, InvoiceCommands, InvoiceTransition, ResourceCommands, run_resource,
};
use super::resources_helpers::{
    fetch_pdf_resource, read_optional_json_file, run_default_additional_text,
};

/// Executes invoice command.
pub async fn run_invoices(
    command: &InvoiceCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        InvoiceCommands::List(args) => {
            run_resource(
                "invoices",
                &ResourceCommands::List((**args).clone()),
                ctx,
                start,
            )
            .await
        }
        InvoiceCommands::Get { id } => {
            run_resource(
                "invoices",
                &ResourceCommands::Get { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        InvoiceCommands::Create { file } => {
            run_resource(
                "invoices",
                &ResourceCommands::Create { file: file.clone() },
                ctx,
                start,
            )
            .await
        }
        InvoiceCommands::Update { id, file } => {
            run_resource(
                "invoices",
                &ResourceCommands::Update {
                    id: id.clone(),
                    file: file.clone(),
                },
                ctx,
                start,
            )
            .await
        }
        InvoiceCommands::Delete { id } => {
            run_resource(
                "invoices",
                &ResourceCommands::Delete { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        InvoiceCommands::Transition { id, action } => {
            ctx.require_writes_allowed()?;
            let spec = by_name("invoices").ok_or_else(|| ChoSdkError::Config {
                message: "Missing invoices resource spec".to_string(),
            })?;
            let api = ctx.client().resource(spec);
            let suffix = match action {
                InvoiceTransition::MarkAsDraft => "transitions/mark_as_draft",
                InvoiceTransition::MarkAsSent => "transitions/mark_as_sent",
                InvoiceTransition::MarkAsScheduled => "transitions/mark_as_scheduled",
                InvoiceTransition::MarkAsCancelled => "transitions/mark_as_cancelled",
                InvoiceTransition::ConvertToCreditNote => "transitions/convert_to_credit_note",
            };
            let value = api
                .action(id, reqwest::Method::PUT, suffix, None, true)
                .await?;
            ctx.emit_success("invoices.transition", &value, start)
        }
        InvoiceCommands::SendEmail { id, file } => {
            ctx.require_writes_allowed()?;
            let spec = by_name("invoices").ok_or_else(|| ChoSdkError::Config {
                message: "Missing invoices resource spec".to_string(),
            })?;
            let api = ctx.client().resource(spec);
            let payload = read_optional_json_file(file)?;
            ctx.log_input("invoices.send-email", &payload);
            let value = api
                .action(
                    id,
                    reqwest::Method::POST,
                    "send_email",
                    Some(&payload),
                    true,
                )
                .await?;
            ctx.emit_success("invoices.send-email", &value, start)
        }
        InvoiceCommands::Duplicate { id } => {
            ctx.require_writes_allowed()?;
            let spec = by_name("invoices").ok_or_else(|| ChoSdkError::Config {
                message: "Missing invoices resource spec".to_string(),
            })?;
            let api = ctx.client().resource(spec);
            let value = api
                .action(
                    id,
                    reqwest::Method::POST,
                    "duplicate",
                    Some(&serde_json::json!({})),
                    true,
                )
                .await?;
            ctx.emit_success("invoices.duplicate", &value, start)
        }
        InvoiceCommands::DirectDebit { id, file } => {
            ctx.require_writes_allowed()?;
            let spec = by_name("invoices").ok_or_else(|| ChoSdkError::Config {
                message: "Missing invoices resource spec".to_string(),
            })?;
            let api = ctx.client().resource(spec);
            let payload = read_optional_json_file(file)?;
            ctx.log_input("invoices.direct-debit", &payload);
            let value = api
                .action(
                    id,
                    reqwest::Method::POST,
                    "direct_debit",
                    Some(&payload),
                    true,
                )
                .await?;
            ctx.emit_success("invoices.direct-debit", &value, start)
        }
        InvoiceCommands::Timeline => {
            let value = ctx.client().get_json("invoices/timeline", &[]).await?;
            ctx.emit_success("invoices.timeline", &value, start)
        }
        InvoiceCommands::GetPdf { id, output } => {
            fetch_pdf_resource(
                "invoices",
                id,
                output.as_deref(),
                "invoices.get-pdf",
                ctx,
                start,
            )
            .await
        }
        InvoiceCommands::DefaultAdditionalText { command } => {
            run_default_additional_text("invoices", command, "invoices", ctx, start).await
        }
    }
}

/// Returns tool name for invoice command.
pub fn invoices_tool_name(command: &InvoiceCommands) -> String {
    match command {
        InvoiceCommands::List(_) => "invoices.list".to_string(),
        InvoiceCommands::Get { .. } => "invoices.get".to_string(),
        InvoiceCommands::Create { .. } => "invoices.create".to_string(),
        InvoiceCommands::Update { .. } => "invoices.update".to_string(),
        InvoiceCommands::Delete { .. } => "invoices.delete".to_string(),
        InvoiceCommands::Transition { .. } => "invoices.transition".to_string(),
        InvoiceCommands::SendEmail { .. } => "invoices.send-email".to_string(),
        InvoiceCommands::Duplicate { .. } => "invoices.duplicate".to_string(),
        InvoiceCommands::DirectDebit { .. } => "invoices.direct-debit".to_string(),
        InvoiceCommands::Timeline => "invoices.timeline".to_string(),
        InvoiceCommands::GetPdf { .. } => "invoices.get-pdf".to_string(),
        InvoiceCommands::DefaultAdditionalText { command } => match command {
            DefaultAdditionalTextCommands::Get => {
                "invoices.default-additional-text.get".to_string()
            }
            DefaultAdditionalTextCommands::Update { .. } => {
                "invoices.default-additional-text.update".to_string()
            }
            DefaultAdditionalTextCommands::Delete => {
                "invoices.default-additional-text.delete".to_string()
            }
        },
    }
}

/// Executes credit note command.
pub async fn run_credit_notes(
    command: &CreditNoteCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        CreditNoteCommands::List(args) => {
            run_resource(
                "credit-notes",
                &ResourceCommands::List((**args).clone()),
                ctx,
                start,
            )
            .await
        }
        CreditNoteCommands::Get { id } => {
            run_resource(
                "credit-notes",
                &ResourceCommands::Get { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        CreditNoteCommands::Create { file } => {
            run_resource(
                "credit-notes",
                &ResourceCommands::Create { file: file.clone() },
                ctx,
                start,
            )
            .await
        }
        CreditNoteCommands::Update { id, file } => {
            run_resource(
                "credit-notes",
                &ResourceCommands::Update {
                    id: id.clone(),
                    file: file.clone(),
                },
                ctx,
                start,
            )
            .await
        }
        CreditNoteCommands::Delete { id } => {
            run_resource(
                "credit-notes",
                &ResourceCommands::Delete { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        CreditNoteCommands::Transition { id, action } => {
            ctx.require_writes_allowed()?;
            let spec = by_name("credit-notes").ok_or_else(|| ChoSdkError::Config {
                message: "Missing credit-notes resource spec".to_string(),
            })?;
            let api = ctx.client().resource(spec);
            let suffix = match action {
                CreditNoteTransition::MarkAsDraft => "transitions/mark_as_draft",
                CreditNoteTransition::MarkAsSent => "transitions/mark_as_sent",
            };
            let value = api
                .action(id, reqwest::Method::PUT, suffix, None, true)
                .await?;
            ctx.emit_success("credit-notes.transition", &value, start)
        }
        CreditNoteCommands::SendEmail { id, file } => {
            ctx.require_writes_allowed()?;
            let spec = by_name("credit-notes").ok_or_else(|| ChoSdkError::Config {
                message: "Missing credit-notes resource spec".to_string(),
            })?;
            let api = ctx.client().resource(spec);
            let payload = read_optional_json_file(file)?;
            ctx.log_input("credit-notes.send-email", &payload);
            let value = api
                .action(
                    id,
                    reqwest::Method::POST,
                    "send_email",
                    Some(&payload),
                    true,
                )
                .await?;
            ctx.emit_success("credit-notes.send-email", &value, start)
        }
        CreditNoteCommands::GetPdf { id, output } => {
            fetch_pdf_resource(
                "credit_notes",
                id,
                output.as_deref(),
                "credit-notes.get-pdf",
                ctx,
                start,
            )
            .await
        }
    }
}

/// Returns tool name for credit note command.
pub fn credit_notes_tool_name(command: &CreditNoteCommands) -> String {
    match command {
        CreditNoteCommands::List(_) => "credit-notes.list".to_string(),
        CreditNoteCommands::Get { .. } => "credit-notes.get".to_string(),
        CreditNoteCommands::Create { .. } => "credit-notes.create".to_string(),
        CreditNoteCommands::Update { .. } => "credit-notes.update".to_string(),
        CreditNoteCommands::Delete { .. } => "credit-notes.delete".to_string(),
        CreditNoteCommands::Transition { .. } => "credit-notes.transition".to_string(),
        CreditNoteCommands::SendEmail { .. } => "credit-notes.send-email".to_string(),
        CreditNoteCommands::GetPdf { .. } => "credit-notes.get-pdf".to_string(),
    }
}

/// Executes estimate command.
pub async fn run_estimates(
    command: &EstimateCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        EstimateCommands::List(args) => {
            run_resource(
                "estimates",
                &ResourceCommands::List((**args).clone()),
                ctx,
                start,
            )
            .await
        }
        EstimateCommands::Get { id } => {
            run_resource(
                "estimates",
                &ResourceCommands::Get { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        EstimateCommands::Create { file } => {
            run_resource(
                "estimates",
                &ResourceCommands::Create { file: file.clone() },
                ctx,
                start,
            )
            .await
        }
        EstimateCommands::Update { id, file } => {
            run_resource(
                "estimates",
                &ResourceCommands::Update {
                    id: id.clone(),
                    file: file.clone(),
                },
                ctx,
                start,
            )
            .await
        }
        EstimateCommands::Delete { id } => {
            run_resource(
                "estimates",
                &ResourceCommands::Delete { id: id.clone() },
                ctx,
                start,
            )
            .await
        }
        EstimateCommands::Transition { id, action } => {
            ctx.require_writes_allowed()?;
            let spec = by_name("estimates").ok_or_else(|| ChoSdkError::Config {
                message: "Missing estimates resource spec".to_string(),
            })?;
            let api = ctx.client().resource(spec);
            let suffix = match action {
                EstimateTransition::MarkAsDraft => "transitions/mark_as_draft",
                EstimateTransition::MarkAsSent => "transitions/mark_as_sent",
                EstimateTransition::MarkAsApproved => "transitions/mark_as_approved",
                EstimateTransition::MarkAsRejected => "transitions/mark_as_rejected",
                EstimateTransition::ConvertToInvoice => "transitions/convert_to_invoice",
            };
            let value = api
                .action(id, reqwest::Method::PUT, suffix, None, true)
                .await?;
            ctx.emit_success("estimates.transition", &value, start)
        }
        EstimateCommands::SendEmail { id, file } => {
            ctx.require_writes_allowed()?;
            let spec = by_name("estimates").ok_or_else(|| ChoSdkError::Config {
                message: "Missing estimates resource spec".to_string(),
            })?;
            let api = ctx.client().resource(spec);
            let payload = read_optional_json_file(file)?;
            ctx.log_input("estimates.send-email", &payload);
            let value = api
                .action(
                    id,
                    reqwest::Method::POST,
                    "send_email",
                    Some(&payload),
                    true,
                )
                .await?;
            ctx.emit_success("estimates.send-email", &value, start)
        }
        EstimateCommands::Duplicate { id } => {
            ctx.require_writes_allowed()?;
            let spec = by_name("estimates").ok_or_else(|| ChoSdkError::Config {
                message: "Missing estimates resource spec".to_string(),
            })?;
            let api = ctx.client().resource(spec);
            let value = api
                .action(
                    id,
                    reqwest::Method::POST,
                    "duplicate",
                    Some(&serde_json::json!({})),
                    true,
                )
                .await?;
            ctx.emit_success("estimates.duplicate", &value, start)
        }
        EstimateCommands::GetPdf { id, output } => {
            fetch_pdf_resource(
                "estimates",
                id,
                output.as_deref(),
                "estimates.get-pdf",
                ctx,
                start,
            )
            .await
        }
        EstimateCommands::DefaultAdditionalText { command } => {
            run_default_additional_text("estimates", command, "estimates", ctx, start).await
        }
    }
}

/// Returns tool name for estimate command.
pub fn estimates_tool_name(command: &EstimateCommands) -> String {
    match command {
        EstimateCommands::List(_) => "estimates.list".to_string(),
        EstimateCommands::Get { .. } => "estimates.get".to_string(),
        EstimateCommands::Create { .. } => "estimates.create".to_string(),
        EstimateCommands::Update { .. } => "estimates.update".to_string(),
        EstimateCommands::Delete { .. } => "estimates.delete".to_string(),
        EstimateCommands::Transition { .. } => "estimates.transition".to_string(),
        EstimateCommands::SendEmail { .. } => "estimates.send-email".to_string(),
        EstimateCommands::Duplicate { .. } => "estimates.duplicate".to_string(),
        EstimateCommands::GetPdf { .. } => "estimates.get-pdf".to_string(),
        EstimateCommands::DefaultAdditionalText { command } => match command {
            DefaultAdditionalTextCommands::Get => {
                "estimates.default-additional-text.get".to_string()
            }
            DefaultAdditionalTextCommands::Update { .. } => {
                "estimates.default-additional-text.update".to_string()
            }
            DefaultAdditionalTextCommands::Delete => {
                "estimates.default-additional-text.delete".to_string()
            }
        },
    }
}
