//! Shared types used across multiple Xero API resources.
//!
//! Includes [`LineItem`], [`Pagination`], [`ValidationError`], [`Address`],
//! [`Phone`], [`ContactPerson`], and other common structures.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::dates::MsDateTime;

/// Pagination metadata returned by Xero collection endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    /// Current page number (1-indexed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    /// Number of items per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,

    /// Total number of pages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<u32>,

    /// Total number of items across all pages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_count: Option<u32>,
}

/// A validation error returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ValidationError {
    /// The error message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// A line item on an invoice, credit note, or other transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LineItem {
    /// Unique identifier for the line item.
    #[serde(rename = "LineItemID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_item_id: Option<Uuid>,

    /// Description of the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Quantity of the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Decimal>,

    /// Unit price of the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_amount: Option<Decimal>,

    /// Code for the item (from Xero Items).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_code: Option<String>,

    /// Account code for the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_code: Option<String>,

    /// The ID of the account.
    #[serde(rename = "AccountID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<Uuid>,

    /// Tax type for the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_type: Option<String>,

    /// Tax amount for the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_amount: Option<Decimal>,

    /// Total line amount (quantity * unit_amount).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_amount: Option<Decimal>,

    /// Discount percentage applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_rate: Option<Decimal>,

    /// Discount amount applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_amount: Option<Decimal>,

    /// Tracking categories assigned to this line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking: Option<Vec<LineItemTracking>>,
}

/// A tracking category assignment on a line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LineItemTracking {
    /// Tracking category name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Tracking option name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option: Option<String>,

    /// Tracking category ID.
    #[serde(rename = "TrackingCategoryID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_category_id: Option<Uuid>,

    /// Tracking option ID.
    #[serde(rename = "TrackingOptionID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_option_id: Option<Uuid>,
}

/// An allocation of a payment, credit note, prepayment, or overpayment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Allocation {
    /// The amount allocated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Decimal>,

    /// The date of the allocation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<super::dates::MsDate>,

    /// The invoice the allocation applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice: Option<AllocationInvoice>,

    /// Whether the allocation is part of a credit note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_deleted: Option<bool>,

    /// Status message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_attribute_string: Option<String>,
}

/// A minimal invoice reference used within allocations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AllocationInvoice {
    /// The invoice ID.
    #[serde(rename = "InvoiceID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<Uuid>,

    /// The invoice number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_number: Option<String>,
}

/// A file attachment on a Xero resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Attachment {
    /// Unique identifier for the attachment.
    #[serde(rename = "AttachmentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_id: Option<Uuid>,

    /// File name of the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,

    /// URL to access the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// MIME type of the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// File size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_length: Option<u64>,

    /// Whether to include the attachment with online invoices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_online: Option<bool>,
}

/// A physical or postal address.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Address {
    /// Address type (e.g., POBOX, STREET, DELIVERY).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_type: Option<AddressType>,

    /// First line of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line1: Option<String>,

    /// Second line of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line2: Option<String>,

    /// Third line of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line3: Option<String>,

    /// Fourth line of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line4: Option<String>,

    /// City name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// Region or state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Postal or ZIP code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,

    /// Country name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// Attention to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attention_to: Option<String>,
}

/// Type of address.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AddressType {
    /// PO Box address.
    #[serde(rename = "POBOX")]
    Pobox,
    /// Street address.
    Street,
    /// Delivery address.
    Delivery,
    /// Unknown address type (catch-all for forward compatibility).
    #[serde(other)]
    Unknown,
}

/// A phone number.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Phone {
    /// Phone type (e.g., DEFAULT, DDI, MOBILE, FAX).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_type: Option<PhoneType>,

    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,

    /// Area code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_area_code: Option<String>,

    /// Country code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_country_code: Option<String>,
}

/// Type of phone number.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PhoneType {
    /// Default phone.
    Default,
    /// Direct dial-in.
    #[serde(rename = "DDI")]
    Ddi,
    /// Mobile phone.
    Mobile,
    /// Fax number.
    Fax,
    /// Unknown type (catch-all for forward compatibility).
    #[serde(other)]
    Unknown,
}

