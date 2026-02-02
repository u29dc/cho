//! Invoice model for the Xero Invoices API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Attachment, LineItem, Pagination, ValidationError, Warning};
use super::contact::Contact;
use super::dates::{MsDate, MsDateTime};
use super::enums::{CurrencyCode, InvoiceStatus, InvoiceType, LineAmountTypes};

/// An invoice (sales or purchase) in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Invoice {
    /// Unique identifier for the invoice.
    #[serde(rename = "InvoiceID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<Uuid>,

    /// Type of invoice (ACCREC or ACCPAY).
    #[serde(rename = "Type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_type: Option<InvoiceType>,

    /// The contact the invoice is raised to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,

    /// Line items on the invoice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_items: Option<Vec<LineItem>>,

    /// Date the invoice was issued.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<MsDate>,

    /// Date the invoice is due.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<MsDate>,

    /// How line amounts are expressed (Exclusive, Inclusive, NoTax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_amount_types: Option<LineAmountTypes>,

    /// Xero-generated invoice number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_number: Option<String>,

    /// Reference text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    /// Branding theme ID.
    #[serde(rename = "BrandingThemeID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branding_theme_id: Option<Uuid>,

    /// URL link to a source document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<CurrencyCode>,

    /// Currency rate for multi-currency invoices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_rate: Option<Decimal>,

    /// Invoice status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<InvoiceStatus>,

    /// Whether sent to contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_to_contact: Option<bool>,

    /// Expected payment date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_payment_date: Option<MsDate>,

    /// Planned payment date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_payment_date: Option<MsDate>,

    /// CIS deduction.
    #[serde(rename = "CISDeduction")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cis_deduction: Option<Decimal>,

    /// CIS rate.
    #[serde(rename = "CISRate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cis_rate: Option<Decimal>,

    /// Subtotal (ex-tax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_total: Option<Decimal>,

    /// Total tax amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tax: Option<Decimal>,

    /// Total amount (inc tax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<Decimal>,

    /// Total discount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_discount: Option<Decimal>,

    /// Amount due.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_due: Option<Decimal>,

    /// Amount paid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_paid: Option<Decimal>,

    /// Amount credited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_credited: Option<Decimal>,

    /// Fully paid date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fully_paid_on_date: Option<MsDate>,

    /// Payments applied to this invoice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payments: Option<Vec<InvoicePayment>>,

    /// Prepayments applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepayments: Option<Vec<InvoicePrepayment>>,

    /// Overpayments applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overpayments: Option<Vec<InvoiceOverpayment>>,

    /// Credit notes applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_notes: Option<Vec<InvoiceCreditNote>>,

    /// Whether the invoice has attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,

    /// Whether the invoice has errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_errors: Option<bool>,

    /// Attachments on the invoice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,

    /// Repeating invoice ID if generated from repeating.
    #[serde(rename = "RepeatingInvoiceID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeating_invoice_id: Option<Uuid>,

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

/// A minimal payment reference embedded in an invoice response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InvoicePayment {
    /// Payment ID.
    #[serde(rename = "PaymentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_id: Option<Uuid>,

    /// Payment date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<MsDate>,

    /// Payment amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Decimal>,

    /// Currency rate at payment time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_rate: Option<Decimal>,

    /// Whether this payment has an account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_account: Option<bool>,

    /// Whether this payment has been validated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_validation_errors: Option<bool>,
}

/// A minimal prepayment reference embedded in an invoice response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InvoicePrepayment {
    /// Prepayment ID.
    #[serde(rename = "PrepaymentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepayment_id: Option<Uuid>,

    /// Applied amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_amount: Option<Decimal>,
}

/// A minimal overpayment reference embedded in an invoice response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InvoiceOverpayment {
    /// Overpayment ID.
    #[serde(rename = "OverpaymentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overpayment_id: Option<Uuid>,

    /// Applied amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_amount: Option<Decimal>,
}

