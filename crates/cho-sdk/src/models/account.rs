//! Account model for the Xero Chart of Accounts.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{ValidationError, Warning};
use super::enums::{AccountClass, AccountStatus, AccountType, CurrencyCode};

/// A chart of accounts entry in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Account {
    /// Unique identifier for the account.
    #[serde(rename = "AccountID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<Uuid>,

    /// Account code (e.g., "200", "400").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Account name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Account type (e.g., BANK, REVENUE, EXPENSE).
    #[serde(rename = "Type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_type: Option<AccountType>,

    /// Bank account number (for BANK type accounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account_number: Option<String>,

    /// Account status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AccountStatus>,

    /// Description of the account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Account class (ASSET, LIABILITY, EQUITY, REVENUE, EXPENSE).
    #[serde(rename = "Class")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_class: Option<AccountClass>,

    /// System account type (e.g., BANKCURRENCYGAIN).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_account: Option<String>,

    /// Whether tax applies to this account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_payments_to_account: Option<bool>,

    /// Whether to show in expense claims.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_in_expense_claims: Option<bool>,

    /// Bank account type (for BANK type accounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account_type: Option<String>,

    /// Currency code for this account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<CurrencyCode>,

    /// Tax type for the account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_type: Option<String>,

    /// Reporting code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporting_code: Option<String>,

    /// Reporting code name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporting_code_name: Option<String>,

    /// Whether the account has attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,

    /// Last updated timestamp.
    #[serde(rename = "UpdatedDateUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date_utc: Option<super::dates::MsDateTime>,

    /// Whether to add to watchlist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_to_watchlist: Option<bool>,

    /// Validation errors on this entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<ValidationError>>,
}

/// Collection wrapper for accounts returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Accounts {
    /// List of accounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<Account>>,

    /// Warnings returned by the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<Warning>>,
}

/// YTD/MTD balance information for bank accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BankAccount {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_deserialize() {
        let json = r#"{
            "AccountID": "ebd06280-af70-4bed-97c6-7451a454ad85",
            "Code": "200",
            "Name": "Sales",
            "Type": "REVENUE",
            "Status": "ACTIVE",
            "Class": "REVENUE",
            "EnablePaymentsToAccount": false,
            "ShowInExpenseClaims": false,
            "CurrencyCode": "NZD",
            "UpdatedDateUTC": "/Date(1573755038314)/"
        }"#;
        let account: Account = serde_json::from_str(json).unwrap();
        assert_eq!(account.code.as_deref(), Some("200"));
        assert_eq!(account.name.as_deref(), Some("Sales"));
        assert_eq!(account.account_type, Some(AccountType::Revenue));
        assert_eq!(account.account_class, Some(AccountClass::Revenue));
        assert_eq!(account.currency_code, Some(CurrencyCode::NZD));
    }

    #[test]
    fn accounts_collection_deserialize() {
        let json = r#"{
            "Accounts": [{
                "AccountID": "ebd06280-af70-4bed-97c6-7451a454ad85",
                "Code": "200",
                "Name": "Sales",
                "Type": "REVENUE"
            }]
        }"#;
        let accounts: Accounts = serde_json::from_str(json).unwrap();
        assert_eq!(accounts.accounts.as_ref().unwrap().len(), 1);
    }
}
