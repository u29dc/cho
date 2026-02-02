//! Currencies API: list currencies.

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::request::ListParams;
use crate::models::currency::{Currencies, Currency};

/// API handle for currency operations.
pub struct CurrenciesApi<'a> {
    client: &'a XeroClient,
}

impl<'a> CurrenciesApi<'a> {
    /// Creates a new currencies API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists all currencies configured for the organisation.
    ///
    /// The Currencies endpoint is not paginated — it returns all currencies
    /// in a single response.
    pub async fn list(&self, params: &ListParams) -> Result<Vec<Currency>> {
        let query = params.to_query_pairs();
        let response: Currencies = self.client.get("Currencies", &query).await?;
        Ok(response.currencies.unwrap_or_default())
    }
}
