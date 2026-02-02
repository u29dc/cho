//! Report models for the Xero Reports API.
//!
//! Xero reports return tabular data as `Rows`/`Cells`/`Attributes` rather than
//! structured objects. This module provides both:
//! - Raw [`Report`] struct that mirrors the API response exactly
//! - Typed report structs ([`BalanceSheetReport`], [`ProfitAndLossReport`],
//!   [`TrialBalanceReport`]) that parse the tabular data into structured sections

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A raw report as returned by the Xero Reports API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Report {
    /// Report ID.
    #[serde(rename = "ReportID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_id: Option<String>,

    /// Report name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_name: Option<String>,

    /// Report type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_type: Option<String>,

    /// Report titles (array of title lines).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_titles: Option<Vec<String>>,

    /// Report date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_date: Option<String>,

    /// Last updated timestamp.
    #[serde(rename = "UpdatedDateUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date_utc: Option<super::dates::MsDateTime>,

    /// Report rows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<ReportRow>>,
}

/// A row in a Xero report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReportRow {
    /// Row type: "Header", "Section", "Row", "SummaryRow".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_type: Option<RowType>,

    /// Title for section rows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Cells in this row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cells: Option<Vec<ReportCell>>,

    /// Nested rows (for Section type rows).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<ReportRow>>,
}

/// A cell in a report row.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReportCell {
    /// Cell value (always a string in the API).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Cell attributes (e.g., account ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<CellAttribute>>,
}

/// An attribute on a report cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CellAttribute {
    /// Attribute ID (e.g., "account").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Attribute value (e.g., account UUID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Type of report row.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RowType {
    /// Header row with column labels.
    Header,
    /// Section grouping (e.g., "Assets", "Revenue").
    Section,
    /// Data row.
    Row,
    /// Summary/total row.
    SummaryRow,
    /// Unknown row type (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Collection wrapper for reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Reports {
    /// List of reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reports: Option<Vec<Report>>,
}

// ── Typed Report Structs ──

/// A parsed balance sheet report.
#[derive(Debug, Clone)]
pub struct BalanceSheetReport {
    /// Report title and date info.
    pub title: String,
    /// Report date.
    pub date: String,
    /// Asset line items.
    pub assets: Vec<ReportLineItem>,
    /// Liability line items.
    pub liabilities: Vec<ReportLineItem>,
    /// Equity line items.
    pub equity: Vec<ReportLineItem>,
    /// Total assets.
    pub total_assets: Option<Decimal>,
    /// Total liabilities.
    pub total_liabilities: Option<Decimal>,
    /// Total equity.
    pub total_equity: Option<Decimal>,
}

/// A parsed profit and loss report.
#[derive(Debug, Clone)]
pub struct ProfitAndLossReport {
    /// Report title.
    pub title: String,
    /// Report date range.
    pub date: String,
    /// Income line items.
    pub income: Vec<ReportLineItem>,
    /// Expense (cost of sales) line items.
    pub cost_of_sales: Vec<ReportLineItem>,
    /// Operating expense line items.
    pub operating_expenses: Vec<ReportLineItem>,
    /// Total income.
    pub total_income: Option<Decimal>,
    /// Total cost of sales.
    pub total_cost_of_sales: Option<Decimal>,
    /// Total operating expenses.
    pub total_operating_expenses: Option<Decimal>,
    /// Net profit.
    pub net_profit: Option<Decimal>,
}

/// A parsed trial balance report.
#[derive(Debug, Clone)]
pub struct TrialBalanceReport {
    /// Report title.
    pub title: String,
    /// Report date.
    pub date: String,
    /// Line items with debit/credit columns.
    pub line_items: Vec<TrialBalanceLineItem>,
}

/// A single line item in a typed report.
#[derive(Debug, Clone)]
pub struct ReportLineItem {
    /// Account name or description.
    pub name: String,
    /// Amount value.
    pub amount: Option<Decimal>,
    /// Account ID (if available from cell attributes).
    pub account_id: Option<String>,
}

