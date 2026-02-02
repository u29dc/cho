//! Prepayment model for the Xero Prepayments API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Allocation, Attachment, LineItem, Pagination, Warning};
use super::contact::Contact;
use super::dates::{MsDate, MsDateTime};
use super::enums::{CurrencyCode, LineAmountTypes, PrepaymentType};

/// A prepayment in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Prepayment {
    /// Unique identifier for the prepayment.
    #[serde(rename = "PrepaymentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepayment_id: Option<Uuid>,

    /// Type of prepayment.
    #[serde(rename = "Type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepayment_type: Option<PrepaymentType>,

    /// Contact associated with the prepayment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,

    /// Date of the prepayment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<MsDate>,

    /// Status of the prepayment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PrepaymentStatus>,

    /// How line amounts are expressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_amount_types: Option<LineAmountTypes>,

    /// Line items on the prepayment.
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

    /// Remaining credit on the prepayment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_credit: Option<Decimal>,

    /// Allocations of the prepayment to invoices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocations: Option<Vec<Allocation>>,

    /// Payments applied to the prepayment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payments: Option<Vec<super::payment::Payment>>,

    /// Reference text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    /// Whether the prepayment has attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,

    /// Attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
}

/// Status of a prepayment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrepaymentStatus {
    /// Authorised prepayment.
    Authorised,
    /// Paid prepayment.
    Paid,
    /// Voided prepayment.
    Voided,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Xero API Prepayments collection wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Prepayments {
    /// List of prepayments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepayments: Option<Vec<Prepayment>>,

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
    fn prepayment_deserialize_basic() {
        let json = serde_json::json!({
            "PrepaymentID": "00000000-0000-0000-0000-000000000001",
            "Type": "RECEIVE-PREPAYMENT",
            "Status": "AUTHORISED",
            "SubTotal": "100.00",
            "TotalTax": "15.00",
            "Total": "115.00",
            "RemainingCredit": "50.00"
        });
        let p: Prepayment = serde_json::from_value(json).unwrap();
        assert_eq!(
            p.prepayment_id.unwrap().to_string(),
            "00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(p.status.unwrap(), PrepaymentStatus::Authorised);
        assert_eq!(p.total.unwrap(), Decimal::new(11500, 2));
    }

    #[test]
    fn prepayments_collection() {
        let json = serde_json::json!({
            "Prepayments": [
                {
                    "PrepaymentID": "00000000-0000-0000-0000-000000000001",
                    "Status": "AUTHORISED"
                }
            ]
        });
        let col: Prepayments = serde_json::from_value(json).unwrap();
        assert_eq!(col.prepayments.unwrap().len(), 1);
    }
}