/// A minimal credit note reference embedded in an invoice response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InvoiceCreditNote {
    /// Credit note ID.
    #[serde(rename = "CreditNoteID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_note_id: Option<Uuid>,

    /// Credit note number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_note_number: Option<String>,

    /// Applied amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_amount: Option<Decimal>,
}

/// Collection wrapper for invoices returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Invoices {
    /// List of invoices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoices: Option<Vec<Invoice>>,

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
    fn invoice_deserialize_basic() {
        let json = r#"{
            "InvoiceID": "243216c5-369e-4f34-ad4e-fc2cf9d19392",
            "Type": "ACCREC",
            "InvoiceNumber": "INV-0001",
            "Reference": "Monthly retainer",
            "Status": "AUTHORISED",
            "Date": "/Date(1539993600000+0000)/",
            "DueDate": "/Date(1542585600000+0000)/",
            "LineAmountTypes": "Exclusive",
            "CurrencyCode": "NZD",
            "SubTotal": 1000.00,
            "TotalTax": 150.00,
            "Total": 1150.00,
            "AmountDue": 1150.00,
            "AmountPaid": 0.00,
            "UpdatedDateUTC": "/Date(1573755038314)/",
            "Contact": {
                "ContactID": "a3e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
                "Name": "Acme Corp"
            },
            "LineItems": [
                {
                    "Description": "Consulting",
                    "Quantity": 10.0,
                    "UnitAmount": 100.00,
                    "TaxAmount": 150.00,
                    "LineAmount": 1000.00,
                    "AccountCode": "200"
                }
            ]
        }"#;
        let invoice: Invoice = serde_json::from_str(json).unwrap();
        assert_eq!(invoice.invoice_number.as_deref(), Some("INV-0001"));
        assert_eq!(invoice.invoice_type, Some(InvoiceType::AccRec));
        assert_eq!(invoice.status, Some(InvoiceStatus::Authorised));
        assert_eq!(invoice.total, Some(Decimal::new(1150, 0)));
        assert!(invoice.contact.is_some());
        assert_eq!(invoice.line_items.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn invoices_collection_with_pagination() {
        let json = r#"{
            "Invoices": [{
                "InvoiceID": "243216c5-369e-4f34-ad4e-fc2cf9d19392",
                "Type": "ACCREC",
                "InvoiceNumber": "INV-0001"
            }],
            "pagination": {"Page": 1, "PageSize": 100, "PageCount": 1, "ItemCount": 1}
        }"#;
        let invoices: Invoices = serde_json::from_str(json).unwrap();
        assert_eq!(invoices.invoices.as_ref().unwrap().len(), 1);
        assert_eq!(invoices.pagination.as_ref().unwrap().page, Some(1));
    }

    #[test]
    fn invoice_with_payments() {
        let json = r#"{
            "InvoiceID": "243216c5-369e-4f34-ad4e-fc2cf9d19392",
            "Payments": [{
                "PaymentID": "b5e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
                "Date": "/Date(1573755038000+0000)/",
                "Amount": 500.00
            }],
            "CreditNotes": [{
                "CreditNoteID": "c6f6f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
                "CreditNoteNumber": "CN-0001",
                "AppliedAmount": 100.00
            }]
        }"#;
        let invoice: Invoice = serde_json::from_str(json).unwrap();
        assert_eq!(invoice.payments.as_ref().unwrap().len(), 1);
        assert_eq!(invoice.credit_notes.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn invoice_decimal_precision() {
        let json = r#"{
            "InvoiceID": "243216c5-369e-4f34-ad4e-fc2cf9d19392",
            "SubTotal": 999999999.99,
            "TotalTax": 0.01,
            "Total": 999999999.99,
            "AmountDue": 0.00,
            "AmountPaid": 999999999.99
        }"#;
        let invoice: Invoice = serde_json::from_str(json).unwrap();
        assert_eq!(invoice.sub_total, Some(Decimal::new(99999999999, 2)));
        assert_eq!(invoice.total_tax, Some(Decimal::new(1, 2)));
    }
}
