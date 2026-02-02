//! Credit note model for the Xero CreditNotes API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Allocation, Attachment, LineItem, Pagination, ValidationError, Warning};
use super::contact::Contact;
use super::dates::{MsDate, MsDateTime};
use super::enums::{CreditNoteStatus, CreditNoteType, CurrencyCode, LineAmountTypes};

/// A credit note in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreditNote {
    /// Unique identifier for the credit note.
    #[serde(rename = "CreditNoteID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_note_id: Option<Uuid>,

    /// Type of credit note (ACCPAYCREDIT or ACCRECCREDIT).
    #[serde(rename = "Type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_note_type: Option<CreditNoteType>,

    /// The contact the credit note is raised for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,

    /// Date the credit note was issued.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<MsDate>,

    /// Due date of the credit note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<MsDate>,

    /// Credit note status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<CreditNoteStatus>,

    /// How line amounts are expressed (Exclusive, Inclusive, NoTax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_amount_types: Option<LineAmountTypes>,

    /// Line items on the credit note.
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

    /// CIS deduction amount (UK Construction Industry Scheme).
    #[serde(rename = "CISDeduction")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cis_deduction: Option<Decimal>,

    /// CIS rate.
    #[serde(rename = "CISRate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cis_rate: Option<Decimal>,

    /// Last updated timestamp.
    #[serde(rename = "UpdatedDateUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date_utc: Option<MsDateTime>,

    /// Currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<CurrencyCode>,

    /// Date the credit note was fully paid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fully_paid_on_date: Option<MsDate>,

    /// Credit note number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_note_number: Option<String>,

    /// Reference text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    /// Whether the credit note has been sent to the contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_to_contact: Option<bool>,

    /// Currency rate for multi-currency credit notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_rate: Option<Decimal>,

    /// Remaining credit amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_credit: Option<Decimal>,

    /// Allocations applied from this credit note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocations: Option<Vec<Allocation>>,

    /// Branding theme ID.
    #[serde(rename = "BrandingThemeID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branding_theme_id: Option<Uuid>,

    /// Whether the credit note has attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,

    /// Attachments on the credit note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,

    /// Whether the credit note has errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_errors: Option<bool>,

    /// Validation errors on this entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<ValidationError>>,

    /// Status attribute string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_attribute_string: Option<String>,
}

/// Collection wrapper for credit notes returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreditNotes {
    /// List of credit notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_notes: Option<Vec<CreditNote>>,

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
    fn credit_note_deserialize_basic() {
        let json = r#"{
            "CreditNoteID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
            "Type": "ACCRECCREDIT",
            "CreditNoteNumber": "CN-0001",
            "Reference": "Refund for overcharge",
            "Status": "AUTHORISED",
            "Date": "/Date(1539993600000+0000)/",
            "DueDate": "/Date(1542585600000+0000)/",
            "LineAmountTypes": "Exclusive",
            "CurrencyCode": "NZD",
            "SubTotal": 500.00,
            "TotalTax": 75.00,
            "Total": 575.00,
            "RemainingCredit": 575.00,
            "UpdatedDateUTC": "/Date(1573755038314)/",
            "Contact": {
                "ContactID": "a3e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
                "Name": "Acme Corp"
            },
            "LineItems": [
                {
                    "Description": "Refund item",
                    "Quantity": 1.0,
                    "UnitAmount": 500.00,
                    "TaxAmount": 75.00,
                    "LineAmount": 500.00,
                    "AccountCode": "200"
                }
            ]
        }"#;
        let cn: CreditNote = serde_json::from_str(json).unwrap();
        assert_eq!(cn.credit_note_number.as_deref(), Some("CN-0001"));
        assert_eq!(cn.credit_note_type, Some(CreditNoteType::AccRecCredit));
        assert_eq!(cn.status, Some(CreditNoteStatus::Authorised));
        assert_eq!(cn.total, Some(Decimal::new(575, 0)));
        assert_eq!(cn.remaining_credit, Some(Decimal::new(575, 0)));
        assert!(cn.contact.is_some());
        assert_eq!(cn.line_items.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn credit_notes_collection_with_pagination() {
        let json = r#"{
            "CreditNotes": [{
                "CreditNoteID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "Type": "ACCPAYCREDIT",
                "CreditNoteNumber": "CN-0002"
            }],
            "pagination": {"Page": 1, "PageSize": 100, "PageCount": 1, "ItemCount": 1}
        }"#;
        let cns: CreditNotes = serde_json::from_str(json).unwrap();
        assert_eq!(cns.credit_notes.as_ref().unwrap().len(), 1);
        assert_eq!(cns.pagination.as_ref().unwrap().page, Some(1));
    }

    #[test]
    fn credit_note_with_allocations() {
        let json = r#"{
            "CreditNoteID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
            "Total": 500.00,
            "RemainingCredit": 200.00,
            "Allocations": [{
                "Amount": 300.00,
                "Date": "/Date(1573755038000+0000)/",
                "Invoice": {
                    "InvoiceID": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
                    "InvoiceNumber": "INV-0005"
                }
            }]
        }"#;
        let cn: CreditNote = serde_json::from_str(json).unwrap();
        assert_eq!(cn.allocations.as_ref().unwrap().len(), 1);
        assert_eq!(cn.remaining_credit, Some(Decimal::new(200, 0)));
    }
}
