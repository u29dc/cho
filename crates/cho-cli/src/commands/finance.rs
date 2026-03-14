//! Finance-focused commands built from shared SDK helpers.

use std::time::Instant;

use cho_sdk::error::{ChoSdkError, Result};
use cho_sdk::liabilities::{ReconcileOptions, TaxCalendarOptions};
use clap::{Args, Subcommand};

use crate::context::CliContext;

/// Tax calendar command args.
#[derive(Debug, Clone, Args)]
pub struct TaxCalendarArgs {
    /// Optional self-assessment user to merge into the company calendar.
    #[arg(long)]
    pub user: Option<String>,
    /// Accepted for explicitness when merging personal obligations.
    #[arg(long)]
    pub merge_personal: bool,
    /// Optional payroll year override.
    #[arg(long)]
    pub payroll_year: Option<i32>,
}

/// Taxes command group.
#[derive(Debug, Clone, Subcommand)]
pub enum TaxesCommands {
    /// Reconcile likely HMRC payments against known liabilities.
    Reconcile {
        /// Optional self-assessment user to include.
        #[arg(long)]
        user: Option<String>,
        /// Optional payroll year override.
        #[arg(long)]
        payroll_year: Option<i32>,
        /// Match window around due dates.
        #[arg(long, default_value_t = 45)]
        match_window_days: i64,
    },
}

/// Tool name for taxes command.
pub fn taxes_tool_name(command: &TaxesCommands) -> &'static str {
    match command {
        TaxesCommands::Reconcile { .. } => "taxes.reconcile",
    }
}

/// Runs top-level `tax-calendar`.
pub async fn run_tax_calendar(
    args: &TaxCalendarArgs,
    ctx: &CliContext,
    start: Instant,
) -> Result<()> {
    if args.merge_personal && args.user.is_none() {
        return Err(ChoSdkError::Config {
            message: "--merge-personal requires --user <id>".to_string(),
        });
    }

    let calendar = ctx
        .client()
        .liabilities()
        .tax_calendar(&TaxCalendarOptions {
            user: args.user.clone(),
            payroll_year: args.payroll_year,
        })
        .await?;
    ctx.emit_success("tax-calendar.get", &calendar, start)
}

/// Runs `taxes` command group.
pub async fn run_taxes(command: &TaxesCommands, ctx: &CliContext, start: Instant) -> Result<()> {
    match command {
        TaxesCommands::Reconcile {
            user,
            payroll_year,
            match_window_days,
        } => {
            let report = ctx
                .client()
                .liabilities()
                .reconcile_hmrc(&ReconcileOptions {
                    user: user.clone(),
                    payroll_year: *payroll_year,
                    match_window_days: *match_window_days,
                })
                .await?;
            ctx.emit_success("taxes.reconcile", &report, start)
        }
    }
}
