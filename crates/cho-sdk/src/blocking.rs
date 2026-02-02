//! Synchronous (blocking) wrapper for the Xero SDK.
//!
//! Provides [`BlockingClient`] which wraps [`XeroClient`](crate::client::XeroClient)
//! and exposes synchronous versions of all API methods using an internal
//! `tokio::runtime::Runtime`.
//!
//! This is useful for callers that cannot use async/await (e.g., simple scripts,
//! synchronous CLI dispatch, or non-async contexts).

use uuid::Uuid;

use crate::client::{XeroClient, XeroClientBuilder};
use crate::error::Result;
use crate::http::pagination::PaginationParams;
use crate::http::request::{ListParams, ReportParams};
use crate::models::account::Account;
use crate::models::bank_transaction::BankTransaction;
use crate::models::connection::Connection;
use crate::models::contact::Contact;
use crate::models::invoice::Invoice;
use crate::models::payment::Payment;
use crate::models::report::{BalanceSheetReport, ProfitAndLossReport, Report, TrialBalanceReport};

/// A synchronous wrapper around [`XeroClient`].
///
/// All methods block the calling thread until the underlying async operation
/// completes. Uses its own `tokio::runtime::Runtime` internally.
///
/// # Example
///
/// ```rust,no_run
/// use cho_sdk::blocking::{BlockingClient, BlockingClientBuilderExt};
///
/// # fn example() -> cho_sdk::error::Result<()> {
/// let client = BlockingClient::builder()
///     .client_id("your-client-id")
///     .tenant_id("your-tenant-id")
///     .build_blocking()?;
///
/// let invoices = client.invoices_list(
///     &cho_sdk::http::request::ListParams::new(),
///     &cho_sdk::http::pagination::PaginationParams::default(),
/// )?;
/// # Ok(())
/// # }
/// ```
pub struct BlockingClient {
    /// The underlying async client.
    inner: XeroClient,

    /// Tokio runtime for blocking on async calls.
    runtime: tokio::runtime::Runtime,
}

impl BlockingClient {
    /// Returns a new builder for the blocking client.
    pub fn builder() -> XeroClientBuilder {
        XeroClient::builder()
    }

    /// Creates a blocking client from an existing async client.
    pub fn from_async(client: XeroClient) -> Result<Self> {
        let runtime =
            tokio::runtime::Runtime::new().map_err(|e| crate::error::ChoSdkError::Config {
                message: format!("Failed to create tokio runtime: {e}"),
            })?;

        Ok(Self {
            inner: client,
            runtime,
        })
    }

    /// Returns a reference to the underlying async client.
    pub fn inner(&self) -> &XeroClient {
        &self.inner
    }

    /// Returns a mutable reference to the underlying async client.
    pub fn inner_mut(&mut self) -> &mut XeroClient {
        &mut self.inner
    }

    // --- Invoices ---

    /// Lists invoices (blocking).
    pub fn invoices_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<Invoice>> {
        self.runtime
            .block_on(self.inner.invoices().list(params, pagination))
    }

    /// Gets a single invoice by ID (blocking).
    pub fn invoices_get(&self, id: Uuid) -> Result<Invoice> {
        self.runtime.block_on(self.inner.invoices().get(id))
    }

    /// Gets a single invoice by number (blocking).
    pub fn invoices_get_by_number(&self, number: &str) -> Result<Invoice> {
        self.runtime
            .block_on(self.inner.invoices().get_by_number(number))
    }

    // --- Contacts ---

    /// Lists contacts (blocking).
    pub fn contacts_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<Contact>> {
        self.runtime
            .block_on(self.inner.contacts().list(params, pagination))
    }

    /// Gets a single contact by ID (blocking).
    pub fn contacts_get(&self, id: Uuid) -> Result<Contact> {
        self.runtime.block_on(self.inner.contacts().get(id))
    }

    /// Searches contacts (blocking).
    pub fn contacts_search(
        &self,
        term: &str,
        pagination: &PaginationParams,
    ) -> Result<Vec<Contact>> {
        self.runtime
            .block_on(self.inner.contacts().search(term, pagination))
    }

    // --- Payments ---

    /// Lists payments (blocking).
    pub fn payments_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<Payment>> {
        self.runtime
            .block_on(self.inner.payments().list(params, pagination))
    }

    /// Gets a single payment by ID (blocking).
    pub fn payments_get(&self, id: Uuid) -> Result<Payment> {
        self.runtime.block_on(self.inner.payments().get(id))
    }

    // --- Bank Transactions ---

    /// Lists bank transactions (blocking).
    pub fn bank_transactions_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<BankTransaction>> {
        self.runtime
            .block_on(self.inner.bank_transactions().list(params, pagination))
    }

