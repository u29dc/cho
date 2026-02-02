//! Large enumeration types used across Xero API models.
//!
//! All enums include an `Unknown` catch-all variant via `#[serde(other)]`
//! for forward compatibility with new values added by Xero.

use serde::{Deserialize, Serialize};

// ── Invoice enums ──

/// Type of invoice.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceType {
    /// Accounts receivable (sales invoice).
    #[serde(rename = "ACCREC")]
    AccRec,
    /// Accounts payable (purchase invoice / bill).
    #[serde(rename = "ACCPAY")]
    AccPay,
    /// Unknown type (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Status of an invoice.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceStatus {
    /// Draft invoice.
    Draft,
    /// Submitted for approval.
    Submitted,
    /// Deleted invoice.
    Deleted,
    /// Approved and awaiting payment.
    Authorised,
    /// Fully paid.
    Paid,
    /// Voided invoice.
    Voided,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// How line amounts are handled on transactions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LineAmountTypes {
    /// Line amounts are exclusive of tax.
    Exclusive,
    /// Line amounts are inclusive of tax.
    Inclusive,
    /// No tax applied.
    NoTax,
    /// Unknown (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Contact enums ──

/// Status of a contact.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContactStatus {
    /// Active contact.
    Active,
    /// Archived contact.
    Archived,
    /// Deleted contact (GDPR).
    #[serde(rename = "GDPRREQUEST")]
    GdprRequest,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Bank Transaction enums ──

/// Type of bank transaction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BankTransactionType {
    /// Receive money.
    Receive,
    /// Spend money.
    Spend,
    /// Receive overpayment.
    #[serde(rename = "RECEIVE-OVERPAYMENT")]
    ReceiveOverpayment,
    /// Receive prepayment.
    #[serde(rename = "RECEIVE-PREPAYMENT")]
    ReceivePrepayment,
    /// Spend overpayment.
    #[serde(rename = "SPEND-OVERPAYMENT")]
    SpendOverpayment,
    /// Spend prepayment.
    #[serde(rename = "SPEND-PREPAYMENT")]
    SpendPrepayment,
    /// Receive transfer.
    #[serde(rename = "RECEIVE-TRANSFER")]
    ReceiveTransfer,
    /// Spend transfer.
    #[serde(rename = "SPEND-TRANSFER")]
    SpendTransfer,
    /// Unknown type (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Status of a bank transaction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BankTransactionStatus {
    /// Authorised transaction.
    Authorised,
    /// Deleted transaction.
    Deleted,
    /// Voided transaction.
    Voided,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Payment enums ──

