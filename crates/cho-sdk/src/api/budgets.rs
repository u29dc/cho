//! Budgets API: list and get budgets.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::request::ListParams;
use crate::models::budget::{Budget, Budgets};

/// API handle for budget operations.
pub struct BudgetsApi<'a> {
    client: &'a XeroClient,
}

impl<'a> BudgetsApi<'a> {
    /// Creates a new budgets API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists all budgets.
    ///
    /// The Budgets endpoint is not paginated — it returns all budgets
    /// in a single response.
    pub async fn list(&self, params: &ListParams) -> Result<Vec<Budget>> {
        let query = params.to_query_pairs();
        let response: Budgets = self.client.get("Budgets", &query).await?;
        Ok(response.budgets.unwrap_or_default())
    }

    /// Gets a single budget by ID.
    pub async fn get(&self, id: Uuid) -> Result<Budget> {
        let response: Budgets = self.client.get(&format!("Budgets/{id}"), &[]).await?;

        response
            .budgets
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "Budget".to_string(),
                id: id.to_string(),
            })
    }
}
