//! Payroll commands.

use std::time::Instant;

use cho_sdk::error::Result;
use clap::Subcommand;

use crate::context::CliContext;

/// Payroll commands.
#[derive(Debug, Clone, Subcommand)]
pub enum PayrollCommands {
    /// List periods for tax year.
    Periods {
        /// Payroll year end (e.g. 2026).
        year: i32,
    },
    /// Get one period and payslips.
    Period {
        /// Payroll year end.
        year: i32,
        /// Period number.
        period: i32,
    },
    /// Mark payroll payment as paid.
    MarkPaymentPaid {
        /// Payroll year end.
        year: i32,
        /// Payment date.
        payment_date: String,
    },
    /// Mark payroll payment as unpaid.
    MarkPaymentUnpaid {
        /// Payroll year end.
        year: i32,
        /// Payment date.
        payment_date: String,
    },
}

/// Payroll profile commands.
#[derive(Debug, Clone, Subcommand)]
pub enum PayrollProfileCommands {
    /// List payroll profiles for year.
    List {
        /// Payroll year end.
        year: i32,
        /// Optional user URL.
        #[arg(long)]
        user: Option<String>,
    },
}

/// Tool name for payroll command.
pub fn payroll_tool_name(command: &PayrollCommands) -> &'static str {
    match command {
        PayrollCommands::Periods { .. } => "payroll.periods",
        PayrollCommands::Period { .. } => "payroll.period",
        PayrollCommands::MarkPaymentPaid { .. } => "payroll.mark-payment-paid",
        PayrollCommands::MarkPaymentUnpaid { .. } => "payroll.mark-payment-unpaid",
    }
}

/// Tool name for payroll profile command.
pub fn payroll_profile_tool_name(command: &PayrollProfileCommands) -> &'static str {
    match command {
        PayrollProfileCommands::List { .. } => "payroll-profiles.list",
    }
}

/// Runs payroll command.
pub async fn run_payroll(
    command: &PayrollCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        PayrollCommands::Periods { year } => {
            let value = ctx
                .client()
                .get_json(&format!("payroll/{year}"), &[])
                .await?;
            ctx.emit_success("payroll.periods", &value, start)
        }
        PayrollCommands::Period { year, period } => {
            let value = ctx
                .client()
                .get_json(&format!("payroll/{year}/{period}"), &[])
                .await?;
            ctx.emit_success("payroll.period", &value, start)
        }
        PayrollCommands::MarkPaymentPaid { year, payment_date } => {
            ctx.require_writes_allowed()?;
            let value = ctx
                .client()
                .put_json(
                    &format!(
                        "payroll/{}/payments/{}/mark_as_paid",
                        year,
                        encode_path_segment(payment_date)
                    ),
                    &serde_json::json!({}),
                    true,
                )
                .await?;
            ctx.emit_success("payroll.mark-payment-paid", &value, start)
        }
        PayrollCommands::MarkPaymentUnpaid { year, payment_date } => {
            ctx.require_writes_allowed()?;
            let value = ctx
                .client()
                .get_json(
                    &format!(
                        "payroll/{}/payments/{}/mark_as_unpaid",
                        year,
                        encode_path_segment(payment_date)
                    ),
                    &[],
                )
                .await?;
            ctx.emit_success("payroll.mark-payment-unpaid", &value, start)
        }
    }
}

/// Runs payroll profile command.
pub async fn run_payroll_profiles(
    command: &PayrollProfileCommands,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    match command {
        PayrollProfileCommands::List { year, user } => {
            let mut query = Vec::new();
            if let Some(user) = user
                && !user.trim().is_empty()
            {
                query.push(("user".to_string(), user.clone()));
            }
            let value = ctx
                .client()
                .get_json(&format!("payroll_profiles/{year}"), &query)
                .await?;
            ctx.emit_success("payroll-profiles.list", &value, start)
        }
    }
}

fn encode_path_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}
