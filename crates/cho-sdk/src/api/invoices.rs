//! Invoices API: list and get invoices.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{PaginatedResponse, PaginationParams};
use crate::http::request::ListParams;
use crate::models::common::Pagination;
use crate::models::invoice::{Invoice, Invoices};

impl PaginatedResponse for Invoices {
    type Item = Invoice;

    fn into_items(self) -> Vec<Invoice> {
        self.invoices.unwrap_or_default()
    }

    fn pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }
}

/// API handle for invoice operations.
pub struct InvoicesApi<'a> {
    client: &'a XeroClient,
}

impl<'a> InvoicesApi<'a> {
    /// Creates a new invoices API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists invoices with optional filtering and pagination.
    pub async fn list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<Invoice>> {
        self.client
            .get_all_pages::<Invoices>("Invoices", params, pagination)
            .await
    }

    /// Gets a single invoice by ID.
    pub async fn get(&self, id: Uuid) -> Result<Invoice> {
        let response: Invoices = self.client.get(&format!("Invoices/{id}"), &[]).await?;

        response
            .invoices
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "Invoice".to_string(),
                id: id.to_string(),
            })
    }

    /// Gets a single invoice by invoice number.
    pub async fn get_by_number(&self, number: &str) -> Result<Invoice> {
        let params = ListParams::new().with_where(format!("InvoiceNumber==\"{number}\""));
        let query = params.to_query_pairs();

        let response: Invoices = self.client.get("Invoices", &query).await?;

        response
            .invoices
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "Invoice".to_string(),
                id: number.to_string(),
            })
    }
}
