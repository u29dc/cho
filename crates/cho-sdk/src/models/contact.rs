//! Contact model for the Xero Contacts API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{
    Address, Attachment, ContactPerson, Pagination, Phone, ValidationError, Warning,
};
use super::dates::{MsDate, MsDateTime};
use super::enums::{ContactStatus, CurrencyCode};

/// A contact (customer, supplier, or both) in Xero.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Contact {
    /// Unique identifier for the contact.
    #[serde(rename = "ContactID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_id: Option<Uuid>,

    /// Xero-generated contact number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_number: Option<String>,

    /// Account number for the contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_number: Option<String>,

    /// Contact status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_status: Option<ContactStatus>,

    /// Contact name (required for creation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// First name of the contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,

    /// Last name of the contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,

    /// Email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_address: Option<String>,

    /// Skype username.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skype_user_name: Option<String>,

    /// Contact persons within this contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_persons: Option<Vec<ContactPerson>>,

    /// Bank account details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account_details: Option<String>,

    /// Tax number (e.g., ABN, GST number).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_number: Option<String>,

    /// Accounts receivable tax type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts_receivable_tax_type: Option<String>,

    /// Accounts payable tax type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts_payable_tax_type: Option<String>,

    /// Addresses for this contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<Address>>,

    /// Phone numbers for this contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phones: Option<Vec<Phone>>,

    /// Whether this contact is a supplier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_supplier: Option<bool>,

    /// Whether this contact is a customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_customer: Option<bool>,

    /// Default currency for this contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_currency: Option<CurrencyCode>,

    /// Website URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// Date of discount (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<MsDate>,

    /// Discount percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount: Option<Decimal>,

    /// Outstanding balances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balances: Option<ContactBalances>,

    /// Payment terms for accounts receivable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_terms: Option<PaymentTerms>,

    /// Whether the contact has attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,

    /// Whether the contact has validation errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_errors: Option<bool>,

    /// Attachments on the contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,

    /// Last updated timestamp.
    #[serde(rename = "UpdatedDateUTC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date_utc: Option<MsDateTime>,

    /// Xero network key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xero_network_key: Option<String>,

    /// Sales default account code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_default_account_code: Option<String>,

    /// Purchases default account code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchases_default_account_code: Option<String>,

    /// Tracking categories for the contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_tracking_categories: Option<Vec<super::common::LineItemTracking>>,

    /// Purchases tracking categories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchases_tracking_categories: Option<Vec<super::common::LineItemTracking>>,

    /// Contact groups this contact belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_groups: Option<Vec<ContactGroup>>,

    /// Validation errors on this entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<ValidationError>>,

    /// Status attribute string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_attribute_string: Option<String>,
}

/// Contact balances (receivable and payable).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContactBalances {
    /// Accounts receivable balances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts_receivable: Option<BalanceDetail>,

    /// Accounts payable balances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts_payable: Option<BalanceDetail>,
}

/// Balance detail for a contact.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BalanceDetail {
    /// Outstanding amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outstanding: Option<Decimal>,

    /// Overdue amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overdue: Option<Decimal>,
}

/// Payment terms for a contact.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentTerms {
    /// Bills payment terms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bills: Option<PaymentTerm>,

    /// Sales payment terms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales: Option<PaymentTerm>,
}

/// Individual payment term.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentTerm {
    /// Number of days.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<u32>,

    /// Payment term type.
    #[serde(rename = "Type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term_type: Option<PaymentTermType>,
}

/// Payment term type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentTermType {
    /// Days after bill date.
    Daysafterbilldate,
    /// Days after bill month.
    Daysafterbillmonth,
    /// Of current month.
    Ofcurrentmonth,
    /// Of following month.
    Offollowingmonth,
    /// Unknown (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// A contact group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContactGroup {
    /// Unique identifier for the contact group.
    #[serde(rename = "ContactGroupID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_group_id: Option<Uuid>,

    /// Name of the group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Status of the group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Collection wrapper for contacts returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Contacts {
    /// List of contacts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contacts: Option<Vec<Contact>>,

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
    fn contact_deserialize_basic() {
        let json = r#"{
            "ContactID": "a3e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
            "Name": "Acme Corp",
            "FirstName": "John",
            "LastName": "Doe",
            "EmailAddress": "john@acme.com",
            "ContactStatus": "ACTIVE",
            "IsSupplier": true,
            "IsCustomer": true,
            "DefaultCurrency": "USD",
            "UpdatedDateUTC": "/Date(1573755038314)/"
        }"#;
        let contact: Contact = serde_json::from_str(json).unwrap();
        assert_eq!(contact.name.as_deref(), Some("Acme Corp"));
        assert_eq!(contact.contact_status, Some(ContactStatus::Active));
        assert_eq!(contact.is_supplier, Some(true));
        assert_eq!(contact.default_currency, Some(CurrencyCode::USD));
    }

    #[test]
    fn contacts_collection_with_pagination() {
        let json = r#"{
            "Contacts": [{"ContactID": "a3e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c", "Name": "Test"}],
            "pagination": {"Page": 1, "PageSize": 100, "PageCount": 1, "ItemCount": 1}
        }"#;
        let contacts: Contacts = serde_json::from_str(json).unwrap();
        assert_eq!(contacts.contacts.as_ref().unwrap().len(), 1);
        assert_eq!(contacts.pagination.as_ref().unwrap().page, Some(1));
    }

    #[test]
    fn contact_with_addresses_and_phones() {
        let json = r#"{
            "ContactID": "a3e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
            "Name": "Test Co",
            "Addresses": [
                {"AddressType": "POBOX", "City": "Wellington", "Country": "New Zealand"},
                {"AddressType": "STREET", "City": "Auckland"}
            ],
            "Phones": [
                {"PhoneType": "DEFAULT", "PhoneNumber": "1234567"},
                {"PhoneType": "MOBILE", "PhoneNumber": "0211234567"}
            ]
        }"#;
        let contact: Contact = serde_json::from_str(json).unwrap();
        assert_eq!(contact.addresses.as_ref().unwrap().len(), 2);
        assert_eq!(contact.phones.as_ref().unwrap().len(), 2);
    }
}
