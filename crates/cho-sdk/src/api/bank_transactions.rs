//! Bank Transactions API: list and get bank transactions.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{PaginatedResponse, PaginationParams};
use crate::http::request::ListParams;
use crate::models::bank_transaction::{BankTransaction, BankTransactions};
use crate::models::common::Pagination;

impl PaginatedResponse for BankTransactions {
    type Item = BankTransaction;

    fn into_items(self) -> Vec<BankTransaction> {
        self.bank_transactions.unwrap_or_default()
    }

    fn pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }
}

/// API handle for bank transaction operations.
pub struct BankTransactionsApi<'a> {
    client: &'a XeroClient,
}

impl<'a> BankTransactionsApi<'a> {
    /// Creates a new bank transactions API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists bank transactions with optional filtering and pagination.
    pub async fn list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<BankTransaction>> {
        self.client
            .get_all_pages::<BankTransactions>("BankTransactions", params, pagination)
            .await
    }

    /// Gets a single bank transaction by ID.
    pub async fn get(&self, id: Uuid) -> Result<BankTransaction> {
        let response: BankTransactions = self
            .client
            .get(&format!("BankTransactions/{id}"), &[])
            .await?;

        response
            .bank_transactions
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "BankTransaction".to_string(),
                id: id.to_string(),
            })
    }
}
