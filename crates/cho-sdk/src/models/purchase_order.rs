//! Purchase order model for the Xero PurchaseOrders API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Attachment, LineItem, Pagination, ValidationError, Warning};
use super::contact::Contact;
use super::dates::{MsDate, MsDateTime};
use super::enums::{CurrencyCode, LineAmountTypes, PurchaseOrderStatus, PurchaseOrderType};

/// A purchase order in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PurchaseOrder {
    /// Unique identifier for the purchase order.
    #[serde(rename = "PurchaseOrderID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_order_id: Option<Uuid>,

    /// Xero-generated purchase order number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_order_number: Option<String>,

    /// Date string representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_string: Option<String>,

    /// Date the purchase order was issued.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<MsDate>,

    /// Delivery date string representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_date_string: Option<String>,

    /// Expected delivery date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_date: Option<MsDate>,

    /// Delivery address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_address: Option<String>,

    /// Attention to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attention_to: Option<String>,

    /// Telephone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telephone: Option<String>,

    /// Delivery instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_instructions: Option<String>,

    /// Whether the purchase order has errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_errors: Option<bool>,

    /// Whether discounts are applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_discounted: Option<bool>,

    /// Reference text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    /// Type of purchase order.
    #[serde(rename = "Type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_order_type: Option<PurchaseOrderType>,

    /// Currency rate for multi-currency purchase orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_rate: Option<Decimal>,

    /// Currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<CurrencyCode>,

    /// The contact (supplier) the purchase order is for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,

    /// Branding theme ID.
    #[serde(rename = "BrandingThemeID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branding_theme_id: Option<Uuid>,

    /// Purchase order status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PurchaseOrderStatus>,

    /// How line amounts are expressed (Exclusive, Inclusive, NoTax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_amount_types: Option<LineAmountTypes>,

    /// Line items on the purchase order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_items: Option<Vec<LineItem>>,

    /// Subtotal (ex-tax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_total: Option<Decimal>,

    /// Total tax amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tax: Option<Decimal>,

    /// Total amount (inc tax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<Decimal>,

    /// Last updated timestamp.
    #[serde(rename = "UpdatedDateUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date_utc: Option<MsDateTime>,

    /// Whether the purchase order has attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,

    /// Attachments on the purchase order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,

    /// Validation errors on this entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<ValidationError>>,

    /// Status attribute string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_attribute_string: Option<String>,
}

/// Collection wrapper for purchase orders returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PurchaseOrders {
    /// List of purchase orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_orders: Option<Vec<PurchaseOrder>>,

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
    fn purchase_order_deserialize_basic() {
        let json = r#"{
            "PurchaseOrderID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
            "PurchaseOrderNumber": "PO-0001",
            "Type": "PURCHASEORDER",
            "Status": "AUTHORISED",
            "Date": "/Date(1539993600000+0000)/",
            "DeliveryDate": "/Date(1542585600000+0000)/",
            "DeliveryAddress": "123 Main St",
            "AttentionTo": "Warehouse Manager",
            "Telephone": "+64 21 123 4567",
            "Reference": "Monthly supplies",
            "LineAmountTypes": "Exclusive",
            "CurrencyCode": "NZD",
            "SubTotal": 2000.00,
            "TotalTax": 300.00,
            "Total": 2300.00,
            "UpdatedDateUTC": "/Date(1573755038314)/",
            "Contact": {
                "ContactID": "a3e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
                "Name": "Supplier Ltd"
            },
            "LineItems": [
                {
                    "Description": "Office supplies",
                    "Quantity": 20.0,
                    "UnitAmount": 100.00,
                    "LineAmount": 2000.00,
                    "AccountCode": "300"
                }
            ]
        }"#;
        let po: PurchaseOrder = serde_json::from_str(json).unwrap();
        assert_eq!(po.purchase_order_number.as_deref(), Some("PO-0001"));
        assert_eq!(
            po.purchase_order_type,
            Some(PurchaseOrderType::PurchaseOrder)
        );
        assert_eq!(po.status, Some(PurchaseOrderStatus::Authorised));
        assert_eq!(po.total, Some(Decimal::new(2300, 0)));
        assert_eq!(po.delivery_address.as_deref(), Some("123 Main St"));
        assert!(po.contact.is_some());
        assert_eq!(po.line_items.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn purchase_orders_collection() {
        let json = r#"{
            "PurchaseOrders": [{
                "PurchaseOrderID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "PurchaseOrderNumber": "PO-0001",
                "Status": "DRAFT"
            }],
            "pagination": {"page": 1, "pageSize": 100, "pageCount": 1, "itemCount": 1}
        }"#;
        let pos: PurchaseOrders = serde_json::from_str(json).unwrap();
        assert_eq!(pos.purchase_orders.as_ref().unwrap().len(), 1);
        assert_eq!(pos.pagination.as_ref().unwrap().page, Some(1));
    }
}
