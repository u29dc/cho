//! Items API: list and get items.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::request::ListParams;
use crate::models::item::{Item, Items};

/// API handle for item operations.
pub struct ItemsApi<'a> {
    client: &'a XeroClient,
}

impl<'a> ItemsApi<'a> {
    /// Creates a new items API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists all items.
    ///
    /// The Items endpoint is not paginated — it returns all items
    /// in a single response.
    pub async fn list(&self, params: &ListParams) -> Result<Vec<Item>> {
        let query = params.to_query_pairs();
        let response: Items = self.client.get("Items", &query).await?;
        Ok(response.items.unwrap_or_default())
    }

    /// Gets a single item by ID.
    pub async fn get(&self, id: Uuid) -> Result<Item> {
        let response: Items = self.client.get(&format!("Items/{id}"), &[]).await?;

        response
            .items
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "Item".to_string(),
                id: id.to_string(),
            })
    }

    /// Gets a single item by item code.
    pub async fn get_by_code(&self, code: &str) -> Result<Item> {
        let response: Items = self.client.get(&format!("Items/{code}"), &[]).await?;

        response
            .items
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "Item".to_string(),
                id: code.to_string(),
            })
    }
}
