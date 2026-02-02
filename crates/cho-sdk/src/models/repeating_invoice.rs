//! RepeatingInvoice model for the Xero Repeating Invoices API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Attachment, LineItem, Warning};
use super::contact::Contact;
use super::dates::MsDate;
use super::enums::{CurrencyCode, InvoiceType, LineAmountTypes};

/// A repeating invoice template in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RepeatingInvoice {
    /// Unique identifier for the repeating invoice.
    #[serde(rename = "RepeatingInvoiceID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeating_invoice_id: Option<Uuid>,

    /// Type of repeating invoice (ACCREC or ACCPAY).
    #[serde(rename = "Type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_type: Option<InvoiceType>,

    /// Contact for the repeating invoice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,

    /// Schedule for invoice generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<Schedule>,

    /// Line items on the repeating invoice template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_items: Option<Vec<LineItem>>,

    /// How line amounts are expressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_amount_types: Option<LineAmountTypes>,

    /// Reference text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    /// Branding theme ID.
    #[serde(rename = "BrandingThemeID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branding_theme_id: Option<Uuid>,

    /// Currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<CurrencyCode>,

    /// Status of the repeating invoice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<RepeatingInvoiceStatus>,

    /// Subtotal (ex-tax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_total: Option<Decimal>,

    /// Total tax amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tax: Option<Decimal>,

    /// Total amount (inc tax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<Decimal>,

    /// Whether the repeating invoice has attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,

    /// Attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,

    /// Whether invoices should be approved on creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_for_sending: Option<bool>,

    /// Whether to send a copy to the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_copy: Option<bool>,

    /// Whether to mark as sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_as_sent: Option<bool>,

    /// Whether to include PDF attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_pdf: Option<bool>,
}

/// Schedule for a repeating invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Schedule {
    /// Period (1-31 for days).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<i32>,

    /// Unit of schedule period.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<ScheduleUnit>,

    /// Due date type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date_type: Option<String>,

    /// Due date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<i32>,

    /// Start date of the schedule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<MsDate>,

    /// Next scheduled date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_scheduled_date: Option<MsDate>,

    /// End date of the schedule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<MsDate>,
}

/// Unit for schedule periods.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScheduleUnit {
    /// Weekly schedule.
    Weekly,
    /// Monthly schedule.
    Monthly,
    /// Unknown unit (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Status of a repeating invoice.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RepeatingInvoiceStatus {
    /// Draft repeating invoice.
    Draft,
    /// Authorised repeating invoice.
    Authorised,
    /// Deleted repeating invoice.
    Deleted,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Xero API RepeatingInvoices collection wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RepeatingInvoices {
    /// List of repeating invoices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeating_invoices: Option<Vec<RepeatingInvoice>>,

    /// Warnings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<Warning>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeating_invoice_deserialize_basic() {
        let json = serde_json::json!({
            "RepeatingInvoiceID": "00000000-0000-0000-0000-000000000001",
            "Type": "ACCREC",
            "Status": "AUTHORISED",
            "SubTotal": "1000.00",
            "TotalTax": "150.00",
            "Total": "1150.00",
            "Schedule": {
                "Period": 1,
                "Unit": "MONTHLY"
            }
        });
        let ri: RepeatingInvoice = serde_json::from_value(json).unwrap();
        assert_eq!(
            ri.repeating_invoice_id.unwrap().to_string(),
            "00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(ri.status.unwrap(), RepeatingInvoiceStatus::Authorised);
        let sched = ri.schedule.unwrap();
        assert_eq!(sched.period.unwrap(), 1);
        assert_eq!(sched.unit.unwrap(), ScheduleUnit::Monthly);
    }

    #[test]
    fn repeating_invoices_collection() {
        let json = serde_json::json!({
            "RepeatingInvoices": [
                {
                    "RepeatingInvoiceID": "00000000-0000-0000-0000-000000000001",
                    "Status": "DRAFT"
                }
            ]
        });
        let col: RepeatingInvoices = serde_json::from_value(json).unwrap();
        assert_eq!(col.repeating_invoices.unwrap().len(), 1);
    }
}
