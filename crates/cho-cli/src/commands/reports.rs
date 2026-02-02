//! Report commands: balance-sheet, pnl, trial-balance, aged-payables, aged-receivables.

use clap::Subcommand;

use cho_sdk::http::request::ReportParams;

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

/// Runs a report subcommand.
pub async fn run(cmd: &ReportCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        ReportCommands::BalanceSheet {
            date,
            periods,
            timeframe,
        } => {
            let params = ReportParams {
                date: date.clone(),
                periods: *periods,
                timeframe: timeframe.clone(),
                ..Default::default()
            };
            let report = ctx.client().reports().balance_sheet_raw(&params).await?;
            let output = ctx.format_output(&report)?;
            println!("{output}");
            Ok(())
        }
        ReportCommands::Pnl {
            from,
            to,
            periods,
            timeframe,
        } => {
            let params = ReportParams {
                from_date: from.clone(),
                to_date: to.clone(),
                periods: *periods,
                timeframe: timeframe.clone(),
                ..Default::default()
            };
            let report = ctx.client().reports().profit_and_loss_raw(&params).await?;
            let output = ctx.format_output(&report)?;
            println!("{output}");
            Ok(())
        }
        ReportCommands::TrialBalance { date } => {
            let params = ReportParams {
                date: date.clone(),
                ..Default::default()
            };
            let report = ctx.client().reports().trial_balance_raw(&params).await?;
            let output = ctx.format_output(&report)?;
            println!("{output}");
            Ok(())
        }
        ReportCommands::AgedPayables { contact, date } => {
            let params = ReportParams {
                contact_id: contact.clone(),
                date: date.clone(),
                ..Default::default()
            };
            let report = ctx.client().reports().aged_payables(&params).await?;
            let output = ctx.format_output(&report)?;
            println!("{output}");
            Ok(())
        }
        ReportCommands::AgedReceivables { contact, date } => {
            let params = ReportParams {
                contact_id: contact.clone(),
                date: date.clone(),
                ..Default::default()
            };
            let report = ctx.client().reports().aged_receivables(&params).await?;
            let output = ctx.format_output(&report)?;
            println!("{output}");
            Ok(())
        }
    }
}
