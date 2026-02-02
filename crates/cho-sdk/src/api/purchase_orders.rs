//! Purchase Orders API: list and get purchase orders.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{PaginatedResponse, PaginationParams};
use crate::http::request::ListParams;
use crate::models::common::Pagination;
use crate::models::purchase_order::{PurchaseOrder, PurchaseOrders};

impl PaginatedResponse for PurchaseOrders {
    type Item = PurchaseOrder;

    fn into_items(self) -> Vec<PurchaseOrder> {
        self.purchase_orders.unwrap_or_default()
    }

    fn pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }
}

/// API handle for purchase order operations.
pub struct PurchaseOrdersApi<'a> {
    client: &'a XeroClient,
}

impl<'a> PurchaseOrdersApi<'a> {
    /// Creates a new purchase orders API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists purchase orders with optional filtering and pagination.
    pub async fn list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<PurchaseOrder>> {
        self.client
            .get_all_pages::<PurchaseOrders>("PurchaseOrders", params, pagination)
            .await
    }

    /// Gets a single purchase order by ID.
    pub async fn get(&self, id: Uuid) -> Result<PurchaseOrder> {
        let response: PurchaseOrders =
            self.client.get(&format!("PurchaseOrders/{id}"), &[]).await?;

        response
            .purchase_orders
            .and_then(|mut v| if v.is_empty() { None } else { Some(v.remove(0)) })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "PurchaseOrder".to_string(),
                id: id.to_string(),
            })
    }
}