/// A trial balance line item with debit and credit columns.
#[derive(Debug, Clone)]
pub struct TrialBalanceLineItem {
    /// Account name.
    pub name: String,
    /// Debit amount.
    pub debit: Option<Decimal>,
    /// Credit amount.
    pub credit: Option<Decimal>,
    /// YTD debit.
    pub ytd_debit: Option<Decimal>,
    /// YTD credit.
    pub ytd_credit: Option<Decimal>,
    /// Account ID (if available).
    pub account_id: Option<String>,
}

// ── Parsing helpers ──

/// Extract the first cell value from a report row.
fn first_cell_value(row: &ReportRow) -> Option<&str> {
    row.cells.as_ref()?.first()?.value.as_deref()
}

/// Extract account ID from the first cell's attributes.
fn first_cell_account_id(row: &ReportRow) -> Option<String> {
    let attrs = row.cells.as_ref()?.first()?.attributes.as_ref()?;
    attrs
        .iter()
        .find(|a| a.id.as_deref() == Some("account"))
        .and_then(|a| a.value.clone())
}

/// Parse a decimal from a cell value string.
fn parse_decimal(s: &str) -> Option<Decimal> {
    // Remove commas and whitespace that Xero sometimes includes
    let cleaned = s.replace(',', "").trim().to_owned();
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse::<Decimal>().ok()
}

/// Extract a specific cell value by index from a report row.
fn cell_value_at(row: &ReportRow, index: usize) -> Option<&str> {
    row.cells.as_ref()?.get(index)?.value.as_deref()
}

/// Parse report line items from a section's nested rows.
///
/// Assumes the amount is in the second cell (index 1).
fn parse_section_items(section: &ReportRow) -> Vec<ReportLineItem> {
    let Some(rows) = section.rows.as_ref() else {
        return vec![];
    };
    rows.iter()
        .filter(|r| r.row_type == Some(RowType::Row))
        .filter_map(|r| {
            let name = first_cell_value(r)?.to_owned();
            let amount = cell_value_at(r, 1).and_then(parse_decimal);
            let account_id = first_cell_account_id(r);
            Some(ReportLineItem {
                name,
                amount,
                account_id,
            })
        })
        .collect()
}

/// Extract total from a section's SummaryRow.
fn parse_section_total(section: &ReportRow) -> Option<Decimal> {
    let rows = section.rows.as_ref()?;
    let summary = rows
        .iter()
        .find(|r| r.row_type == Some(RowType::SummaryRow))?;
    cell_value_at(summary, 1).and_then(parse_decimal)
}

impl BalanceSheetReport {
    /// Parse a [`BalanceSheetReport`] from a raw [`Report`].
    pub fn from_report(report: &Report) -> Option<Self> {
        let titles = report.report_titles.as_ref()?;
        let title = titles.first().cloned().unwrap_or_default();
        let date = titles.get(1).cloned().unwrap_or_default();

        let rows = report.rows.as_ref()?;
        let sections: Vec<_> = rows
            .iter()
            .filter(|r| r.row_type == Some(RowType::Section))
            .collect();

        // Typically: Assets, Liabilities, Equity
        let mut assets = vec![];
        let mut liabilities = vec![];
        let mut equity = vec![];
        let mut total_assets = None;
        let mut total_liabilities = None;
        let mut total_equity = None;

        for section in &sections {
            let section_title = section.title.as_deref().unwrap_or("");
            let lower = section_title.to_lowercase();
            if lower.contains("asset") {
                assets = parse_section_items(section);
                total_assets = parse_section_total(section);
            } else if lower.contains("liabilit") {
                liabilities = parse_section_items(section);
                total_liabilities = parse_section_total(section);
            } else if lower.contains("equity") {
                equity = parse_section_items(section);
                total_equity = parse_section_total(section);
            }
        }

        Some(BalanceSheetReport {
            title,
            date,
            assets,
            liabilities,
            equity,
            total_assets,
            total_liabilities,
            total_equity,
        })
    }
}

