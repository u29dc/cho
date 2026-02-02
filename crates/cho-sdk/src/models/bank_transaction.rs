//! Bank transaction model for the Xero BankTransactions API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Attachment, LineItem, Pagination, ValidationError, Warning};
use super::contact::Contact;
use super::dates::{MsDate, MsDateTime};
use super::enums::{BankTransactionStatus, BankTransactionType, CurrencyCode, LineAmountTypes};

/// A bank transaction (receive/spend money) in Xero.
///
/// Note: `Type`, `LineItems`, and `BankAccount` are required for creation,
/// but all fields are optional for deserialization of API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BankTransaction {
    /// Unique identifier for the bank transaction.
    #[serde(rename = "BankTransactionID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_transaction_id: Option<Uuid>,

    /// Type of bank transaction.
    #[serde(rename = "Type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<BankTransactionType>,

    /// Contact for the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,

    /// Line items on the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_items: Option<Vec<LineItem>>,

    /// Bank account for the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account: Option<TransactionBankAccount>,

    /// Date of the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<MsDate>,

    /// How line amounts are expressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_amount_types: Option<LineAmountTypes>,

    /// Reference text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    /// URL link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<CurrencyCode>,

    /// Currency rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_rate: Option<Decimal>,

    /// Transaction status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<BankTransactionStatus>,

    /// Subtotal (ex-tax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_total: Option<Decimal>,

    /// Total tax.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tax: Option<Decimal>,

    /// Total amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<Decimal>,

    /// Whether the transaction is reconciled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_reconciled: Option<bool>,

    /// Whether the transaction has attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,

    /// Attachments on the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,

    /// Prepayment ID (if this transaction created a prepayment).
    #[serde(rename = "PrepaymentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepayment_id: Option<Uuid>,

    /// Overpayment ID (if this transaction created an overpayment).
    #[serde(rename = "OverpaymentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overpayment_id: Option<Uuid>,

    /// Last updated timestamp.
    #[serde(rename = "UpdatedDateUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date_utc: Option<MsDateTime>,

    /// Validation errors on this entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<ValidationError>>,

    /// Status attribute string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_attribute_string: Option<String>,
}

/// Bank account reference within a bank transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TransactionBankAccount {
    /// Account ID.
    #[serde(rename = "AccountID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<Uuid>,

    /// Account code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Account name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Collection wrapper for bank transactions returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BankTransactions {
    /// List of bank transactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_transactions: Option<Vec<BankTransaction>>,

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
    fn bank_transaction_deserialize() {
        let json = r#"{
            "BankTransactionID": "d7e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
            "Type": "RECEIVE",
            "Date": "/Date(1573755038000+0000)/",
            "Status": "AUTHORISED",
            "LineAmountTypes": "Inclusive",
            "SubTotal": 100.00,
            "TotalTax": 15.00,
            "Total": 115.00,
            "IsReconciled": false,
            "CurrencyCode": "NZD",
            "UpdatedDateUTC": "/Date(1573755038314)/",
            "Contact": {
                "ContactID": "a3e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
                "Name": "Test Client"
            },
            "BankAccount": {
                "AccountID": "ebd06280-af70-4bed-97c6-7451a454ad85",
                "Code": "090",
                "Name": "Business Bank Account"
            },
            "LineItems": [
                {
                    "Description": "Payment received",
                    "Quantity": 1.0,
                    "UnitAmount": 100.00,
                    "LineAmount": 100.00
                }
            ]
        }"#;
        let tx: BankTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.transaction_type, Some(BankTransactionType::Receive));
        assert_eq!(tx.status, Some(BankTransactionStatus::Authorised));
        assert_eq!(tx.total, Some(Decimal::new(115, 0)));
        assert!(tx.contact.is_some());
        assert!(tx.bank_account.is_some());
        assert_eq!(tx.line_items.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn bank_transactions_collection() {
        let json = r#"{
            "BankTransactions": [{
                "BankTransactionID": "d7e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
                "Type": "SPEND",
                "Total": 50.00
            }],
            "pagination": {"page": 1, "pageSize": 100, "pageCount": 1, "itemCount": 1}
        }"#;
        let txns: BankTransactions = serde_json::from_str(json).unwrap();
        assert_eq!(txns.bank_transactions.as_ref().unwrap().len(), 1);
        assert_eq!(txns.pagination.as_ref().unwrap().page, Some(1));
    }
}
