//! Organisation model for the Xero Organisation API.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Address, Pagination, Phone, Warning};
use super::dates::{MsDate, MsDateTime};
use super::enums::{CurrencyCode, OrganisationClass, OrganisationType};

/// A Xero organisation (company, sole trader, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Organisation {
    /// Unique identifier for the organisation.
    #[serde(rename = "OrganisationID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organisation_id: Option<Uuid>,

    /// Xero API key.
    #[serde(rename = "APIKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Organisation name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Legal name of the organisation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_name: Option<String>,

    /// Whether the organisation pays tax.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pays_tax: Option<bool>,

    /// Version of the organisation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Type of organisation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organisation_type: Option<OrganisationType>,

    /// Base currency of the organisation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_currency: Option<CurrencyCode>,

    /// Country code (ISO 3166 alpha-2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,

    /// Whether this is a demo company.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_demo_company: Option<bool>,

    /// Organisation status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organisation_status: Option<String>,

    /// Company registration number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_number: Option<String>,

    /// Employer identification number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employer_identification_number: Option<String>,

    /// Tax number (e.g., ABN, GST, VAT number).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_number: Option<String>,

    /// Day of the financial year end (1-31).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_year_end_day: Option<i32>,

    /// Month of the financial year end (1-12).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_year_end_month: Option<i32>,

    /// Sales tax basis (e.g., "Payments", "Invoice", "None").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_tax_basis: Option<String>,

    /// Sales tax period (e.g., "MONTHLY", "QUARTERLY").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_tax_period: Option<String>,

    /// Default sales tax type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_sales_tax: Option<String>,

    /// Default purchases tax type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_purchases_tax: Option<String>,

    /// Period lock date (transactions before this date cannot be edited).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_lock_date: Option<MsDate>,

    /// End of year lock date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_of_year_lock_date: Option<MsDate>,

    /// Date the organisation was created in Xero.
    #[serde(rename = "CreatedDateUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date_utc: Option<MsDateTime>,

    /// Timezone of the organisation (IANA timezone name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    /// Organisation entity type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organisation_entity_type: Option<String>,

    /// Short code for the organisation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_code: Option<String>,

    /// Xero subscription class.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<OrganisationClass>,

    /// Xero edition (e.g., "BUSINESS", "PARTNER").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,

    /// Line of business.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_of_business: Option<String>,

    /// Addresses for the organisation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<Address>>,

    /// Phone numbers for the organisation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phones: Option<Vec<Phone>>,

    /// External links associated with the organisation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_links: Option<Vec<ExternalLink>>,
}

/// An external link associated with an organisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExternalLink {
    /// Type of link (e.g., "Facebook", "Twitter", "Website").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_type: Option<String>,

    /// URL of the link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Collection wrapper for organisations returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Organisations {
    /// List of organisations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organisations: Option<Vec<Organisation>>,

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
    fn organisation_deserialize_basic() {
        let json = r#"{
            "OrganisationID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
            "Name": "Demo Company (NZ)",
            "LegalName": "Demo Company (NZ) Limited",
            "PaysTax": true,
            "Version": "NZ",
            "OrganisationType": "COMPANY",
            "BaseCurrency": "NZD",
            "CountryCode": "NZ",
            "IsDemoCompany": true,
            "OrganisationStatus": "ACTIVE",
            "RegistrationNumber": "1234567",
            "TaxNumber": "12-345-678",
            "FinancialYearEndDay": 31,
            "FinancialYearEndMonth": 3,
            "SalesTaxBasis": "Payments",
            "SalesTaxPeriod": "TWOMONTHS",
            "DefaultSalesTax": "OUTPUT2",
            "DefaultPurchasesTax": "INPUT2",
            "CreatedDateUTC": "/Date(1573755038314)/",
            "Timezone": "NEWZEALANDSTANDARDTIME",
            "ShortCode": "!7L4db",
            "Class": "DEMO",
            "Edition": "BUSINESS",
            "Addresses": [
                {"AddressType": "POBOX", "City": "Wellington", "Country": "New Zealand"}
            ],
            "Phones": [
                {"PhoneType": "DEFAULT", "PhoneNumber": "1234567"}
            ],
            "ExternalLinks": [
                {"LinkType": "Facebook", "Url": "https://facebook.com/democompany"}
            ]
        }"#;
        let org: Organisation = serde_json::from_str(json).unwrap();
        assert_eq!(org.name.as_deref(), Some("Demo Company (NZ)"));
        assert_eq!(org.organisation_type, Some(OrganisationType::Company));
        assert_eq!(org.base_currency, Some(CurrencyCode::NZD));
        assert_eq!(org.is_demo_company, Some(true));
        assert_eq!(org.class, Some(OrganisationClass::Demo));
        assert_eq!(org.financial_year_end_day, Some(31));
        assert_eq!(org.financial_year_end_month, Some(3));
        assert_eq!(org.addresses.as_ref().unwrap().len(), 1);
        assert_eq!(org.phones.as_ref().unwrap().len(), 1);
        assert_eq!(org.external_links.as_ref().unwrap().len(), 1);
        assert_eq!(
            org.external_links.as_ref().unwrap()[0].link_type.as_deref(),
            Some("Facebook")
        );
    }

    #[test]
    fn organisations_collection() {
        let json = r#"{
            "Organisations": [{
                "OrganisationID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "Name": "Demo Company",
                "Class": "PREMIUM_20"
            }]
        }"#;
        let orgs: Organisations = serde_json::from_str(json).unwrap();
        assert_eq!(orgs.organisations.as_ref().unwrap().len(), 1);
        assert_eq!(
            orgs.organisations.as_ref().unwrap()[0].class,
            Some(OrganisationClass::Premium20)
        );
    }
}
