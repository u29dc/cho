//! LinkedTransaction model for the Xero Linked Transactions API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Pagination, Warning};
use super::dates::MsDateTime;

/// A linked transaction in Xero, linking a billable expense to a sales invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LinkedTransaction {
    /// Unique identifier for the linked transaction.
    #[serde(rename = "LinkedTransactionID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_transaction_id: Option<Uuid>,

    /// Source transaction ID (e.g., a bill).
    #[serde(rename = "SourceTransactionID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_transaction_id: Option<Uuid>,

    /// Source line item ID.
    #[serde(rename = "SourceLineItemID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_line_item_id: Option<Uuid>,

    /// Contact ID on the source transaction.
    #[serde(rename = "ContactID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_id: Option<Uuid>,

    /// Target transaction ID (e.g., a sales invoice).
    #[serde(rename = "TargetTransactionID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_transaction_id: Option<Uuid>,

    /// Target line item ID.
    #[serde(rename = "TargetLineItemID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_line_item_id: Option<Uuid>,

    /// Status of the linked transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<LinkedTransactionStatus>,

    /// Type of the linked transaction.
    #[serde(rename = "Type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_transaction_type: Option<String>,

    /// Source transaction type code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,

    /// Amount of the source line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_amount: Option<Decimal>,

    /// Last updated timestamp.
    #[serde(rename = "UpdatedDateUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date_utc: Option<MsDateTime>,
}

/// Status of a linked transaction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LinkedTransactionStatus {
    /// Approved linked transaction.
    Approved,
    /// Draft linked transaction.
    Draft,
    /// Ondraft linked transaction.
    Ondraft,
    /// Billed linked transaction.
    Billed,
    /// Voided linked transaction.
    Voided,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Xero API LinkedTransactions collection wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LinkedTransactions {
    /// List of linked transactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_transactions: Option<Vec<LinkedTransaction>>,

    /// Pagination info.
    #[serde(rename = "pagination")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,

    /// Warnings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<Warning>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linked_transaction_deserialize_basic() {
        let json = serde_json::json!({
            "LinkedTransactionID": "00000000-0000-0000-0000-000000000001",
            "SourceTransactionID": "00000000-0000-0000-0000-000000000002",
            "ContactID": "00000000-0000-0000-0000-000000000003",
            "Status": "APPROVED",
            "SourceAmount": "500.00"
        });
        let lt: LinkedTransaction = serde_json::from_value(json).unwrap();
        assert_eq!(
            lt.linked_transaction_id.unwrap().to_string(),
            "00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(lt.status.unwrap(), LinkedTransactionStatus::Approved);
        assert_eq!(lt.source_amount.unwrap(), Decimal::new(50000, 2));
    }

    #[test]
    fn linked_transactions_collection() {
        let json = serde_json::json!({
            "LinkedTransactions": [
                {
                    "LinkedTransactionID": "00000000-0000-0000-0000-000000000001",
                    "Status": "DRAFT"
                }
            ]
        });
        let col: LinkedTransactions = serde_json::from_value(json).unwrap();
        assert_eq!(col.linked_transactions.unwrap().len(), 1);
    }
}
