//! Payments API: list, get, and create payments.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{ListResult, PaginatedResponse, PaginationParams};
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
    ) -> Result<ListResult<Payment>> {
        self.client
            .get_all_pages::<Payments>("Payments", params, pagination)
            .await
    }

    /// Creates a new payment.
    ///
    /// Payments in Xero are create-only (cannot be updated, only deleted).
    pub async fn create(
        &self,
        payment: &Payment,
        idempotency_key: Option<&str>,
    ) -> Result<Payment> {
        let wrapper = Payments {
            payments: Some(vec![payment.clone()]),
            pagination: None,
            warnings: None,
        };

        let response: Payments = self
            .client
            .put("Payments", &wrapper, idempotency_key)
            .await?;

        response
            .payments
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::Parse {
                message: "No payment returned in create response".to_string(),
            })
    }

    /// Deletes a payment (sets status to DELETED).
    ///
    /// Xero payments cannot be updated, only deleted by POSTing with
    /// `Status: "DELETED"`.
    pub async fn delete(&self, id: Uuid, idempotency_key: Option<&str>) -> Result<Payment> {
        let payment = Payment {
            status: Some(crate::models::enums::PaymentStatus::Deleted),
            ..Default::default()
        };

        let wrapper = Payments {
            payments: Some(vec![payment]),
            pagination: None,
            warnings: None,
        };

        let response: Payments = self
            .client
            .post(&format!("Payments/{id}"), &wrapper, idempotency_key)
            .await?;

        response
            .payments
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::Parse {
                message: "No payment returned in delete response".to_string(),
            })
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
