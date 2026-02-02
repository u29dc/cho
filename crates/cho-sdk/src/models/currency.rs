//! Currency model for the Xero Currencies API.

use serde::{Deserialize, Serialize};

use super::common::{Pagination, Warning};
use super::enums::CurrencyCode;

/// A currency configured in a Xero organisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Currency {
    /// Currency code (ISO 4217).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<CurrencyCode>,

    /// Description of the currency (e.g., "United States Dollar").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Collection wrapper for currencies returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Currencies {
    /// List of currencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currencies: Option<Vec<Currency>>,

    /// Pagination metadata (currencies endpoint is not paginated, but included for consistency).
    #[serde(rename = "pagination")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,

    /// Warnings returned by the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<Warning>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_deserialize_basic() {
        let json = r#"{
            "Code": "NZD",
            "Description": "New Zealand Dollar"
        }"#;
        let currency: Currency = serde_json::from_str(json).unwrap();
        assert_eq!(currency.code, Some(CurrencyCode::NZD));
        assert_eq!(currency.description.as_deref(), Some("New Zealand Dollar"));
    }

    #[test]
    fn currencies_collection() {
        let json = r#"{
            "Currencies": [
                {"Code": "NZD", "Description": "New Zealand Dollar"},
                {"Code": "USD", "Description": "United States Dollar"},
                {"Code": "GBP", "Description": "British Pound"}
            ]
        }"#;
        let currencies: Currencies = serde_json::from_str(json).unwrap();
        assert_eq!(currencies.currencies.as_ref().unwrap().len(), 3);
        assert_eq!(
            currencies.currencies.as_ref().unwrap()[1].code,
            Some(CurrencyCode::USD)
        );
    }
}