/// A contact person within a contact organisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContactPerson {
    /// First name of the contact person.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,

    /// Last name of the contact person.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,

    /// Email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_address: Option<String>,

    /// Whether to include in emails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_in_emails: Option<bool>,
}

/// A warning returned by the Xero API alongside successful responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Warning {
    /// Warning message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Response metadata fields present on mutating API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ResponseMeta {
    /// Response ID (not the resource ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,

    /// Response status (e.g., "OK").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Provider name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_name: Option<String>,

    /// Response timestamp.
    #[serde(rename = "DateTimeUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_time_utc: Option<MsDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_deserialize() {
        let json = r#"{"page": 1, "pageSize": 100, "pageCount": 3, "itemCount": 250}"#;
        let p: Pagination = serde_json::from_str(json).unwrap();
        assert_eq!(p.page, Some(1));
        assert_eq!(p.page_size, Some(100));
        assert_eq!(p.page_count, Some(3));
        assert_eq!(p.item_count, Some(250));
    }

    #[test]
    fn line_item_with_decimal() {
        let json = r#"{
            "LineItemID": "d0f5d748-4f3b-4d1f-8e1a-5c8b2b5e1234",
            "Description": "Widget",
            "Quantity": 2.0,
            "UnitAmount": 49.99,
            "TaxAmount": 10.00,
            "LineAmount": 99.98
        }"#;
        let li: LineItem = serde_json::from_str(json).unwrap();
        assert_eq!(li.description.as_deref(), Some("Widget"));
        assert_eq!(li.unit_amount, Some(rust_decimal::Decimal::new(4999, 2)));
        assert_eq!(li.line_amount, Some(rust_decimal::Decimal::new(9998, 2)));
    }

    #[test]
    fn address_type_serde() {
        let json = r#""POBOX""#;
        let at: AddressType = serde_json::from_str(json).unwrap();
        assert_eq!(at, AddressType::Pobox);

        let json = r#""STREET""#;
        let at: AddressType = serde_json::from_str(json).unwrap();
        assert_eq!(at, AddressType::Street);

        // Unknown variant
        let json = r#""SOMETHING_NEW""#;
        let at: AddressType = serde_json::from_str(json).unwrap();
        assert_eq!(at, AddressType::Unknown);
    }

    #[test]
    fn phone_type_serde() {
        let json = r#""DEFAULT""#;
        let pt: PhoneType = serde_json::from_str(json).unwrap();
        assert_eq!(pt, PhoneType::Default);

        let json = r#""DDI""#;
        let pt: PhoneType = serde_json::from_str(json).unwrap();
        assert_eq!(pt, PhoneType::Ddi);

        let json = r#""MOBILE""#;
        let pt: PhoneType = serde_json::from_str(json).unwrap();
        assert_eq!(pt, PhoneType::Mobile);
    }

    #[test]
    fn decimal_precision_round_trip() {
        // Test that Decimal values survive JSON round-trip without precision loss
        let li = LineItem {
            line_item_id: None,
            description: None,
            quantity: Some(rust_decimal::Decimal::new(1, 0)),
            unit_amount: Some(rust_decimal::Decimal::new(1, 2)), // 0.01
            item_code: None,
            account_code: None,
            account_id: None,
            tax_type: None,
            tax_amount: None,
            line_amount: Some(rust_decimal::Decimal::new(99999999999, 2)), // 999999999.99
            discount_rate: None,
            discount_amount: None,
            tracking: None,
        };
        let json = serde_json::to_string(&li).unwrap();
        let li2: LineItem = serde_json::from_str(&json).unwrap();
        assert_eq!(li.unit_amount, li2.unit_amount);
        assert_eq!(li.line_amount, li2.line_amount);
    }

    #[test]
    fn decimal_negative_and_zero() {
        let json = r#"{"UnitAmount": -50.25, "LineAmount": 0.00}"#;
        let li: LineItem = serde_json::from_str(json).unwrap();
        assert_eq!(li.unit_amount, Some(rust_decimal::Decimal::new(-5025, 2)));
        assert_eq!(li.line_amount, Some(rust_decimal::Decimal::new(0, 2)));
    }
}