impl ProfitAndLossReport {
    /// Parse a [`ProfitAndLossReport`] from a raw [`Report`].
    pub fn from_report(report: &Report) -> Option<Self> {
        let titles = report.report_titles.as_ref()?;
        let title = titles.first().cloned().unwrap_or_default();
        let date = titles.get(1).cloned().unwrap_or_default();

        let rows = report.rows.as_ref()?;
        let sections: Vec<_> = rows
            .iter()
            .filter(|r| r.row_type == Some(RowType::Section))
            .collect();

        let mut income = vec![];
        let mut cost_of_sales = vec![];
        let mut operating_expenses = vec![];
        let mut total_income = None;
        let mut total_cost_of_sales = None;
        let mut total_operating_expenses = None;
        let mut net_profit = None;

        for section in &sections {
            let section_title = section.title.as_deref().unwrap_or("");
            let lower = section_title.to_lowercase();
            if lower.contains("income") || lower.contains("revenue") {
                income = parse_section_items(section);
                total_income = parse_section_total(section);
            } else if lower.contains("cost of sales") || lower.contains("direct cost") {
                cost_of_sales = parse_section_items(section);
                total_cost_of_sales = parse_section_total(section);
            } else if lower.contains("expense") || lower.contains("overhead") {
                operating_expenses = parse_section_items(section);
                total_operating_expenses = parse_section_total(section);
            } else if lower.contains("net profit") || lower.contains("net loss") {
                net_profit = parse_section_total(section);
            }
        }

        Some(ProfitAndLossReport {
            title,
            date,
            income,
            cost_of_sales,
            operating_expenses,
            total_income,
            total_cost_of_sales,
            total_operating_expenses,
            net_profit,
        })
    }
}

