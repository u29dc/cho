//! Tax Rates API: list tax rates.

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::request::ListParams;
use crate::models::tax_rate::{TaxRate, TaxRates};

/// API handle for tax rate operations.
pub struct TaxRatesApi<'a> {
    client: &'a XeroClient,
}

impl<'a> TaxRatesApi<'a> {
    /// Creates a new tax rates API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists all tax rates.
    ///
    /// The TaxRates endpoint is not paginated — it returns all tax rates
    /// in a single response.
    pub async fn list(&self, params: &ListParams) -> Result<Vec<TaxRate>> {
        let query = params.to_query_pairs();
        let response: TaxRates = self.client.get("TaxRates", &query).await?;
        Ok(response.tax_rates.unwrap_or_default())
    }
}
