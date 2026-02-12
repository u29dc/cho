//! Report commands: balance-sheet, pnl, trial-balance, aged-payables, aged-receivables.

use std::time::Instant;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ReportParams;

use crate::commands::utils::validate_date;
use crate::context::CliContext;

/// Report subcommands.
#[derive(Debug, Subcommand)]
pub enum ReportCommands {
    /// Balance Sheet report.
    BalanceSheet {
        /// Report date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,

        /// Number of periods to compare.
        #[arg(long)]
        periods: Option<u32>,

        /// Timeframe: MONTH, QUARTER, or YEAR.
        #[arg(long)]
        timeframe: Option<String>,
    },
    /// Profit and Loss report.
    Pnl {
        /// From date (YYYY-MM-DD).
        #[arg(long)]
        from: Option<String>,

        /// To date (YYYY-MM-DD).
        #[arg(long)]
        to: Option<String>,

        /// Number of periods to compare.
        #[arg(long)]
        periods: Option<u32>,

        /// Timeframe: MONTH, QUARTER, or YEAR.
        #[arg(long)]
        timeframe: Option<String>,
    },
    /// Trial Balance report.
    TrialBalance {
        /// Report date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,
    },
    /// Aged Payables report.
    AgedPayables {
        /// Contact ID to filter by.
        #[arg(long)]
        contact: Option<String>,

        /// Report date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,
    },
    /// Aged Receivables report.
    AgedReceivables {
        /// Contact ID to filter by.
        #[arg(long)]
        contact: Option<String>,

        /// Report date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,
    },
}

/// Returns the tool name for a report subcommand.
pub fn tool_name(cmd: &ReportCommands) -> &'static str {
    match cmd {
        ReportCommands::BalanceSheet { .. } => "reports.balance-sheet",
        ReportCommands::Pnl { .. } => "reports.pnl",
        ReportCommands::TrialBalance { .. } => "reports.trial-balance",
        ReportCommands::AgedPayables { .. } => "reports.aged-payables",
        ReportCommands::AgedReceivables { .. } => "reports.aged-receivables",
    }
}

/// Runs a report subcommand.
pub async fn run(
    cmd: &ReportCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        ReportCommands::BalanceSheet {
            date,
            periods,
            timeframe,
        } => {
            if let Some(d) = date {
                validate_date(d, "--date")?;
            }
            let params = ReportParams {
                date: date.clone(),
                periods: *periods,
                timeframe: timeframe.clone(),
                ..Default::default()
            };
            let report = ctx.client().reports().balance_sheet_raw(&params).await?;
            ctx.emit_success("reports.balance-sheet", &report, start)?;
            Ok(())
        }
        ReportCommands::Pnl {
            from,
            to,
            periods,
            timeframe,
        } => {
            if let Some(d) = from {
                validate_date(d, "--from")?;
            }
            if let Some(d) = to {
                validate_date(d, "--to")?;
            }
            let params = ReportParams {
                from_date: from.clone(),
                to_date: to.clone(),
                periods: *periods,
                timeframe: timeframe.clone(),
                ..Default::default()
            };
            let report = ctx.client().reports().profit_and_loss_raw(&params).await?;
            ctx.emit_success("reports.pnl", &report, start)?;
            Ok(())
        }
        ReportCommands::TrialBalance { date } => {
            if let Some(d) = date {
                validate_date(d, "--date")?;
            }
            let params = ReportParams {
                date: date.clone(),
                ..Default::default()
            };
            let report = ctx.client().reports().trial_balance_raw(&params).await?;
            ctx.emit_success("reports.trial-balance", &report, start)?;
            Ok(())
        }
        ReportCommands::AgedPayables { contact, date } => {
            if let Some(c) = contact {
                c.parse::<Uuid>()
                    .map_err(|_| cho_sdk::error::ChoSdkError::Config {
                        message: format!("Invalid --contact UUID: \"{c}\""),
                    })?;
            }
            if let Some(d) = date {
                validate_date(d, "--date")?;
            }
            let params = ReportParams {
                contact_id: contact.clone(),
                date: date.clone(),
                ..Default::default()
            };
            let report = ctx.client().reports().aged_payables(&params).await?;
            ctx.emit_success("reports.aged-payables", &report, start)?;
            Ok(())
        }
        ReportCommands::AgedReceivables { contact, date } => {
            if let Some(c) = contact {
                c.parse::<Uuid>()
                    .map_err(|_| cho_sdk::error::ChoSdkError::Config {
                        message: format!("Invalid --contact UUID: \"{c}\""),
                    })?;
            }
            if let Some(d) = date {
                validate_date(d, "--date")?;
            }
            let params = ReportParams {
                contact_id: contact.clone(),
                date: date.clone(),
                ..Default::default()
            };
            let report = ctx.client().reports().aged_receivables(&params).await?;
            ctx.emit_success("reports.aged-receivables", &report, start)?;
            Ok(())
        }
    }
}