/// Status of a payment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    /// Authorised payment.
    Authorised,
    /// Deleted payment.
    Deleted,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Type of payment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentType {
    /// Accounts receivable payment.
    #[serde(rename = "ACCRECPAYMENT")]
    AccRecPayment,
    /// Accounts payable payment.
    #[serde(rename = "ACCPAYPAYMENT")]
    AccPayPayment,
    /// AR credit payment.
    #[serde(rename = "ARCREDITPAYMENT")]
    ArCreditPayment,
    /// AP credit payment.
    #[serde(rename = "APCREDITPAYMENT")]
    ApCreditPayment,
    /// AR overpayment payment.
    #[serde(rename = "AROVERPAYMENTPAYMENT")]
    ArOverpaymentPayment,
    /// AP overpayment payment.
    #[serde(rename = "APOVERPAYMENTPAYMENT")]
    ApOverpaymentPayment,
    /// AR prepayment payment.
    #[serde(rename = "ARPREPAYMENTPAYMENT")]
    ArPrepaymentPayment,
    /// AP prepayment payment.
    #[serde(rename = "APPREPAYMENTPAYMENT")]
    ApPrepaymentPayment,
    /// Unknown type (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Account enums ──

/// Type of account.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    /// Bank account.
    Bank,
    /// Current asset.
    Current,
    /// Current liability.
    Currliab,
    /// Depreciation.
    Depreciatn,
    /// Direct costs.
    #[serde(rename = "DIRECTCOSTS")]
    DirectCosts,
    /// Equity.
    Equity,
    /// Expense.
    Expense,
    /// Fixed asset.
    Fixed,
    /// Inventory.
    Inventory,
    /// Liability.
    Liability,
    /// Non-current asset.
    Noncurrent,
    /// Other income.
    Otherincome,
    /// Overheads.
    Overheads,
    /// Prepayment.
    Prepayment,
    /// Revenue.
    Revenue,
    /// Sales.
    Sales,
    /// Non-current liability.
    #[serde(rename = "TERMLIAB")]
    TermLiab,
    /// Payable/receivable on tracking.
    #[serde(rename = "PAYGLIABILITY")]
    PaygLiability,
    /// Superannuation expense.
    #[serde(rename = "SUPERANNUATIONEXPENSE")]
    SuperannuationExpense,
    /// Superannuation liability.
    #[serde(rename = "SUPERANNUATIONLIABILITY")]
    SuperannuationLiability,
    /// Wages expense.
    #[serde(rename = "WAGESEXPENSE")]
    WagesExpense,
    /// Unknown type (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Class of account.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountClass {
    /// Asset class.
    Asset,
    /// Equity class.
    Equity,
    /// Expense class.
    Expense,
    /// Liability class.
    Liability,
    /// Revenue class.
    Revenue,
    /// Unknown class (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Status of an account.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountStatus {
    /// Active account.
    Active,
    /// Archived account.
    Archived,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Currency Code (ISO 4217, ~170 variants) ──

/// ISO 4217 currency codes used by Xero.
///
/// Includes ~170 standard currency codes plus Xero's `EMPTY_CURRENCY` sentinel.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum CurrencyCode {
    AED,
    AFN,
    ALL,
    AMD,
    ANG,
    AOA,
    ARS,
    AUD,
    AWG,
    AZN,
    BAM,
    BBD,
    BDT,
    BGN,
    BHD,
    BIF,
    BMD,
    BND,
    BOB,
    BRL,
    BSD,
    BTN,
    BWP,
    BYN,
    BZD,
    CAD,
    CDF,
    CHF,
    CLP,
    CNY,
    COP,
    CRC,
    CUP,
    CVE,
    CZK,
    DJF,
    DKK,
    DOP,
    DZD,
    EGP,
    ERN,
    ETB,
    EUR,
    FJD,
    FKP,
    GBP,
    GEL,
    GHS,
    GIP,
    GMD,
    GNF,
    GTQ,
    GYD,
    HKD,
    HNL,
    HRK,
    HTG,
    HUF,
    IDR,
    ILS,
    INR,
    IQD,
    IRR,
    ISK,
    JMD,
    JOD,
    JPY,
    KES,
    KGS,
    KHR,
    KMF,
    KPW,
    KRW,
    KWD,
    KYD,
    KZT,
    LAK,
    LBP,
    LKR,
    LRD,
    LSL,
    LYD,
    MAD,
    MDL,
    MGA,
    MKD,
    MMK,
    MNT,
    MOP,
    MRU,
    MUR,
    MVR,
    MWK,
    MXN,
    MYR,
    MZN,
    NAD,
    NGN,
    NIO,
    NOK,
    NPR,
    NZD,
    OMR,
    PAB,
    PEN,
    PGK,
    PHP,
    PKR,
    PLN,
    PYG,
    QAR,
    RON,
    RSD,
    RUB,
    RWF,
    SAR,
    SBD,
    SCR,
    SDG,
    SEK,
    SGD,
    SHP,
    SLE,
    SLL,
    SOS,
    SRD,
    SSP,
    STN,
    SVC,
    SYP,
    SZL,
    THB,
    TJS,
    TMT,
    TND,
    TOP,
    TRY,
    TTD,
    TWD,
    TZS,
    UAH,
    UGX,
    USD,
    UYU,
    UZS,
    VES,
    VND,
    VUV,
    WST,
    XAF,
    XCD,
    XOF,
    XPF,
    YER,
    ZAR,
    ZMW,
    ZWL,
    /// Xero sentinel for empty/unset currency.
    #[serde(rename = "")]
    EmptyCurrency,
    /// Unknown currency code (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Tax Type (subset for Tier 1) ──

/// Tax types used by Xero.
///
/// This covers the most common tax types. Xero has ~130+ tax types including
/// year-suffixed variants (e.g., `INPUTY23`, `INPUTY24`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaxType {
    /// Output tax (sales).
    #[serde(rename = "OUTPUT")]
    Output,
    /// Output tax v2.
    #[serde(rename = "OUTPUT2")]
    Output2,
    /// Input tax (purchases).
    #[serde(rename = "INPUT")]
    Input,
    /// Input tax v2.
    #[serde(rename = "INPUT2")]
    Input2,
    /// Capital expenses input tax.
    #[serde(rename = "CAPEXINPUT")]
    CapexInput,
    /// Capital expenses input tax v2.
    #[serde(rename = "CAPEXINPUT2")]
    CapexInput2,
    /// Exempt output.
    #[serde(rename = "EXEMPTOUTPUT")]
    ExemptOutput,
    /// Exempt input.
    #[serde(rename = "EXEMPTINPUT")]
    ExemptInput,
    /// Zero-rated output.
    #[serde(rename = "ZERORATEDOUTPUT")]
    ZeroRatedOutput,
    /// Zero-rated input.
    #[serde(rename = "ZERORATEDINPUT")]
    ZeroRatedInput,
    /// Reduced-rate output.
    #[serde(rename = "RRINPUT")]
    RrInput,
    /// Reduced-rate input.
    #[serde(rename = "RROUTPUT")]
    RrOutput,
    /// GST on income.
    #[serde(rename = "GSTONIMPORTS")]
    GstOnImports,
    /// No tax.
    #[serde(rename = "NONE")]
    None,
    /// Zero-rated EC services.
    #[serde(rename = "ECZROUTPUT")]
    EcZrOutput,
    /// Zero-rated EC services input.
    #[serde(rename = "ECZROUTPUTSERVICES")]
    EcZrOutputServices,
    /// EC acquisitions.
    #[serde(rename = "ECACQUISITIONS")]
    EcAcquisitions,
    /// Unknown tax type (forward compatibility).
    ///
    /// Captures year-suffixed variants and any new types.
    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoice_type_serde() {
        assert_eq!(
            serde_json::from_str::<InvoiceType>(r#""ACCREC""#).unwrap(),
            InvoiceType::AccRec
        );
        assert_eq!(
            serde_json::from_str::<InvoiceType>(r#""ACCPAY""#).unwrap(),
            InvoiceType::AccPay
        );
        assert_eq!(
            serde_json::from_str::<InvoiceType>(r#""SOMETHING""#).unwrap(),
            InvoiceType::Unknown
        );
    }

    #[test]
    fn invoice_status_serde() {
        assert_eq!(
            serde_json::from_str::<InvoiceStatus>(r#""DRAFT""#).unwrap(),
            InvoiceStatus::Draft
        );
        assert_eq!(
            serde_json::from_str::<InvoiceStatus>(r#""AUTHORISED""#).unwrap(),
            InvoiceStatus::Authorised
        );
        assert_eq!(
            serde_json::from_str::<InvoiceStatus>(r#""PAID""#).unwrap(),
            InvoiceStatus::Paid
        );
    }

    #[test]
    fn bank_transaction_type_hyphenated() {
        assert_eq!(
            serde_json::from_str::<BankTransactionType>(r#""RECEIVE-OVERPAYMENT""#).unwrap(),
            BankTransactionType::ReceiveOverpayment
        );
        assert_eq!(
            serde_json::from_str::<BankTransactionType>(r#""SPEND-PREPAYMENT""#).unwrap(),
            BankTransactionType::SpendPrepayment
        );
    }

    #[test]
    fn line_amount_types_pascal_case() {
        assert_eq!(
            serde_json::from_str::<LineAmountTypes>(r#""Exclusive""#).unwrap(),
            LineAmountTypes::Exclusive
        );
        assert_eq!(
            serde_json::from_str::<LineAmountTypes>(r#""Inclusive""#).unwrap(),
            LineAmountTypes::Inclusive
        );
        assert_eq!(
            serde_json::from_str::<LineAmountTypes>(r#""NoTax""#).unwrap(),
            LineAmountTypes::NoTax
        );
    }

    #[test]
    fn currency_code_common() {
        assert_eq!(
            serde_json::from_str::<CurrencyCode>(r#""USD""#).unwrap(),
            CurrencyCode::USD
        );
        assert_eq!(
            serde_json::from_str::<CurrencyCode>(r#""NZD""#).unwrap(),
            CurrencyCode::NZD
        );
        assert_eq!(
            serde_json::from_str::<CurrencyCode>(r#""EUR""#).unwrap(),
            CurrencyCode::EUR
        );
    }

    #[test]
    fn currency_code_empty_sentinel() {
        assert_eq!(
            serde_json::from_str::<CurrencyCode>(r#""""#).unwrap(),
            CurrencyCode::EmptyCurrency
        );
    }

    #[test]
    fn currency_code_unknown() {
        assert_eq!(
            serde_json::from_str::<CurrencyCode>(r#""XYZ999""#).unwrap(),
            CurrencyCode::Unknown
        );
    }

    #[test]
    fn tax_type_known_and_unknown() {
        assert_eq!(
            serde_json::from_str::<TaxType>(r#""OUTPUT""#).unwrap(),
            TaxType::Output
        );
        assert_eq!(
            serde_json::from_str::<TaxType>(r#""NONE""#).unwrap(),
            TaxType::None
        );
        // Year-suffixed variant falls through to Unknown
        assert_eq!(
            serde_json::from_str::<TaxType>(r#""INPUTY24""#).unwrap(),
            TaxType::Unknown
        );
    }

    #[test]
    fn account_type_serde() {
        assert_eq!(
            serde_json::from_str::<AccountType>(r#""BANK""#).unwrap(),
            AccountType::Bank
        );
        assert_eq!(
            serde_json::from_str::<AccountType>(r#""REVENUE""#).unwrap(),
            AccountType::Revenue
        );
        assert_eq!(
            serde_json::from_str::<AccountType>(r#""DIRECTCOSTS""#).unwrap(),
            AccountType::DirectCosts
        );
    }

    #[test]
    fn contact_status_gdpr() {
        assert_eq!(
            serde_json::from_str::<ContactStatus>(r#""GDPRREQUEST""#).unwrap(),
            ContactStatus::GdprRequest
        );
    }

    #[test]
    fn payment_status_serde() {
        assert_eq!(
            serde_json::from_str::<PaymentStatus>(r#""AUTHORISED""#).unwrap(),
            PaymentStatus::Authorised
        );
        assert_eq!(
            serde_json::from_str::<PaymentStatus>(r#""DELETED""#).unwrap(),
            PaymentStatus::Deleted
        );
    }
}
