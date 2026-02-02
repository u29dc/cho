//! Invoices API: list, get, create, and update invoices.

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

    /// Creates a new invoice.
    ///
    /// Xero uses PUT for creating invoices. The invoice is sent as the body
    /// and the response contains the created invoice with server-assigned fields.
    pub async fn create(
        &self,
        invoice: &Invoice,
        idempotency_key: Option<&str>,
    ) -> Result<Invoice> {
        let wrapper = Invoices {
            invoices: Some(vec![invoice.clone()]),
            pagination: None,
            warnings: None,
        };

        let response: Invoices = self
            .client
            .put("Invoices", &wrapper, idempotency_key)
            .await?;

        response
            .invoices
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::Parse {
                message: "No invoice returned in create response".to_string(),
            })
    }

    /// Updates an existing invoice.
    ///
    /// Xero uses POST for updating invoices. The invoice must include the
    /// `invoice_id` field to identify which invoice to update.
    pub async fn update(
        &self,
        id: Uuid,
        invoice: &Invoice,
        idempotency_key: Option<&str>,
    ) -> Result<Invoice> {
        let wrapper = Invoices {
            invoices: Some(vec![invoice.clone()]),
            pagination: None,
            warnings: None,
        };

        let response: Invoices = self
            .client
            .post(&format!("Invoices/{id}"), &wrapper, idempotency_key)
            .await?;

        response
            .invoices
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::Parse {
                message: "No invoice returned in update response".to_string(),
            })
    }

    /// Gets a single invoice by invoice number.
    ///
    /// The invoice number is sanitized to prevent OData filter injection.
    pub async fn get_by_number(&self, number: &str) -> Result<Invoice> {
        // Reject characters that could break the OData where filter
        if number.contains('"') || number.contains('\\') {
            return Err(crate::error::ChoSdkError::Parse {
                message: "Invalid invoice number: contains illegal characters (quotes or backslashes)".to_string(),
            });
        }
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
