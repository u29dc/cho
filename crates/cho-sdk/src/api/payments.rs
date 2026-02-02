//! Payments API: list and get payments.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{PaginatedResponse, PaginationParams};
use crate::http::request::ListParams;
use crate::models::common::Pagination;
use crate::models::payment::{Payment, Payments};

impl PaginatedResponse for Payments {
    type Item = Payment;

    fn into_items(self) -> Vec<Payment> {
        self.payments.unwrap_or_default()
    }

    fn pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }
}

/// API handle for payment operations.
pub struct PaymentsApi<'a> {
    client: &'a XeroClient,
}

impl<'a> PaymentsApi<'a> {
    /// Creates a new payments API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists payments with optional filtering and pagination.
    pub async fn list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<Payment>> {
        self.client
            .get_all_pages::<Payments>("Payments", params, pagination)
            .await
    }

    /// Gets a single payment by ID.
    pub async fn get(&self, id: Uuid) -> Result<Payment> {
        let response: Payments = self.client.get(&format!("Payments/{id}"), &[]).await?;

        response
            .payments
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "Payment".to_string(),
                id: id.to_string(),
            })
    }
}
