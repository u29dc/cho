//! Reports API: balance sheet, profit & loss, trial balance, and aged reports.

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::request::ReportParams;
use crate::models::report::{
    BalanceSheetReport, ProfitAndLossReport, Report, Reports, TrialBalanceReport,
};

/// API handle for report operations.
pub struct ReportsApi<'a> {
    client: &'a XeroClient,
}

impl<'a> ReportsApi<'a> {
    /// Creates a new reports API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Fetches the Balance Sheet report (raw tabular format).
    pub async fn balance_sheet_raw(&self, params: &ReportParams) -> Result<Report> {
        let query = params.to_query_pairs();
        let response: Reports = self.client.get("Reports/BalanceSheet", &query).await?;
        first_report(response, "BalanceSheet")
    }

    /// Fetches the Balance Sheet report as a typed struct.
    pub async fn balance_sheet(&self, params: &ReportParams) -> Result<BalanceSheetReport> {
        let raw = self.balance_sheet_raw(params).await?;
        BalanceSheetReport::from_report(&raw).ok_or_else(|| crate::error::ChoSdkError::Parse {
            message: "Failed to parse Balance Sheet report structure".to_string(),
        })
    }

    /// Fetches the Profit and Loss report (raw tabular format).
    pub async fn profit_and_loss_raw(&self, params: &ReportParams) -> Result<Report> {
        let query = params.to_query_pairs();
        let response: Reports = self.client.get("Reports/ProfitAndLoss", &query).await?;
        first_report(response, "ProfitAndLoss")
    }

    /// Fetches the Profit and Loss report as a typed struct.
    pub async fn profit_and_loss(&self, params: &ReportParams) -> Result<ProfitAndLossReport> {
        let raw = self.profit_and_loss_raw(params).await?;
        ProfitAndLossReport::from_report(&raw).ok_or_else(|| crate::error::ChoSdkError::Parse {
            message: "Failed to parse Profit and Loss report structure".to_string(),
        })
    }

    /// Fetches the Trial Balance report (raw tabular format).
    pub async fn trial_balance_raw(&self, params: &ReportParams) -> Result<Report> {
        let query = params.to_query_pairs();
        let response: Reports = self.client.get("Reports/TrialBalance", &query).await?;
        first_report(response, "TrialBalance")
    }

    /// Fetches the Trial Balance report as a typed struct.
    pub async fn trial_balance(&self, params: &ReportParams) -> Result<TrialBalanceReport> {
        let raw = self.trial_balance_raw(params).await?;
        TrialBalanceReport::from_report(&raw).ok_or_else(|| crate::error::ChoSdkError::Parse {
            message: "Failed to parse Trial Balance report structure".to_string(),
        })
    }

    /// Fetches the Aged Payables by Contact report.
    pub async fn aged_payables(&self, params: &ReportParams) -> Result<Report> {
        let query = params.to_query_pairs();
        let response: Reports = self
            .client
            .get("Reports/AgedPayablesByContact", &query)
            .await?;
        first_report(response, "AgedPayablesByContact")
    }

    /// Fetches the Aged Receivables by Contact report.
    pub async fn aged_receivables(&self, params: &ReportParams) -> Result<Report> {
        let query = params.to_query_pairs();
        let response: Reports = self
            .client
            .get("Reports/AgedReceivablesByContact", &query)
            .await?;
        first_report(response, "AgedReceivablesByContact")
    }
}

/// Extracts the first report from a Reports collection response.
fn first_report(response: Reports, name: &str) -> Result<Report> {
    response
        .reports
        .and_then(|mut v| {
            if v.is_empty() {
                None
            } else {
                Some(v.remove(0))
            }
        })
        .ok_or_else(|| crate::error::ChoSdkError::NotFound {
            resource: "Report".to_string(),
            id: name.to_string(),
        })
}
