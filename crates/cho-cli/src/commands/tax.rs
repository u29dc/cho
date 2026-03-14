//! Tax and filing commands.

use std::time::Instant;

use cho_sdk::api::specs::by_name;
use cho_sdk::error::{ChoSdkError, Result};
use cho_sdk::liabilities::annotate_tax_response;
use cho_sdk::models::{ListResult, Pagination};
use clap::Subcommand;

use crate::context::CliContext;

use super::resources::ListArgs;
use super::resources_helpers::list_query;

/// Corporation tax return commands.
#[derive(Debug, Clone, Subcommand)]
pub enum CorporationTaxReturnCommands {
    /// List returns.
    List(Box<ListArgs>),
    /// Get one return by period end date.
    Get { period_ends_on: String },
    /// Mark as filed.
    MarkFiled { period_ends_on: String },
    /// Mark as unfiled.
    MarkUnfiled { period_ends_on: String },
    /// Mark as paid.
    MarkPaid { period_ends_on: String },
    /// Mark as unpaid.
    MarkUnpaid { period_ends_on: String },
}

/// VAT return commands.
#[derive(Debug, Clone, Subcommand)]
pub enum VatReturnCommands {
    /// List returns.
    List(Box<ListArgs>),
    /// Get one return by period end date.
    Get { period_ends_on: String },
    /// Mark as filed.
    MarkFiled { period_ends_on: String },
    /// Mark as unfiled.
    MarkUnfiled { period_ends_on: String },
    /// Mark payment as paid.
    MarkPaymentPaid {
        period_ends_on: String,
        payment_date: String,
    },
    /// Mark payment as unpaid.
    MarkPaymentUnpaid {
        period_ends_on: String,
        payment_date: String,
    },
}

/// Final accounts report commands.
#[derive(Debug, Clone, Subcommand)]
pub enum FinalAccountsReportCommands {
    /// List reports.
    List(Box<ListArgs>),
    /// Get report by period end date.
    Get { period_ends_on: String },
    /// Mark report as filed.
    MarkFiled { period_ends_on: String },
    /// Mark report as unfiled.
    MarkUnfiled { period_ends_on: String },
}

/// Self-assessment return commands.
#[derive(Debug, Clone, Subcommand)]
pub enum SelfAssessmentReturnCommands {
    /// List returns for a user.
    List {
        /// User id.
        #[arg(long)]
        user: String,
        /// Optional updated_since filter.
        #[arg(long)]
        updated_since: Option<String>,
    },
    /// Get return for user and period.
    Get {
        /// User id.
        #[arg(long)]
        user: String,
        /// Period end date.
        period_ends_on: String,
    },
    /// Mark as filed.
    MarkFiled {
        /// User id.
        #[arg(long)]
        user: String,
        /// Period end date.
        period_ends_on: String,
    },
    /// Mark as unfiled.
    MarkUnfiled {
        /// User id.
        #[arg(long)]
        user: String,
        /// Period end date.
        period_ends_on: String,
    },
    /// Mark payment as paid.
    MarkPaymentPaid {
        /// User id.
        #[arg(long)]
        user: String,
        /// Period end date.
        period_ends_on: String,
        /// Payment date.
        payment_date: String,
    },
    /// Mark payment as unpaid.
    MarkPaymentUnpaid {
        /// User id.
        #[arg(long)]
        user: String,
        /// Period end date.
        period_ends_on: String,
        /// Payment date.
        payment_date: String,
    },
}

/// Tool name for corporation tax command.
pub fn corporation_tool_name(command: &CorporationTaxReturnCommands) -> &'static str {
    match command {
        CorporationTaxReturnCommands::List(_) => "corporation-tax-returns.list",
        CorporationTaxReturnCommands::Get { .. } => "corporation-tax-returns.get",
        CorporationTaxReturnCommands::MarkFiled { .. } => "corporation-tax-returns.mark-filed",
        CorporationTaxReturnCommands::MarkUnfiled { .. } => "corporation-tax-returns.mark-unfiled",
        CorporationTaxReturnCommands::MarkPaid { .. } => "corporation-tax-returns.mark-paid",
        CorporationTaxReturnCommands::MarkUnpaid { .. } => "corporation-tax-returns.mark-unpaid",
    }
}

