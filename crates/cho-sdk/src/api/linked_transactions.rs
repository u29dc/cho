//! Linked Transactions API: list and get linked transactions.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{PaginatedResponse, PaginationParams};
use crate::http::request::ListParams;
use crate::models::common::Pagination;
use crate::models::linked_transaction::{LinkedTransaction, LinkedTransactions};

impl PaginatedResponse for LinkedTransactions {
    type Item = LinkedTransaction;

    fn into_items(self) -> Vec<LinkedTransaction> {
        self.linked_transactions.unwrap_or_default()
    }

    fn pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }
}

/// API handle for linked transaction operations.
pub struct LinkedTransactionsApi<'a> {
    client: &'a XeroClient,
}

impl<'a> LinkedTransactionsApi<'a> {
    /// Creates a new linked transactions API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists linked transactions with optional filtering and pagination.
    pub async fn list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<LinkedTransaction>> {
        self.client
            .get_all_pages::<LinkedTransactions>("LinkedTransactions", params, pagination)
            .await
    }

    /// Gets a single linked transaction by ID.
    pub async fn get(&self, id: Uuid) -> Result<LinkedTransaction> {
        let response: LinkedTransactions = self
            .client
            .get(&format!("LinkedTransactions/{id}"), &[])
            .await?;

        response
            .linked_transactions
            .and_then(|mut v| if v.is_empty() { None } else { Some(v.remove(0)) })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "LinkedTransaction".to_string(),
                id: id.to_string(),
            })
    }
}
