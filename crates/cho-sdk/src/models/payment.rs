//! Payment model for the Xero Payments API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Pagination, ValidationError, Warning};
use super::dates::{MsDate, MsDateTime};
use super::enums::{CurrencyCode, PaymentStatus, PaymentType};

/// A payment in Xero, applied to an invoice, credit note, prepayment, or overpayment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Payment {
    /// Unique identifier for the payment.
    #[serde(rename = "PaymentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_id: Option<Uuid>,

    /// Date of the payment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<MsDate>,

    /// Amount of the payment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Decimal>,

    /// Currency rate at the time of payment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_rate: Option<Decimal>,

    /// Payment type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_type: Option<PaymentType>,

    /// Payment status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PaymentStatus>,

    /// Reference text for the payment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    /// Whether this payment is reconciled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_reconciled: Option<bool>,

    /// The bank account the payment is from/to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<PaymentAccount>,

    /// Invoice this payment is applied to (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice: Option<PaymentInvoice>,

    /// Credit note this payment is applied to (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_note: Option<PaymentCreditNote>,

    /// Prepayment this payment is applied to (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepayment: Option<PaymentPrepayment>,

    /// Overpayment this payment is applied to (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overpayment: Option<PaymentOverpayment>,

    /// Currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<CurrencyCode>,

    /// Bank amount (in bank currency).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_amount: Option<Decimal>,

    /// Batch payment ID (if part of a batch).
    #[serde(rename = "BatchPaymentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_payment_id: Option<Uuid>,

    /// Whether the payment has an account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_account: Option<bool>,

    /// Whether the payment has validation errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_validation_errors: Option<bool>,

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

/// Bank account reference in a payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentAccount {
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

/// Invoice reference within a payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentInvoice {
    /// Invoice ID.
    #[serde(rename = "InvoiceID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<Uuid>,

    /// Invoice number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_number: Option<String>,
}

/// Credit note reference within a payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentCreditNote {
    /// Credit note ID.
    #[serde(rename = "CreditNoteID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_note_id: Option<Uuid>,

    /// Credit note number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_note_number: Option<String>,
}

/// Prepayment reference within a payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentPrepayment {
    /// Prepayment ID.
    #[serde(rename = "PrepaymentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepayment_id: Option<Uuid>,
}

/// Overpayment reference within a payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentOverpayment {
    /// Overpayment ID.
    #[serde(rename = "OverpaymentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overpayment_id: Option<Uuid>,
}

/// Collection wrapper for payments returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Payments {
    /// List of payments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payments: Option<Vec<Payment>>,

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
    fn payment_deserialize() {
        let json = r#"{
            "PaymentID": "b5e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
            "Date": "/Date(1573755038000+0000)/",
            "Amount": 500.00,
            "Status": "AUTHORISED",
            "PaymentType": "ACCRECPAYMENT",
            "IsReconciled": true,
            "CurrencyCode": "NZD",
            "UpdatedDateUTC": "/Date(1573755038314)/",
            "Invoice": {
                "InvoiceID": "243216c5-369e-4f34-ad4e-fc2cf9d19392",
                "InvoiceNumber": "INV-0001"
            },
            "Account": {
                "AccountID": "ebd06280-af70-4bed-97c6-7451a454ad85",
                "Code": "090",
                "Name": "Business Bank Account"
            }
        }"#;
        let payment: Payment = serde_json::from_str(json).unwrap();
        assert_eq!(payment.amount, Some(Decimal::new(500, 0)));
        assert_eq!(payment.status, Some(PaymentStatus::Authorised));
        assert_eq!(payment.payment_type, Some(PaymentType::AccRecPayment));
        assert!(payment.invoice.is_some());
        assert!(payment.account.is_some());
    }

    #[test]
    fn payments_collection() {
        let json = r#"{
            "Payments": [{
                "PaymentID": "b5e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
                "Amount": 100.00
            }],
            "pagination": {"page": 1, "pageSize": 100, "pageCount": 1, "itemCount": 1}
        }"#;
        let payments: Payments = serde_json::from_str(json).unwrap();
        assert_eq!(payments.payments.as_ref().unwrap().len(), 1);
    }
}