    /// Gets a single bank transaction by ID (blocking).
    pub fn bank_transactions_get(&self, id: Uuid) -> Result<BankTransaction> {
        self.runtime
            .block_on(self.inner.bank_transactions().get(id))
    }

    // --- Accounts ---

    /// Lists accounts (blocking).
    pub fn accounts_list(&self, params: &ListParams) -> Result<Vec<Account>> {
        self.runtime.block_on(self.inner.accounts().list(params))
    }

    // --- Reports ---

    /// Fetches the Balance Sheet report (blocking, typed).
    pub fn reports_balance_sheet(&self, params: &ReportParams) -> Result<BalanceSheetReport> {
        self.runtime
            .block_on(self.inner.reports().balance_sheet(params))
    }

    /// Fetches the Balance Sheet report (blocking, raw).
    pub fn reports_balance_sheet_raw(&self, params: &ReportParams) -> Result<Report> {
        self.runtime
            .block_on(self.inner.reports().balance_sheet_raw(params))
    }

    /// Fetches the Profit and Loss report (blocking, typed).
    pub fn reports_profit_and_loss(&self, params: &ReportParams) -> Result<ProfitAndLossReport> {
        self.runtime
            .block_on(self.inner.reports().profit_and_loss(params))
    }

    /// Fetches the Profit and Loss report (blocking, raw).
    pub fn reports_profit_and_loss_raw(&self, params: &ReportParams) -> Result<Report> {
        self.runtime
            .block_on(self.inner.reports().profit_and_loss_raw(params))
    }

    /// Fetches the Trial Balance report (blocking, typed).
    pub fn reports_trial_balance(&self, params: &ReportParams) -> Result<TrialBalanceReport> {
        self.runtime
            .block_on(self.inner.reports().trial_balance(params))
    }

    /// Fetches the Trial Balance report (blocking, raw).
    pub fn reports_trial_balance_raw(&self, params: &ReportParams) -> Result<Report> {
        self.runtime
            .block_on(self.inner.reports().trial_balance_raw(params))
    }

    /// Fetches the Aged Payables report (blocking, raw).
    pub fn reports_aged_payables(&self, params: &ReportParams) -> Result<Report> {
        self.runtime
            .block_on(self.inner.reports().aged_payables(params))
    }

    /// Fetches the Aged Receivables report (blocking, raw).
    pub fn reports_aged_receivables(&self, params: &ReportParams) -> Result<Report> {
        self.runtime
            .block_on(self.inner.reports().aged_receivables(params))
    }

    // --- Identity ---

    /// Lists connected organisations (blocking).
    pub fn identity_connections(&self) -> Result<Vec<Connection>> {
        self.runtime.block_on(self.inner.identity().connections())
    }

    // --- Auth ---

    /// Runs the PKCE login flow (blocking).
    pub fn auth_login_pkce(&self, port: u16) -> Result<()> {
        self.runtime.block_on(self.inner.auth().login_pkce(port))
    }

    /// Forces a token refresh (blocking).
    pub fn auth_refresh(&self) -> Result<()> {
        self.runtime.block_on(self.inner.auth().refresh())
    }

    /// Returns whether the client is currently authenticated (blocking).
    pub fn auth_is_authenticated(&self) -> bool {
        self.runtime.block_on(self.inner.auth().is_authenticated())
    }

    /// Logs out and clears stored tokens (blocking).
    pub fn auth_logout(&self) -> Result<()> {
        self.runtime.block_on(self.inner.auth().logout())
    }
}

impl std::fmt::Debug for BlockingClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockingClient")
            .field("inner", &self.inner)
            .finish()
    }
}

/// Extension trait for [`XeroClientBuilder`] to build a [`BlockingClient`].
pub trait BlockingClientBuilderExt {
    /// Builds a [`BlockingClient`] instead of an async [`XeroClient`].
    fn build_blocking(self) -> Result<BlockingClient>;
}

impl BlockingClientBuilderExt for XeroClientBuilder {
    fn build_blocking(self) -> Result<BlockingClient> {
        let client = self.build()?;
        BlockingClient::from_async(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocking_client_builder() {
        let client = BlockingClient::builder()
            .client_id("test-id")
            .tenant_id("test-tenant")
            .build_blocking()
            .unwrap();
        assert_eq!(client.inner().tenant_id(), "test-tenant");
    }

    #[test]
    fn blocking_client_debug() {
        let client = BlockingClient::builder()
            .client_id("test")
            .tenant_id("tenant")
            .build_blocking()
            .unwrap();
        let debug = format!("{client:?}");
        assert!(debug.contains("BlockingClient"));
    }
}
