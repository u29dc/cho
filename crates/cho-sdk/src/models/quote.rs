//! Quote model for the Xero Quotes API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{LineItem, Pagination, ValidationError, Warning};
use super::contact::Contact;
use super::dates::{MsDate, MsDateTime};
use super::enums::{CurrencyCode, LineAmountTypes, QuoteStatus};

/// A quote in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Quote {
    /// Unique identifier for the quote.
    #[serde(rename = "QuoteID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<Uuid>,

    /// Xero-generated quote number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_number: Option<String>,

    /// Reference text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    /// Terms of the quote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms: Option<String>,

    /// The contact the quote is for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,

    /// Line items on the quote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_items: Option<Vec<LineItem>>,

    /// Date the quote was issued.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<MsDate>,

    /// Date string representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_string: Option<String>,

    /// Expiry date of the quote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_date: Option<MsDate>,

    /// Expiry date string representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_date_string: Option<String>,

    /// Quote status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<QuoteStatus>,

    /// Currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<CurrencyCode>,

    /// Currency rate for multi-currency quotes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_rate: Option<Decimal>,

    /// Subtotal (ex-tax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_total: Option<Decimal>,

    /// Total tax amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tax: Option<Decimal>,

    /// Total amount (inc tax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<Decimal>,

    /// Total discount amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_discount: Option<Decimal>,

    /// Title of the quote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Summary text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Branding theme ID.
    #[serde(rename = "BrandingThemeID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branding_theme_id: Option<Uuid>,

    /// Last updated timestamp.
    #[serde(rename = "UpdatedDateUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date_utc: Option<MsDateTime>,

    /// How line amounts are expressed (Exclusive, Inclusive, NoTax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_amount_types: Option<LineAmountTypes>,

    /// Validation errors on this entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<ValidationError>>,

    /// Status attribute string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_attribute_string: Option<String>,
}

/// Collection wrapper for quotes returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Quotes {
    /// List of quotes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotes: Option<Vec<Quote>>,

    /// Pagination metadata.
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
    fn quote_deserialize_basic() {
        let json = r#"{
            "QuoteID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
            "QuoteNumber": "QU-0001",
            "Reference": "Website redesign",
            "Title": "Web Project Quote",
            "Summary": "Quote for website redesign project",
            "Status": "SENT",
            "Date": "/Date(1539993600000+0000)/",
            "ExpiryDate": "/Date(1542585600000+0000)/",
            "LineAmountTypes": "Exclusive",
            "CurrencyCode": "USD",
            "SubTotal": 5000.00,
            "TotalTax": 750.00,
            "Total": 5750.00,
            "TotalDiscount": 0.00,
            "UpdatedDateUTC": "/Date(1573755038314)/",
            "Contact": {
                "ContactID": "a3e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
                "Name": "Client Co"
            },
            "LineItems": [
                {
                    "Description": "Design work",
                    "Quantity": 50.0,
                    "UnitAmount": 100.00,
                    "LineAmount": 5000.00
                }
            ]
        }"#;
        let quote: Quote = serde_json::from_str(json).unwrap();
        assert_eq!(quote.quote_number.as_deref(), Some("QU-0001"));
        assert_eq!(quote.status, Some(QuoteStatus::Sent));
        assert_eq!(quote.total, Some(Decimal::new(5750, 0)));
        assert_eq!(quote.title.as_deref(), Some("Web Project Quote"));
        assert!(quote.contact.is_some());
        assert_eq!(quote.line_items.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn quotes_collection_with_pagination() {
        let json = r#"{
            "Quotes": [{
                "QuoteID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "QuoteNumber": "QU-0001",
                "Status": "DRAFT"
            }],
            "pagination": {"Page": 1, "PageSize": 100, "PageCount": 1, "ItemCount": 1}
        }"#;
        let quotes: Quotes = serde_json::from_str(json).unwrap();
        assert_eq!(quotes.quotes.as_ref().unwrap().len(), 1);
        assert_eq!(quotes.pagination.as_ref().unwrap().page, Some(1));
    }
}
