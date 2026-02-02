//! Overpayment model for the Xero Overpayments API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Allocation, Attachment, LineItem, Pagination, Warning};
use super::contact::Contact;
use super::dates::{MsDate, MsDateTime};
use super::enums::{CurrencyCode, LineAmountTypes, OverpaymentType};

/// An overpayment in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Overpayment {
    /// Unique identifier for the overpayment.
    #[serde(rename = "OverpaymentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overpayment_id: Option<Uuid>,

    /// Type of overpayment.
    #[serde(rename = "Type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overpayment_type: Option<OverpaymentType>,

    /// Contact associated with the overpayment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,

    /// Date of the overpayment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<MsDate>,

    /// Status of the overpayment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OverpaymentStatus>,

    /// How line amounts are expressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_amount_types: Option<LineAmountTypes>,

    /// Line items on the overpayment.
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

    /// Currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<CurrencyCode>,

    /// Currency exchange rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_rate: Option<Decimal>,

    /// Remaining credit on the overpayment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_credit: Option<Decimal>,

    /// Allocations of the overpayment to invoices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocations: Option<Vec<Allocation>>,

    /// Payments applied to the overpayment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payments: Option<Vec<super::payment::Payment>>,

    /// Whether the overpayment has attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,

    /// Attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
}

/// Status of an overpayment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OverpaymentStatus {
    /// Authorised overpayment.
    Authorised,
    /// Paid overpayment.
    Paid,
    /// Voided overpayment.
    Voided,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Xero API Overpayments collection wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Overpayments {
    /// List of overpayments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overpayments: Option<Vec<Overpayment>>,

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
    fn overpayment_deserialize_basic() {
        let json = serde_json::json!({
            "OverpaymentID": "00000000-0000-0000-0000-000000000001",
            "Type": "RECEIVE-OVERPAYMENT",
            "Status": "AUTHORISED",
            "SubTotal": "200.00",
            "TotalTax": "30.00",
            "Total": "230.00",
            "RemainingCredit": "100.00"
        });
        let o: Overpayment = serde_json::from_value(json).unwrap();
        assert_eq!(
            o.overpayment_id.unwrap().to_string(),
            "00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(o.status.unwrap(), OverpaymentStatus::Authorised);
        assert_eq!(o.remaining_credit.unwrap(), Decimal::new(10000, 2));
    }

    #[test]
    fn overpayments_collection() {
        let json = serde_json::json!({
            "Overpayments": [
                {
                    "OverpaymentID": "00000000-0000-0000-0000-000000000001",
                    "Status": "PAID"
                }
            ]
        });
        let col: Overpayments = serde_json::from_value(json).unwrap();
        assert_eq!(col.overpayments.unwrap().len(), 1);
    }
}