impl TrialBalanceReport {
    /// Parse a [`TrialBalanceReport`] from a raw [`Report`].
    pub fn from_report(report: &Report) -> Option<Self> {
        let titles = report.report_titles.as_ref()?;
        let title = titles.first().cloned().unwrap_or_default();
        let date = titles.get(1).cloned().unwrap_or_default();

        let rows = report.rows.as_ref()?;
        let mut line_items = vec![];

        for section in rows.iter().filter(|r| r.row_type == Some(RowType::Section)) {
            if let Some(nested) = section.rows.as_ref() {
                for row in nested.iter().filter(|r| r.row_type == Some(RowType::Row)) {
                    let name = first_cell_value(row).unwrap_or("").to_owned();
                    let debit = cell_value_at(row, 1).and_then(parse_decimal);
                    let credit = cell_value_at(row, 2).and_then(parse_decimal);
                    let ytd_debit = cell_value_at(row, 3).and_then(parse_decimal);
                    let ytd_credit = cell_value_at(row, 4).and_then(parse_decimal);
                    let account_id = first_cell_account_id(row);

                    line_items.push(TrialBalanceLineItem {
                        name,
                        debit,
                        credit,
                        ytd_debit,
                        ytd_credit,
                        account_id,
                    });
                }
            }
        }

        Some(TrialBalanceReport {
            title,
            date,
            line_items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_raw_deserialize() {
        let json = r#"{
            "Reports": [{
                "ReportID": "BalanceSheet",
                "ReportName": "Balance Sheet",
                "ReportType": "BalanceSheet",
                "ReportTitles": ["Balance Sheet", "As at 31 October 2019"],
                "Rows": [
                    {
                        "RowType": "Header",
                        "Cells": [
                            {"Value": ""},
                            {"Value": "31 Oct 2019"}
                        ]
                    },
                    {
                        "RowType": "Section",
                        "Title": "Assets",
                        "Rows": [
                            {
                                "RowType": "Row",
                                "Cells": [
                                    {"Value": "Bank Accounts", "Attributes": [{"Id": "account", "Value": "abc-123"}]},
                                    {"Value": "15000.00"}
                                ]
                            },
                            {
                                "RowType": "SummaryRow",
                                "Cells": [
                                    {"Value": "Total Assets"},
                                    {"Value": "15000.00"}
                                ]
                            }
                        ]
                    },
                    {
                        "RowType": "Section",
                        "Title": "Liabilities",
                        "Rows": [
                            {
                                "RowType": "Row",
                                "Cells": [
                                    {"Value": "Accounts Payable"},
                                    {"Value": "3000.00"}
                                ]
                            },
                            {
                                "RowType": "SummaryRow",
                                "Cells": [
                                    {"Value": "Total Liabilities"},
                                    {"Value": "3000.00"}
                                ]
                            }
                        ]
                    },
                    {
                        "RowType": "Section",
                        "Title": "Equity",
                        "Rows": [
                            {
                                "RowType": "Row",
                                "Cells": [
                                    {"Value": "Retained Earnings"},
                                    {"Value": "12000.00"}
                                ]
                            },
                            {
                                "RowType": "SummaryRow",
                                "Cells": [
                                    {"Value": "Total Equity"},
                                    {"Value": "12000.00"}
                                ]
                            }
                        ]
                    }
                ]
            }]
        }"#;
        let reports: Reports = serde_json::from_str(json).unwrap();
        let report = &reports.reports.as_ref().unwrap()[0];
        assert_eq!(report.report_name.as_deref(), Some("Balance Sheet"));
        assert_eq!(report.rows.as_ref().unwrap().len(), 4);
    }

    #[test]
    fn balance_sheet_typed_parse() {
        let report = Report {
            report_id: Some("BalanceSheet".to_owned()),
            report_name: Some("Balance Sheet".to_owned()),
            report_type: Some("BalanceSheet".to_owned()),
            report_titles: Some(vec![
                "Balance Sheet".to_owned(),
                "As at 31 October 2019".to_owned(),
            ]),
            report_date: None,
            updated_date_utc: None,
            rows: Some(vec![
                ReportRow {
                    row_type: Some(RowType::Section),
                    title: Some("Assets".to_owned()),
                    cells: None,
                    rows: Some(vec![
                        ReportRow {
                            row_type: Some(RowType::Row),
                            title: None,
                            cells: Some(vec![
                                ReportCell {
                                    value: Some("Bank Accounts".to_owned()),
                                    attributes: Some(vec![CellAttribute {
                                        id: Some("account".to_owned()),
                                        value: Some("abc-123".to_owned()),
                                    }]),
                                },
                                ReportCell {
                                    value: Some("15000.00".to_owned()),
                                    attributes: None,
                                },
                            ]),
                            rows: None,
                        },
                        ReportRow {
                            row_type: Some(RowType::SummaryRow),
                            title: None,
                            cells: Some(vec![
                                ReportCell {
                                    value: Some("Total Assets".to_owned()),
                                    attributes: None,
                                },
                                ReportCell {
                                    value: Some("15000.00".to_owned()),
                                    attributes: None,
                                },
                            ]),
                            rows: None,
                        },
                    ]),
                },
                ReportRow {
                    row_type: Some(RowType::Section),
                    title: Some("Liabilities".to_owned()),
                    cells: None,
                    rows: Some(vec![ReportRow {
                        row_type: Some(RowType::SummaryRow),
                        title: None,
                        cells: Some(vec![
                            ReportCell {
                                value: Some("Total Liabilities".to_owned()),
                                attributes: None,
                            },
                            ReportCell {
                                value: Some("3000.00".to_owned()),
                                attributes: None,
                            },
                        ]),
                        rows: None,
                    }]),
                },
                ReportRow {
                    row_type: Some(RowType::Section),
                    title: Some("Equity".to_owned()),
                    cells: None,
                    rows: Some(vec![ReportRow {
                        row_type: Some(RowType::SummaryRow),
                        title: None,
                        cells: Some(vec![
                            ReportCell {
                                value: Some("Total Equity".to_owned()),
                                attributes: None,
                            },
                            ReportCell {
                                value: Some("12000.00".to_owned()),
                                attributes: None,
                            },
                        ]),
                        rows: None,
                    }]),
                },
            ]),
        };

        let bs = BalanceSheetReport::from_report(&report).unwrap();
        assert_eq!(bs.title, "Balance Sheet");
        assert_eq!(bs.assets.len(), 1);
        assert_eq!(bs.assets[0].name, "Bank Accounts");
        assert_eq!(bs.assets[0].amount, Some(Decimal::new(1500000, 2)));
        assert_eq!(bs.assets[0].account_id.as_deref(), Some("abc-123"));
        assert_eq!(bs.total_assets, Some(Decimal::new(1500000, 2)));
        assert_eq!(bs.total_liabilities, Some(Decimal::new(300000, 2)));
        assert_eq!(bs.total_equity, Some(Decimal::new(1200000, 2)));
    }

    #[test]
    fn profit_and_loss_typed_parse() {
        let report = Report {
            report_id: Some("ProfitAndLoss".to_owned()),
            report_name: Some("Profit and Loss".to_owned()),
            report_type: Some("ProfitAndLoss".to_owned()),
            report_titles: Some(vec![
                "Profit and Loss".to_owned(),
                "1 Jan 2019 to 31 Oct 2019".to_owned(),
            ]),
            report_date: None,
            updated_date_utc: None,
            rows: Some(vec![
                ReportRow {
                    row_type: Some(RowType::Section),
                    title: Some("Income".to_owned()),
                    cells: None,
                    rows: Some(vec![
                        ReportRow {
                            row_type: Some(RowType::Row),
                            title: None,
                            cells: Some(vec![
                                ReportCell {
                                    value: Some("Sales".to_owned()),
                                    attributes: None,
                                },
                                ReportCell {
                                    value: Some("50000.00".to_owned()),
                                    attributes: None,
                                },
                            ]),
                            rows: None,
                        },
                        ReportRow {
                            row_type: Some(RowType::SummaryRow),
                            title: None,
                            cells: Some(vec![
                                ReportCell {
                                    value: Some("Total Income".to_owned()),
                                    attributes: None,
                                },
                                ReportCell {
                                    value: Some("50000.00".to_owned()),
                                    attributes: None,
                                },
                            ]),
                            rows: None,
                        },
                    ]),
                },
                ReportRow {
                    row_type: Some(RowType::Section),
                    title: Some("Operating Expenses".to_owned()),
                    cells: None,
                    rows: Some(vec![ReportRow {
                        row_type: Some(RowType::SummaryRow),
                        title: None,
                        cells: Some(vec![
                            ReportCell {
                                value: Some("Total Expenses".to_owned()),
                                attributes: None,
                            },
                            ReportCell {
                                value: Some("30000.00".to_owned()),
                                attributes: None,
                            },
                        ]),
                        rows: None,
                    }]),
                },
            ]),
        };

        let pnl = ProfitAndLossReport::from_report(&report).unwrap();
        assert_eq!(pnl.title, "Profit and Loss");
        assert_eq!(pnl.income.len(), 1);
        assert_eq!(pnl.income[0].name, "Sales");
        assert_eq!(pnl.total_income, Some(Decimal::new(5000000, 2)));
        assert_eq!(pnl.total_operating_expenses, Some(Decimal::new(3000000, 2)));
    }

    #[test]
    fn cell_attribute_deserialize() {
        let json = r#"{"Id": "account", "Value": "abc-123-def"}"#;
        let attr: CellAttribute = serde_json::from_str(json).unwrap();
        assert_eq!(attr.id.as_deref(), Some("account"));
        assert_eq!(attr.value.as_deref(), Some("abc-123-def"));
    }

    #[test]
    fn parse_decimal_with_commas() {
        assert_eq!(parse_decimal("1,234.56"), Some(Decimal::new(123456, 2)));
        assert_eq!(parse_decimal("0.00"), Some(Decimal::new(0, 2)));
        assert_eq!(parse_decimal(""), None);
        assert_eq!(parse_decimal("-500.00"), Some(Decimal::new(-50000, 2)));
    }

    #[test]
    fn row_type_serde() {
        assert_eq!(
            serde_json::from_str::<RowType>(r#""Header""#).unwrap(),
            RowType::Header
        );
        assert_eq!(
            serde_json::from_str::<RowType>(r#""Section""#).unwrap(),
            RowType::Section
        );
        assert_eq!(
            serde_json::from_str::<RowType>(r#""Row""#).unwrap(),
            RowType::Row
        );
        assert_eq!(
            serde_json::from_str::<RowType>(r#""SummaryRow""#).unwrap(),
            RowType::SummaryRow
        );
    }
}
