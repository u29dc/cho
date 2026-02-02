//! Tracking Categories API: list and get tracking categories.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::request::ListParams;
use crate::models::tracking_category::{TrackingCategories, TrackingCategory};

/// API handle for tracking category operations.
pub struct TrackingCategoriesApi<'a> {
    client: &'a XeroClient,
}

impl<'a> TrackingCategoriesApi<'a> {
    /// Creates a new tracking categories API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists all tracking categories.
    ///
    /// The TrackingCategories endpoint is not paginated — it returns all
    /// tracking categories in a single response.
    pub async fn list(&self, params: &ListParams) -> Result<Vec<TrackingCategory>> {
        let query = params.to_query_pairs();
        let response: TrackingCategories =
            self.client.get("TrackingCategories", &query).await?;
        Ok(response.tracking_categories.unwrap_or_default())
    }

    /// Gets a single tracking category by ID.
    pub async fn get(&self, id: Uuid) -> Result<TrackingCategory> {
        let response: TrackingCategories = self
            .client
            .get(&format!("TrackingCategories/{id}"), &[])
            .await?;

        response
            .tracking_categories
            .and_then(|mut v| if v.is_empty() { None } else { Some(v.remove(0)) })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "TrackingCategory".to_string(),
                id: id.to_string(),
            })
    }
}
