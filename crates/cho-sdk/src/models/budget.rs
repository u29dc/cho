//! Budget model for the Xero Budgets API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::Warning;
use super::dates::MsDateTime;

/// A budget in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Budget {
    /// Unique identifier for the budget.
    #[serde(rename = "BudgetID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_id: Option<Uuid>,

    /// Type of budget.
    #[serde(rename = "Type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_type: Option<BudgetType>,

    /// Description of the budget.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Last updated timestamp.
    #[serde(rename = "UpdatedDateUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date_utc: Option<MsDateTime>,

    /// Budget lines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_lines: Option<Vec<BudgetLine>>,

    /// Tracking categories associated with the budget.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking: Option<Vec<BudgetTracking>>,
}

/// A budget line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BudgetLine {
    /// Account ID for this budget line.
    #[serde(rename = "AccountID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<Uuid>,

    /// Account code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_code: Option<String>,

    /// Budget balances by period.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_balances: Option<Vec<BudgetBalance>>,
}

/// A budget balance for a specific period.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BudgetBalance {
    /// Period (e.g., month start date).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,

    /// Budget amount for this period.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Decimal>,

    /// Unit amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_amount: Option<Decimal>,

    /// Notes for this period.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Tracking category reference on a budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BudgetTracking {
    /// Tracking category ID.
    #[serde(rename = "TrackingCategoryID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_category_id: Option<Uuid>,

    /// Tracking option ID.
    #[serde(rename = "TrackingOptionID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_option_id: Option<Uuid>,

    /// Tracking category name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Tracking option name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option: Option<String>,
}

/// Type of budget.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BudgetType {
    /// Overall budget.
    Overall,
    /// Tracking category budget.
    Tracking,
    /// Unknown type (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Xero API Budgets collection wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Budgets {
    /// List of budgets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budgets: Option<Vec<Budget>>,

    /// Warnings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<Warning>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_deserialize_basic() {
        let json = serde_json::json!({
            "BudgetID": "00000000-0000-0000-0000-000000000001",
            "Type": "OVERALL",
            "Description": "FY2025 Budget",
            "BudgetLines": [
                {
                    "AccountID": "00000000-0000-0000-0000-000000000002",
                    "AccountCode": "200",
                    "BudgetBalances": [
                        {
                            "Period": "2025-01",
                            "Amount": "10000.00"
                        }
                    ]
                }
            ]
        });
        let b: Budget = serde_json::from_value(json).unwrap();
        assert_eq!(
            b.budget_id.unwrap().to_string(),
            "00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(b.budget_type.unwrap(), BudgetType::Overall);
        let lines = b.budget_lines.unwrap();
        assert_eq!(lines.len(), 1);
        let balances = lines[0].budget_balances.as_ref().unwrap();
        assert_eq!(balances[0].amount.unwrap(), Decimal::new(1_000_000, 2));
    }

    #[test]
    fn budgets_collection() {
        let json = serde_json::json!({
            "Budgets": [
                {
                    "BudgetID": "00000000-0000-0000-0000-000000000001",
                    "Type": "OVERALL"
                }
            ]
        });
        let col: Budgets = serde_json::from_value(json).unwrap();
        assert_eq!(col.budgets.unwrap().len(), 1);
    }
}
