//! Tax rate model for the Xero TaxRates API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::common::{Pagination, Warning};
use super::enums::TaxRateStatus;

/// A tax rate in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TaxRate {
    /// Name of the tax rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Tax type identifier (e.g., "OUTPUT", "INPUT").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_type: Option<String>,

    /// Components that make up this tax rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_components: Option<Vec<TaxComponent>>,

    /// Tax rate status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaxRateStatus>,

    /// Report tax type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_tax_type: Option<String>,

    /// Whether this tax rate can apply to asset accounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_apply_to_assets: Option<bool>,

    /// Whether this tax rate can apply to equity accounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_apply_to_equity: Option<bool>,

    /// Whether this tax rate can apply to expense accounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_apply_to_expenses: Option<bool>,

    /// Whether this tax rate can apply to liability accounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_apply_to_liabilities: Option<bool>,

    /// Whether this tax rate can apply to revenue accounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_apply_to_revenue: Option<bool>,

    /// Display tax rate (effective combined rate).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_tax_rate: Option<Decimal>,

    /// Effective rate (may differ from display rate for compound taxes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_rate: Option<Decimal>,
}

/// A component of a tax rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TaxComponent {
    /// Name of the tax component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Rate of this component (percentage).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<Decimal>,

    /// Whether this component is a compound tax.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_compound: Option<bool>,

    /// Whether this tax component is non-recoverable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_non_recoverable: Option<bool>,
}

/// Collection wrapper for tax rates returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TaxRates {
    /// List of tax rates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_rates: Option<Vec<TaxRate>>,

    /// Pagination metadata (tax rates endpoint is not paginated, but included for consistency).
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
    fn tax_rate_deserialize_basic() {
        let json = r#"{
            "Name": "15% GST on Income",
            "TaxType": "OUTPUT",
            "Status": "ACTIVE",
            "ReportTaxType": "OUTPUT",
            "CanApplyToAssets": false,
            "CanApplyToEquity": false,
            "CanApplyToExpenses": false,
            "CanApplyToLiabilities": false,
            "CanApplyToRevenue": true,
            "DisplayTaxRate": 15.0000,
            "EffectiveRate": 15.0000,
            "TaxComponents": [
                {
                    "Name": "GST",
                    "Rate": 15.0000,
                    "IsCompound": false,
                    "IsNonRecoverable": false
                }
            ]
        }"#;
        let tax_rate: TaxRate = serde_json::from_str(json).unwrap();
        assert_eq!(tax_rate.name.as_deref(), Some("15% GST on Income"));
        assert_eq!(tax_rate.tax_type.as_deref(), Some("OUTPUT"));
        assert_eq!(tax_rate.status, Some(TaxRateStatus::Active));
        assert_eq!(tax_rate.can_apply_to_revenue, Some(true));
        assert_eq!(tax_rate.can_apply_to_expenses, Some(false));
        assert_eq!(tax_rate.display_tax_rate, Some(Decimal::new(150000, 4)));

        let components = tax_rate.tax_components.as_ref().unwrap();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].name.as_deref(), Some("GST"));
        assert_eq!(components[0].is_compound, Some(false));
    }

    #[test]
    fn tax_rates_collection() {
        let json = r#"{
            "TaxRates": [
                {
                    "Name": "15% GST on Income",
                    "TaxType": "OUTPUT",
                    "Status": "ACTIVE"
                },
                {
                    "Name": "15% GST on Expenses",
                    "TaxType": "INPUT",
                    "Status": "ACTIVE"
                }
            ]
        }"#;
        let rates: TaxRates = serde_json::from_str(json).unwrap();
        assert_eq!(rates.tax_rates.as_ref().unwrap().len(), 2);
    }
}
