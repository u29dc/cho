//! Manual journal model for the Xero ManualJournals API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{LineItemTracking, Pagination, ValidationError, Warning};
use super::dates::{MsDate, MsDateTime};
use super::enums::ManualJournalStatus;

/// A manual journal entry in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManualJournal {
    /// Unique identifier for the manual journal.
    #[serde(rename = "ManualJournalID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_journal_id: Option<Uuid>,

    /// Date of the journal entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<MsDate>,

    /// Status of the manual journal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ManualJournalStatus>,

    /// Narration (description) of the journal entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narration: Option<String>,

    /// Journal lines (debit and credit entries).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_lines: Option<Vec<ManualJournalLine>>,

    /// URL link to a source document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Whether to show this journal on cash basis reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_on_cash_basis_reports: Option<bool>,

    /// Whether the journal has attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,

    /// Last updated timestamp.
    #[serde(rename = "UpdatedDateUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date_utc: Option<MsDateTime>,

    /// Warnings on this journal entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<Warning>>,

    /// Validation errors on this entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<ValidationError>>,

    /// Status attribute string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_attribute_string: Option<String>,
}

/// A line within a manual journal (debit or credit entry).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManualJournalLine {
    /// Line amount (positive for debit, negative for credit, or vice versa).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_amount: Option<Decimal>,

    /// Account code for this journal line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_code: Option<String>,

    /// Account ID for this journal line.
    #[serde(rename = "AccountID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<Uuid>,

    /// Description of the journal line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Tax type for the journal line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_type: Option<String>,

    /// Tracking categories for this journal line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking: Option<Vec<LineItemTracking>>,

    /// Tax amount for the journal line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_amount: Option<Decimal>,

    /// Whether this line is blank.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_blank: Option<bool>,
}

/// Collection wrapper for manual journals returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManualJournals {
    /// List of manual journals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_journals: Option<Vec<ManualJournal>>,

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
    fn manual_journal_deserialize_basic() {
        let json = r#"{
            "ManualJournalID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
            "Date": "/Date(1539993600000+0000)/",
            "Status": "POSTED",
            "Narration": "Year end adjustment",
            "Url": "https://example.com/journal",
            "ShowOnCashBasisReports": true,
            "HasAttachments": false,
            "UpdatedDateUTC": "/Date(1573755038314)/",
            "JournalLines": [
                {
                    "LineAmount": 1000.00,
                    "AccountCode": "200",
                    "Description": "Revenue adjustment",
                    "TaxType": "OUTPUT",
                    "TaxAmount": 150.00,
                    "IsBlank": false
                },
                {
                    "LineAmount": -1000.00,
                    "AccountCode": "400",
                    "Description": "Expense adjustment",
                    "TaxType": "INPUT",
                    "TaxAmount": -150.00,
                    "IsBlank": false
                }
            ]
        }"#;
        let mj: ManualJournal = serde_json::from_str(json).unwrap();
        assert_eq!(mj.narration.as_deref(), Some("Year end adjustment"));
        assert_eq!(mj.status, Some(ManualJournalStatus::Posted));
        assert_eq!(mj.show_on_cash_basis_reports, Some(true));
        assert_eq!(mj.has_attachments, Some(false));

        let lines = mj.journal_lines.as_ref().unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].line_amount, Some(Decimal::new(1000, 0)));
        assert_eq!(lines[0].account_code.as_deref(), Some("200"));
        assert_eq!(lines[1].line_amount, Some(Decimal::new(-1000, 0)));
    }

    #[test]
    fn manual_journals_collection_with_pagination() {
        let json = r#"{
            "ManualJournals": [{
                "ManualJournalID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "Narration": "Test journal",
                "Status": "DRAFT"
            }],
            "pagination": {"Page": 1, "PageSize": 100, "PageCount": 1, "ItemCount": 1}
        }"#;
        let mjs: ManualJournals = serde_json::from_str(json).unwrap();
        assert_eq!(mjs.manual_journals.as_ref().unwrap().len(), 1);
        assert_eq!(mjs.pagination.as_ref().unwrap().page, Some(1));
    }
}
