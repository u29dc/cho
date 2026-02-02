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
use crate::http::pagination::{ListResult, PaginationParams};
use crate::http::request::{ListParams, ReportParams};
use crate::models::account::Account;
use crate::models::bank_transaction::BankTransaction;
use crate::models::budget::Budget;
use crate::models::connection::Connection;
use crate::models::contact::Contact;
use crate::models::credit_note::CreditNote;
use crate::models::currency::Currency;
use crate::models::invoice::Invoice;
use crate::models::item::Item;
use crate::models::linked_transaction::LinkedTransaction;
use crate::models::manual_journal::ManualJournal;
use crate::models::organisation::Organisation;
use crate::models::overpayment::Overpayment;
use crate::models::payment::Payment;
use crate::models::prepayment::Prepayment;
use crate::models::purchase_order::PurchaseOrder;
use crate::models::quote::Quote;
use crate::models::repeating_invoice::RepeatingInvoice;
use crate::models::report::{BalanceSheetReport, ProfitAndLossReport, Report, TrialBalanceReport};
use crate::models::tax_rate::TaxRate;
use crate::models::tracking_category::TrackingCategory;

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
    ) -> Result<ListResult<Invoice>> {
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

    /// Creates a new invoice (blocking).
    pub fn invoices_create(
        &self,
        invoice: &Invoice,
        idempotency_key: Option<&str>,
    ) -> Result<Invoice> {
        self.runtime
            .block_on(self.inner.invoices().create(invoice, idempotency_key))
    }

    /// Updates an existing invoice (blocking).
    pub fn invoices_update(
        &self,
        id: Uuid,
        invoice: &Invoice,
        idempotency_key: Option<&str>,
    ) -> Result<Invoice> {
        self.runtime
            .block_on(self.inner.invoices().update(id, invoice, idempotency_key))
    }

    // --- Contacts ---

    /// Lists contacts (blocking).
    pub fn contacts_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<Contact>> {
        self.runtime
            .block_on(self.inner.contacts().list(params, pagination))
    }

    /// Gets a single contact by ID (blocking).
    pub fn contacts_get(&self, id: Uuid) -> Result<Contact> {
        self.runtime.block_on(self.inner.contacts().get(id))
    }

    /// Creates a new contact (blocking).
    pub fn contacts_create(
        &self,
        contact: &Contact,
        idempotency_key: Option<&str>,
    ) -> Result<Contact> {
        self.runtime
            .block_on(self.inner.contacts().create(contact, idempotency_key))
    }

    /// Updates an existing contact (blocking).
    pub fn contacts_update(
        &self,
        id: Uuid,
        contact: &Contact,
        idempotency_key: Option<&str>,
    ) -> Result<Contact> {
        self.runtime
            .block_on(self.inner.contacts().update(id, contact, idempotency_key))
    }

    /// Searches contacts (blocking).
    pub fn contacts_search(
        &self,
        term: &str,
        pagination: &PaginationParams,
    ) -> Result<ListResult<Contact>> {
        self.runtime
            .block_on(self.inner.contacts().search(term, pagination))
    }

    // --- Payments ---

    /// Lists payments (blocking).
    pub fn payments_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<Payment>> {
        self.runtime
            .block_on(self.inner.payments().list(params, pagination))
    }

    /// Gets a single payment by ID (blocking).
    pub fn payments_get(&self, id: Uuid) -> Result<Payment> {
        self.runtime.block_on(self.inner.payments().get(id))
    }

    /// Creates a new payment (blocking).
    pub fn payments_create(
        &self,
        payment: &Payment,
        idempotency_key: Option<&str>,
    ) -> Result<Payment> {
        self.runtime
            .block_on(self.inner.payments().create(payment, idempotency_key))
    }

    /// Deletes a payment (blocking).
    pub fn payments_delete(&self, id: Uuid, idempotency_key: Option<&str>) -> Result<Payment> {
        self.runtime
            .block_on(self.inner.payments().delete(id, idempotency_key))
    }

    // --- Bank Transactions ---

    /// Lists bank transactions (blocking).
    pub fn bank_transactions_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<BankTransaction>> {
        self.runtime
            .block_on(self.inner.bank_transactions().list(params, pagination))
    }

    /// Gets a single bank transaction by ID (blocking).
    pub fn bank_transactions_get(&self, id: Uuid) -> Result<BankTransaction> {
        self.runtime
            .block_on(self.inner.bank_transactions().get(id))
    }

    /// Creates a new bank transaction (blocking).
    pub fn bank_transactions_create(
        &self,
        transaction: &BankTransaction,
        idempotency_key: Option<&str>,
    ) -> Result<BankTransaction> {
        self.runtime.block_on(
            self.inner
                .bank_transactions()
                .create(transaction, idempotency_key),
        )
    }

    /// Updates an existing bank transaction (blocking).
    pub fn bank_transactions_update(
        &self,
        id: Uuid,
        transaction: &BankTransaction,
        idempotency_key: Option<&str>,
    ) -> Result<BankTransaction> {
        self.runtime.block_on(self.inner.bank_transactions().update(
            id,
            transaction,
            idempotency_key,
        ))
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

    // --- Credit Notes ---

    /// Lists credit notes (blocking).
    pub fn credit_notes_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<CreditNote>> {
        self.runtime
            .block_on(self.inner.credit_notes().list(params, pagination))
    }

    /// Gets a single credit note by ID (blocking).
    pub fn credit_notes_get(&self, id: Uuid) -> Result<CreditNote> {
        self.runtime.block_on(self.inner.credit_notes().get(id))
    }

    // --- Quotes ---

    /// Lists quotes (blocking).
    pub fn quotes_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<Quote>> {
        self.runtime
            .block_on(self.inner.quotes().list(params, pagination))
    }

    /// Gets a single quote by ID (blocking).
    pub fn quotes_get(&self, id: Uuid) -> Result<Quote> {
        self.runtime.block_on(self.inner.quotes().get(id))
    }

    // --- Purchase Orders ---

    /// Lists purchase orders (blocking).
    pub fn purchase_orders_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<PurchaseOrder>> {
        self.runtime
            .block_on(self.inner.purchase_orders().list(params, pagination))
    }

    /// Gets a single purchase order by ID (blocking).
    pub fn purchase_orders_get(&self, id: Uuid) -> Result<PurchaseOrder> {
        self.runtime.block_on(self.inner.purchase_orders().get(id))
    }

    // --- Items ---

    /// Lists items (blocking).
    pub fn items_list(&self, params: &ListParams) -> Result<Vec<Item>> {
        self.runtime.block_on(self.inner.items().list(params))
    }

    /// Gets a single item by ID (blocking).
    pub fn items_get(&self, id: Uuid) -> Result<Item> {
        self.runtime.block_on(self.inner.items().get(id))
    }

    // --- Tax Rates ---

    /// Lists tax rates (blocking).
    pub fn tax_rates_list(&self, params: &ListParams) -> Result<Vec<TaxRate>> {
        self.runtime.block_on(self.inner.tax_rates().list(params))
    }

    // --- Currencies ---

    /// Lists currencies (blocking).
    pub fn currencies_list(&self, params: &ListParams) -> Result<Vec<Currency>> {
        self.runtime.block_on(self.inner.currencies().list(params))
    }

    // --- Tracking Categories ---

    /// Lists tracking categories (blocking).
    pub fn tracking_categories_list(&self, params: &ListParams) -> Result<Vec<TrackingCategory>> {
        self.runtime
            .block_on(self.inner.tracking_categories().list(params))
    }

    /// Gets a single tracking category by ID (blocking).
    pub fn tracking_categories_get(&self, id: Uuid) -> Result<TrackingCategory> {
        self.runtime
            .block_on(self.inner.tracking_categories().get(id))
    }

    // --- Organisations ---

    /// Gets the organisation details (blocking).
    pub fn organisations_get(&self) -> Result<Organisation> {
        self.runtime.block_on(self.inner.organisations().get())
    }

    // --- Manual Journals ---

    /// Lists manual journals (blocking).
    pub fn manual_journals_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<ManualJournal>> {
        self.runtime
            .block_on(self.inner.manual_journals().list(params, pagination))
    }

    /// Gets a single manual journal by ID (blocking).
    pub fn manual_journals_get(&self, id: Uuid) -> Result<ManualJournal> {
        self.runtime.block_on(self.inner.manual_journals().get(id))
    }

    // --- Prepayments ---

    /// Lists prepayments (blocking).
    pub fn prepayments_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<Prepayment>> {
        self.runtime
            .block_on(self.inner.prepayments().list(params, pagination))
    }

    /// Gets a single prepayment by ID (blocking).
    pub fn prepayments_get(&self, id: Uuid) -> Result<Prepayment> {
        self.runtime.block_on(self.inner.prepayments().get(id))
    }

    // --- Overpayments ---

    /// Lists overpayments (blocking).
    pub fn overpayments_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<Overpayment>> {
        self.runtime
            .block_on(self.inner.overpayments().list(params, pagination))
    }

    /// Gets a single overpayment by ID (blocking).
    pub fn overpayments_get(&self, id: Uuid) -> Result<Overpayment> {
        self.runtime.block_on(self.inner.overpayments().get(id))
    }

    // --- Linked Transactions ---

    /// Lists linked transactions (blocking).
    pub fn linked_transactions_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<LinkedTransaction>> {
        self.runtime
            .block_on(self.inner.linked_transactions().list(params, pagination))
    }

    /// Gets a single linked transaction by ID (blocking).
    pub fn linked_transactions_get(&self, id: Uuid) -> Result<LinkedTransaction> {
        self.runtime
            .block_on(self.inner.linked_transactions().get(id))
    }

    // --- Budgets ---

    /// Lists budgets (blocking).
    pub fn budgets_list(&self, params: &ListParams) -> Result<Vec<Budget>> {
        self.runtime.block_on(self.inner.budgets().list(params))
    }

    /// Gets a single budget by ID (blocking).
    pub fn budgets_get(&self, id: Uuid) -> Result<Budget> {
        self.runtime.block_on(self.inner.budgets().get(id))
    }

    // --- Repeating Invoices ---

    /// Lists repeating invoices (blocking).
    pub fn repeating_invoices_list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<RepeatingInvoice>> {
        self.runtime
            .block_on(self.inner.repeating_invoices().list(params, pagination))
    }

    /// Gets a single repeating invoice by ID (blocking).
    pub fn repeating_invoices_get(&self, id: Uuid) -> Result<RepeatingInvoice> {
        self.runtime
            .block_on(self.inner.repeating_invoices().get(id))
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
