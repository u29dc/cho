//! Item model for the Xero Items API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Pagination, ValidationError, Warning};
use super::dates::MsDateTime;

/// An item (product or service) in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    /// Unique identifier for the item.
    #[serde(rename = "ItemID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<Uuid>,

    /// User-defined item code (up to 30 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Item name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Sales description of the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Purchase description of the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_description: Option<String>,

    /// Whether the item is sold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_sold: Option<bool>,

    /// Whether the item is purchased.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_purchased: Option<bool>,

    /// Whether the item is tracked as inventory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_tracked_as_inventory: Option<bool>,

    /// Account code for the inventory asset account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_asset_account_code: Option<String>,

    /// Total cost pool for inventory items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost_pool: Option<Decimal>,

    /// Quantity on hand for inventory items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity_on_hand: Option<Decimal>,

    /// Last updated timestamp.
    #[serde(rename = "UpdatedDateUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date_utc: Option<MsDateTime>,

    /// Purchase details (cost pricing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_details: Option<ItemDetails>,

    /// Sales details (selling pricing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_details: Option<ItemDetails>,

    /// Validation errors on this entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<ValidationError>>,

    /// Status attribute string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_attribute_string: Option<String>,
}

/// Pricing details for an item (purchase or sales side).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ItemDetails {
    /// Unit price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_price: Option<Decimal>,

    /// Account code for this pricing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_code: Option<String>,

    /// Cost of goods sold account code.
    #[serde(rename = "COGSAccountCode")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cogs_account_code: Option<String>,

    /// Tax type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_type: Option<String>,
}

/// Collection wrapper for items returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Items {
    /// List of items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Item>>,

    /// Pagination metadata (items endpoint is not paginated, but included for consistency).
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
    fn item_deserialize_basic() {
        let json = r#"{
            "ItemID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
            "Code": "WIDGET-01",
            "Name": "Standard Widget",
            "Description": "A standard widget for sale",
            "PurchaseDescription": "Widget purchased from supplier",
            "IsSold": true,
            "IsPurchased": true,
            "IsTrackedAsInventory": true,
            "InventoryAssetAccountCode": "630",
            "TotalCostPool": 5000.00,
            "QuantityOnHand": 100.00,
            "UpdatedDateUTC": "/Date(1573755038314)/",
            "PurchaseDetails": {
                "UnitPrice": 25.00,
                "AccountCode": "300",
                "COGSAccountCode": "500",
                "TaxType": "INPUT"
            },
            "SalesDetails": {
                "UnitPrice": 50.00,
                "AccountCode": "200",
                "TaxType": "OUTPUT"
            }
        }"#;
        let item: Item = serde_json::from_str(json).unwrap();
        assert_eq!(item.code.as_deref(), Some("WIDGET-01"));
        assert_eq!(item.name.as_deref(), Some("Standard Widget"));
        assert_eq!(item.is_sold, Some(true));
        assert_eq!(item.is_tracked_as_inventory, Some(true));
        assert_eq!(item.quantity_on_hand, Some(Decimal::new(100, 0)));

        let purchase = item.purchase_details.as_ref().unwrap();
        assert_eq!(purchase.unit_price, Some(Decimal::new(25, 0)));
        assert_eq!(purchase.account_code.as_deref(), Some("300"));
        assert_eq!(purchase.cogs_account_code.as_deref(), Some("500"));

        let sales = item.sales_details.as_ref().unwrap();
        assert_eq!(sales.unit_price, Some(Decimal::new(50, 0)));
        assert_eq!(sales.tax_type.as_deref(), Some("OUTPUT"));
    }

    #[test]
    fn items_collection() {
        let json = r#"{
            "Items": [{
                "ItemID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "Code": "WIDGET-01",
                "Name": "Widget"
            }]
        }"#;
        let items: Items = serde_json::from_str(json).unwrap();
        assert_eq!(items.items.as_ref().unwrap().len(), 1);
    }
}