/// Tool name for vat command.
pub fn vat_tool_name(command: &VatReturnCommands) -> &'static str {
    match command {
        VatReturnCommands::List(_) => "vat-returns.list",
        VatReturnCommands::Get { .. } => "vat-returns.get",
        VatReturnCommands::MarkFiled { .. } => "vat-returns.mark-filed",
        VatReturnCommands::MarkUnfiled { .. } => "vat-returns.mark-unfiled",
        VatReturnCommands::MarkPaymentPaid { .. } => "vat-returns.mark-payment-paid",
        VatReturnCommands::MarkPaymentUnpaid { .. } => "vat-returns.mark-payment-unpaid",
    }
}

/// Tool name for final accounts command.
pub fn final_accounts_tool_name(command: &FinalAccountsReportCommands) -> &'static str {
    match command {
        FinalAccountsReportCommands::List(_) => "final-accounts-reports.list",
        FinalAccountsReportCommands::Get { .. } => "final-accounts-reports.get",
        FinalAccountsReportCommands::MarkFiled { .. } => "final-accounts-reports.mark-filed",
        FinalAccountsReportCommands::MarkUnfiled { .. } => "final-accounts-reports.mark-unfiled",
    }
}

/// Tool name for self-assessment command.
pub fn self_assessment_tool_name(command: &SelfAssessmentReturnCommands) -> &'static str {
    match command {
        SelfAssessmentReturnCommands::List { .. } => "self-assessment-returns.list",
        SelfAssessmentReturnCommands::Get { .. } => "self-assessment-returns.get",
        SelfAssessmentReturnCommands::MarkFiled { .. } => "self-assessment-returns.mark-filed",
        SelfAssessmentReturnCommands::MarkUnfiled { .. } => "self-assessment-returns.mark-unfiled",
        SelfAssessmentReturnCommands::MarkPaymentPaid { .. } => {
            "self-assessment-returns.mark-payment-paid"
        }
        SelfAssessmentReturnCommands::MarkPaymentUnpaid { .. } => {
            "self-assessment-returns.mark-payment-unpaid"
        }
    }
}

/// Runs corporation tax commands.
pub async fn run_corporation_tax(
    command: &CorporationTaxReturnCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    let spec = by_name("corporation-tax-returns").ok_or_else(|| ChoSdkError::Config {
        message: "Missing corporation-tax-returns resource spec".to_string(),
    })?;
    let api = ctx.client().resource(spec);

    match command {
        CorporationTaxReturnCommands::List(args) => {
            let result = tax_list_result(
                api.list(&list_query(args)?, pagination_from(ctx, args))
                    .await?,
            );
            ctx.emit_list("corporation-tax-returns.list", &result, start)
        }
        CorporationTaxReturnCommands::Get { period_ends_on } => {
            let mut value = api.get(period_ends_on).await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("corporation-tax-returns.get", &value, start)
        }
        CorporationTaxReturnCommands::MarkFiled { period_ends_on } => {
            ctx.require_writes_allowed()?;
            let mut value = api
                .action(
                    period_ends_on,
                    reqwest::Method::PUT,
                    "mark_as_filed",
                    None,
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("corporation-tax-returns.mark-filed", &value, start)
        }
        CorporationTaxReturnCommands::MarkUnfiled { period_ends_on } => {
            ctx.require_writes_allowed()?;
            let mut value = api
                .action(
                    period_ends_on,
                    reqwest::Method::PUT,
                    "mark_as_unfiled",
                    None,
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("corporation-tax-returns.mark-unfiled", &value, start)
        }
        CorporationTaxReturnCommands::MarkPaid { period_ends_on } => {
            ctx.require_writes_allowed()?;
            let mut value = api
                .action(
                    period_ends_on,
                    reqwest::Method::PUT,
                    "mark_as_paid",
                    None,
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("corporation-tax-returns.mark-paid", &value, start)
        }
        CorporationTaxReturnCommands::MarkUnpaid { period_ends_on } => {
            ctx.require_writes_allowed()?;
            let mut value = api
                .action(
                    period_ends_on,
                    reqwest::Method::PUT,
                    "mark_as_unpaid",
                    None,
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("corporation-tax-returns.mark-unpaid", &value, start)
        }
    }
}

/// Runs vat commands.
pub async fn run_vat(command: &VatReturnCommands, ctx: &CliContext, start: Instant) -> Result<()> {
    let spec = by_name("vat-returns").ok_or_else(|| ChoSdkError::Config {
        message: "Missing vat-returns resource spec".to_string(),
    })?;
    let api = ctx.client().resource(spec);

    match command {
        VatReturnCommands::List(args) => {
            let result = tax_list_result(
                api.list(&list_query(args)?, pagination_from(ctx, args))
                    .await?,
            );
            ctx.emit_list("vat-returns.list", &result, start)
        }
        VatReturnCommands::Get { period_ends_on } => {
            let mut value = api.get(period_ends_on).await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("vat-returns.get", &value, start)
        }
        VatReturnCommands::MarkFiled { period_ends_on } => {
            ctx.require_writes_allowed()?;
            let mut value = api
                .action(
                    period_ends_on,
                    reqwest::Method::PUT,
                    "mark_as_filed",
                    None,
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("vat-returns.mark-filed", &value, start)
        }
        VatReturnCommands::MarkUnfiled { period_ends_on } => {
            ctx.require_writes_allowed()?;
            let mut value = api
                .action(
                    period_ends_on,
                    reqwest::Method::PUT,
                    "mark_as_unfiled",
                    None,
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("vat-returns.mark-unfiled", &value, start)
        }
        VatReturnCommands::MarkPaymentPaid {
            period_ends_on,
            payment_date,
        } => {
            ctx.require_writes_allowed()?;
            let mut value = api
                .action(
                    period_ends_on,
                    reqwest::Method::PUT,
                    &format!(
                        "payments/{}/mark_as_paid",
                        encode_path_segment(payment_date)
                    ),
                    None,
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("vat-returns.mark-payment-paid", &value, start)
        }
        VatReturnCommands::MarkPaymentUnpaid {
            period_ends_on,
            payment_date,
        } => {
            ctx.require_writes_allowed()?;
            let mut value = api
                .action(
                    period_ends_on,
                    reqwest::Method::PUT,
                    &format!(
                        "payments/{}/mark_as_unpaid",
                        encode_path_segment(payment_date)
                    ),
                    None,
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("vat-returns.mark-payment-unpaid", &value, start)
        }
    }
}

/// Runs final accounts report commands.
pub async fn run_final_accounts(
    command: &FinalAccountsReportCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    let spec = by_name("final-accounts-reports").ok_or_else(|| ChoSdkError::Config {
        message: "Missing final-accounts-reports resource spec".to_string(),
    })?;
    let api = ctx.client().resource(spec);

    match command {
        FinalAccountsReportCommands::List(args) => {
            let result = tax_list_result(
                api.list(&list_query(args)?, pagination_from(ctx, args))
                    .await?,
            );
            ctx.emit_list("final-accounts-reports.list", &result, start)
        }
        FinalAccountsReportCommands::Get { period_ends_on } => {
            let mut value = api.get(period_ends_on).await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("final-accounts-reports.get", &value, start)
        }
        FinalAccountsReportCommands::MarkFiled { period_ends_on } => {
            ctx.require_writes_allowed()?;
            let mut value = api
                .action(
                    period_ends_on,
                    reqwest::Method::PUT,
                    "mark_as_filed",
                    None,
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("final-accounts-reports.mark-filed", &value, start)
        }
        FinalAccountsReportCommands::MarkUnfiled { period_ends_on } => {
            ctx.require_writes_allowed()?;
            let mut value = api
                .action(
                    period_ends_on,
                    reqwest::Method::PUT,
                    "mark_as_unfiled",
                    None,
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("final-accounts-reports.mark-unfiled", &value, start)
        }
    }
}

/// Runs self-assessment commands.
pub async fn run_self_assessment(
    command: &SelfAssessmentReturnCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        SelfAssessmentReturnCommands::List {
            user,
            updated_since,
        } => {
            let mut query = Vec::new();
            if let Some(updated_since) = updated_since
                && !updated_since.trim().is_empty()
            {
                query.push(("updated_since".to_string(), updated_since.clone()));
            }

            let pagination = ctx.pagination();
            let result = ctx
                .client()
                .list_paginated(
                    &format!("users/{}/self_assessment_returns", user_id_segment(user)),
                    "self_assessment_returns",
                    &query,
                    pagination,
                )
                .await?;
            let result = tax_list_result(result);
            ctx.emit_list("self-assessment-returns.list", &result, start)
        }
        SelfAssessmentReturnCommands::Get {
            user,
            period_ends_on,
        } => {
            let value = ctx
                .client()
                .get_json(
                    &format!(
                        "users/{}/self_assessment_returns/{}",
                        user_id_segment(user),
                        encode_path_segment(period_ends_on)
                    ),
                    &[],
                )
                .await?;
            let mut payload = value
                .get("self_assessment_return")
                .cloned()
                .unwrap_or(value);
            annotate_tax_response(&mut payload);
            ctx.emit_success("self-assessment-returns.get", &payload, start)
        }
        SelfAssessmentReturnCommands::MarkFiled {
            user,
            period_ends_on,
        } => {
            ctx.require_writes_allowed()?;
            let mut value = ctx
                .client()
                .put_json(
                    &format!(
                        "users/{}/self_assessment_returns/{}/mark_as_filed",
                        user_id_segment(user),
                        encode_path_segment(period_ends_on)
                    ),
                    &serde_json::json!({}),
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("self-assessment-returns.mark-filed", &value, start)
        }
        SelfAssessmentReturnCommands::MarkUnfiled {
            user,
            period_ends_on,
        } => {
            ctx.require_writes_allowed()?;
            let mut value = ctx
                .client()
                .put_json(
                    &format!(
                        "users/{}/self_assessment_returns/{}/mark_as_unfiled",
                        user_id_segment(user),
                        encode_path_segment(period_ends_on)
                    ),
                    &serde_json::json!({}),
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("self-assessment-returns.mark-unfiled", &value, start)
        }
        SelfAssessmentReturnCommands::MarkPaymentPaid {
            user,
            period_ends_on,
            payment_date,
        } => {
            ctx.require_writes_allowed()?;
            let mut value = ctx
                .client()
                .put_json(
                    &format!(
                        "users/{}/self_assessment_returns/{}/payments/{}/mark_as_paid",
                        user_id_segment(user),
                        encode_path_segment(period_ends_on),
                        encode_path_segment(payment_date)
                    ),
                    &serde_json::json!({}),
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("self-assessment-returns.mark-payment-paid", &value, start)
        }
        SelfAssessmentReturnCommands::MarkPaymentUnpaid {
            user,
            period_ends_on,
            payment_date,
        } => {
            ctx.require_writes_allowed()?;
            let mut value = ctx
                .client()
                .put_json(
                    &format!(
                        "users/{}/self_assessment_returns/{}/payments/{}/mark_as_unpaid",
                        user_id_segment(user),
                        encode_path_segment(period_ends_on),
                        encode_path_segment(payment_date)
                    ),
                    &serde_json::json!({}),
                    true,
                )
                .await?;
            annotate_tax_response(&mut value);
            ctx.emit_success("self-assessment-returns.mark-payment-unpaid", &value, start)
        }
    }
}

fn tax_list_result(mut result: ListResult) -> ListResult {
    for item in &mut result.items {
        annotate_tax_response(item);
    }
    result
}

fn pagination_from(ctx: &CliContext, args: &ListArgs) -> Pagination {
    let mut pagination = ctx.pagination();
    if let Some(per_page) = args.per_page {
        pagination.per_page = per_page.clamp(1, 100);
    }
    pagination
}

fn encode_path_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn user_id_segment(user: &str) -> String {
    let trimmed = user.trim().trim_end_matches('/');
    if (trimmed.starts_with("https://") || trimmed.starts_with("http://"))
        && let Some(id) = trimmed.rsplit('/').next()
    {
        return encode_path_segment(id);
    }

    encode_path_segment(trimmed)
}
