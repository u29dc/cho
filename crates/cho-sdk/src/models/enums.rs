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

// ── Credit Note enums ──

/// Type of credit note.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreditNoteType {
    /// Accounts payable credit (supplier credit).
    #[serde(rename = "ACCPAYCREDIT")]
    AccPayCredit,
    /// Accounts receivable credit (customer credit).
    #[serde(rename = "ACCRECCREDIT")]
    AccRecCredit,
    /// Unknown type (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Status of a credit note.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreditNoteStatus {
    /// Draft credit note.
    Draft,
    /// Submitted for approval.
    Submitted,
    /// Approved credit note.
    Authorised,
    /// Fully paid/allocated credit note.
    Paid,
    /// Voided credit note.
    Voided,
    /// Deleted credit note.
    Deleted,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Quote enums ──

/// Status of a quote.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuoteStatus {
    /// Draft quote.
    Draft,
    /// Sent to contact.
    Sent,
    /// Declined by contact.
    Declined,
    /// Accepted by contact.
    Accepted,
    /// Converted to invoice.
    Invoiced,
    /// Deleted quote.
    Deleted,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Purchase Order enums ──

/// Type of purchase order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PurchaseOrderType {
    /// Standard purchase order.
    #[serde(rename = "PURCHASEORDER")]
    PurchaseOrder,
    /// Unknown type (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Status of a purchase order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PurchaseOrderStatus {
    /// Draft purchase order.
    Draft,
    /// Submitted for approval.
    Submitted,
    /// Approved purchase order.
    Authorised,
    /// Billed (converted to bill).
    Billed,
    /// Deleted purchase order.
    Deleted,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Tax Rate enums ──

/// Status of a tax rate.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaxRateStatus {
    /// Active tax rate.
    Active,
    /// Deleted tax rate.
    Deleted,
    /// Archived tax rate.
    Archived,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Tracking Category enums ──

/// Status of a tracking category.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrackingCategoryStatus {
    /// Active tracking category.
    Active,
    /// Archived tracking category.
    Archived,
    /// Deleted tracking category.
    Deleted,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Status of a tracking option.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrackingOptionStatus {
    /// Active tracking option.
    Active,
    /// Archived tracking option.
    Archived,
    /// Deleted tracking option.
    Deleted,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Manual Journal enums ──

/// Status of a manual journal.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ManualJournalStatus {
    /// Draft journal.
    Draft,
    /// Posted journal.
    Posted,
    /// Deleted journal.
    Deleted,
    /// Voided journal.
    Voided,
    /// Unknown status (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Organisation enums ──

/// Type of organisation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrganisationType {
    /// Company.
    Company,
    /// Charity.
    Charity,
    /// Club or society.
    #[serde(rename = "CLUBSOCIETY")]
    ClubSociety,
    /// Partnership.
    Partnership,
    /// Practice.
    Practice,
    /// Person.
    Person,
    /// Sole trader.
    #[serde(rename = "SOLETRADER")]
    SoleTrader,
    /// Trust.
    Trust,
    /// Unknown type (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Xero subscription class of an organisation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrganisationClass {
    /// Demo organisation.
    Demo,
    /// Trial organisation.
    Trial,
    /// Starter plan.
    Starter,
    /// Standard plan.
    Standard,
    /// Premium plan.
    Premium,
    /// Premium 20 plan.
    #[serde(rename = "PREMIUM_20")]
    Premium20,
    /// Premium 50 plan.
    #[serde(rename = "PREMIUM_50")]
    Premium50,
    /// Premium 100 plan.
    #[serde(rename = "PREMIUM_100")]
    Premium100,
    /// Ledger plan.
    Ledger,
    /// GST cashbook.
    #[serde(rename = "GST_CASHBOOK")]
    GstCashbook,
    /// Non-GST cashbook.
    #[serde(rename = "NON_GST_CASHBOOK")]
    NonGstCashbook,
    /// Ultimate plan.
    Ultimate,
    /// Unknown class (forward compatibility).
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

// ── Prepayment/Overpayment type enums ──

/// Type of prepayment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrepaymentType {
    /// Receive prepayment.
    #[serde(rename = "RECEIVE-PREPAYMENT")]
    ReceivePrepayment,
    /// Spend prepayment.
    #[serde(rename = "SPEND-PREPAYMENT")]
    SpendPrepayment,
    /// Accounts receivable prepayment.
    #[serde(rename = "ARPREPAYMENT")]
    ArPrepayment,
    /// Accounts payable prepayment.
    #[serde(rename = "APPREPAYMENT")]
    ApPrepayment,
    /// Unknown type (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// Type of overpayment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OverpaymentType {
    /// Receive overpayment.
    #[serde(rename = "RECEIVE-OVERPAYMENT")]
    ReceiveOverpayment,
    /// Spend overpayment.
    #[serde(rename = "SPEND-OVERPAYMENT")]
    SpendOverpayment,
    /// Accounts receivable overpayment.
    #[serde(rename = "AROVERPAYMENT")]
    ArOverpayment,
    /// Accounts payable overpayment.
    #[serde(rename = "APROVERPAYMENT")]
    ApOverpayment,
    /// Unknown type (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── Country Code (ISO 3166-1 alpha-2, ~60 common codes) ──

/// ISO 3166-1 alpha-2 country codes used by Xero.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CountryCode {
    /// United States.
    #[serde(rename = "US")]
    UnitedStates,
    /// United Kingdom.
    #[serde(rename = "GB")]
    UnitedKingdom,
    /// New Zealand.
    #[serde(rename = "NZ")]
    NewZealand,
    /// Australia.
    #[serde(rename = "AU")]
    Australia,
    /// Canada.
    #[serde(rename = "CA")]
    Canada,
    /// Germany.
    #[serde(rename = "DE")]
    Germany,
    /// France.
    #[serde(rename = "FR")]
    France,
    /// Italy.
    #[serde(rename = "IT")]
    Italy,
    /// Spain.
    #[serde(rename = "ES")]
    Spain,
    /// Japan.
    #[serde(rename = "JP")]
    Japan,
    /// China.
    #[serde(rename = "CN")]
    China,
    /// India.
    #[serde(rename = "IN")]
    India,
    /// Brazil.
    #[serde(rename = "BR")]
    Brazil,
    /// Mexico.
    #[serde(rename = "MX")]
    Mexico,
    /// South Africa.
    #[serde(rename = "ZA")]
    SouthAfrica,
    /// Singapore.
    #[serde(rename = "SG")]
    Singapore,
    /// Hong Kong.
    #[serde(rename = "HK")]
    HongKong,
    /// Ireland.
    #[serde(rename = "IE")]
    Ireland,
    /// Netherlands.
    #[serde(rename = "NL")]
    Netherlands,
    /// Belgium.
    #[serde(rename = "BE")]
    Belgium,
    /// Austria.
    #[serde(rename = "AT")]
    Austria,
    /// Switzerland.
    #[serde(rename = "CH")]
    Switzerland,
    /// Sweden.
    #[serde(rename = "SE")]
    Sweden,
    /// Norway.
    #[serde(rename = "NO")]
    Norway,
    /// Denmark.
    #[serde(rename = "DK")]
    Denmark,
    /// Finland.
    #[serde(rename = "FI")]
    Finland,
    /// Portugal.
    #[serde(rename = "PT")]
    Portugal,
    /// Greece.
    #[serde(rename = "GR")]
    Greece,
    /// Poland.
    #[serde(rename = "PL")]
    Poland,
    /// Czech Republic.
    #[serde(rename = "CZ")]
    CzechRepublic,
    /// Hungary.
    #[serde(rename = "HU")]
    Hungary,
    /// Romania.
    #[serde(rename = "RO")]
    Romania,
    /// Bulgaria.
    #[serde(rename = "BG")]
    Bulgaria,
    /// Croatia.
    #[serde(rename = "HR")]
    Croatia,
    /// Slovenia.
    #[serde(rename = "SI")]
    Slovenia,
    /// Slovakia.
    #[serde(rename = "SK")]
    Slovakia,
    /// Lithuania.
    #[serde(rename = "LT")]
    Lithuania,
    /// Latvia.
    #[serde(rename = "LV")]
    Latvia,
    /// Estonia.
    #[serde(rename = "EE")]
    Estonia,
    /// Malta.
    #[serde(rename = "MT")]
    Malta,
    /// Cyprus.
    #[serde(rename = "CY")]
    Cyprus,
    /// Luxembourg.
    #[serde(rename = "LU")]
    Luxembourg,
    /// Iceland.
    #[serde(rename = "IS")]
    Iceland,
    /// Malaysia.
    #[serde(rename = "MY")]
    Malaysia,
    /// Thailand.
    #[serde(rename = "TH")]
    Thailand,
    /// Philippines.
    #[serde(rename = "PH")]
    Philippines,
    /// Indonesia.
    #[serde(rename = "ID")]
    Indonesia,
    /// Vietnam.
    #[serde(rename = "VN")]
    Vietnam,
    /// South Korea.
    #[serde(rename = "KR")]
    SouthKorea,
    /// Taiwan.
    #[serde(rename = "TW")]
    Taiwan,
    /// United Arab Emirates.
    #[serde(rename = "AE")]
    UnitedArabEmirates,
    /// Saudi Arabia.
    #[serde(rename = "SA")]
    SaudiArabia,
    /// Qatar.
    #[serde(rename = "QA")]
    Qatar,
    /// Kuwait.
    #[serde(rename = "KW")]
    Kuwait,
    /// Bahrain.
    #[serde(rename = "BH")]
    Bahrain,
    /// Oman.
    #[serde(rename = "OM")]
    Oman,
    /// Israel.
    #[serde(rename = "IL")]
    Israel,
    /// Turkey.
    #[serde(rename = "TR")]
    Turkey,
    /// Egypt.
    #[serde(rename = "EG")]
    Egypt,
    /// Kenya.
    #[serde(rename = "KE")]
    Kenya,
    /// Nigeria.
    #[serde(rename = "NG")]
    Nigeria,
    /// Ghana.
    #[serde(rename = "GH")]
    Ghana,
    /// Unknown country code (forward compatibility).
    #[serde(other)]
    Unknown,
}

// ── TimeZone (Windows timezone identifiers used by Xero) ──

/// Windows timezone identifiers used by Xero API.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TimeZone {
    /// New Zealand Standard Time (Pacific/Auckland).
    #[serde(rename = "NEWZEALANDSTANDARDTIME")]
    NewZealandStandard,
    /// AUS Eastern Standard Time (Australia/Sydney).
    #[serde(rename = "AUSEASTERNSTANDARDTIME")]
    AusEasternStandard,
    /// AUS Central Standard Time (Australia/Adelaide).
    #[serde(rename = "AUSCENTRALSTANDARDTIME")]
    AusCentralStandard,
    /// AUS Western Standard Time (Australia/Perth).
    #[serde(rename = "AUSWESTERNSTANDARDTIME")]
    AusWesternStandard,
    /// Tasmania Standard Time (Australia/Hobart).
    #[serde(rename = "TASMANIASTANDARDTIME")]
    TasmaniaStandard,
    /// GMT Standard Time (Europe/London).
    #[serde(rename = "GMTSTANDARDTIME")]
    GmtStandard,
    /// Greenwich Mean Time (Atlantic/Reykjavik).
    #[serde(rename = "GREENWICHMEANTIME")]
    GreenwichMeanTime,
    /// Pacific Standard Time (America/Los_Angeles).
    #[serde(rename = "PACIFICSTANDARDTIME")]
    PacificStandard,
    /// Mountain Standard Time (America/Denver).
    #[serde(rename = "MOUNTAINSTANDARDTIME")]
    MountainStandard,
    /// Central Standard Time (America/Chicago).
    #[serde(rename = "CENTRALSTANDARDTIME")]
    CentralStandard,
    /// Eastern Standard Time (America/New_York).
    #[serde(rename = "EASTERNSTANDARDTIME")]
    EasternStandard,
    /// Atlantic Standard Time (America/Halifax).
    #[serde(rename = "ATLANTICSTANDARDTIME")]
    AtlanticStandard,
    /// Newfoundland Standard Time (America/St_Johns).
    #[serde(rename = "NEWFOUNDLANDSTANDARDTIME")]
    NewfoundlandStandard,
    /// Hawaiian Standard Time (Pacific/Honolulu).
    #[serde(rename = "HAWAIIANSTANDARDTIME")]
    HawaiianStandard,
    /// Alaskan Standard Time (America/Anchorage).
    #[serde(rename = "ALASKANSTANDARDTIME")]
    AlaskanStandard,
    /// UTC (Coordinated Universal Time).
    #[serde(rename = "UTC")]
    Utc,
    /// UTC-12 (Dateline Standard Time).
    #[serde(rename = "UTCM12")]
    UtcMinus12,
    /// Tokyo Standard Time (Asia/Tokyo).
    #[serde(rename = "TOKYOSTANDARDTIME")]
    TokyoStandard,
    /// China Standard Time (Asia/Shanghai).
    #[serde(rename = "CHINASTANDARDTIME")]
    ChinaStandard,
    /// India Standard Time (Asia/Kolkata).
    #[serde(rename = "INDIASTANDARDTIME")]
    IndiaStandard,
    /// Singapore Standard Time (Asia/Singapore).
    #[serde(rename = "SINGAPORESTANDARDTIME")]
    SingaporeStandard,
    /// Korea Standard Time (Asia/Seoul).
    #[serde(rename = "KOREASTANDARDTIME")]
    KoreaStandard,
    /// Taipei Standard Time (Asia/Taipei).
    #[serde(rename = "TAIPEISTANDARTIME")]
    TaipeiStandard,
    /// West Asia Standard Time (Asia/Tashkent).
    #[serde(rename = "WESTASIASTANDARDTIME")]
    WestAsiaStandard,
    /// Central Asia Standard Time (Asia/Almaty).
    #[serde(rename = "CENTRALASIASTANDARDTIME")]
    CentralAsiaStandard,
    /// Arabian Standard Time (Asia/Dubai).
    #[serde(rename = "ARABIANSTANDARDTIME")]
    ArabianStandard,
    /// Russian Standard Time (Europe/Moscow).
    #[serde(rename = "RUSSIANSTANDARDTIME")]
    RussianStandard,
    /// W. Europe Standard Time (Europe/Berlin).
    #[serde(rename = "WESTEUROPESTANDARDTIME")]
    WestEuropeStandard,
    /// Central Europe Standard Time (Europe/Budapest).
    #[serde(rename = "CENTRALEUROPESTANDARDTIME")]
    CentralEuropeStandard,
    /// E. Europe Standard Time (Europe/Bucharest).
    #[serde(rename = "EASTEUROPESTANDARDTIME")]
    EastEuropeStandard,
    /// Romance Standard Time (Europe/Paris).
    #[serde(rename = "ROMANCESTANDARDTIME")]
    RomanceStandard,
    /// SA Eastern Standard Time (America/Cayenne).
    #[serde(rename = "SAEASTERNSTANDARDTIME")]
    SaEasternStandard,
    /// SA Pacific Standard Time (America/Bogota).
    #[serde(rename = "SAPACIFICSTANDARDTIME")]
    SaPacificStandard,
    /// SA Western Standard Time (America/La_Paz).
    #[serde(rename = "SAWESTERNSTANDARDTIME")]
    SaWesternStandard,
    /// South Africa Standard Time (Africa/Johannesburg).
    #[serde(rename = "SOUTHAFRICASTANDARDTIME")]
    SouthAfricaStandard,
    /// Unknown timezone (forward compatibility).
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
    fn credit_note_type_serde() {
        assert_eq!(
            serde_json::from_str::<CreditNoteType>(r#""ACCPAYCREDIT""#).unwrap(),
            CreditNoteType::AccPayCredit
        );
        assert_eq!(
            serde_json::from_str::<CreditNoteType>(r#""ACCRECCREDIT""#).unwrap(),
            CreditNoteType::AccRecCredit
        );
        assert_eq!(
            serde_json::from_str::<CreditNoteType>(r#""SOMETHING""#).unwrap(),
            CreditNoteType::Unknown
        );
    }

    #[test]
    fn credit_note_status_serde() {
        assert_eq!(
            serde_json::from_str::<CreditNoteStatus>(r#""DRAFT""#).unwrap(),
            CreditNoteStatus::Draft
        );
        assert_eq!(
            serde_json::from_str::<CreditNoteStatus>(r#""AUTHORISED""#).unwrap(),
            CreditNoteStatus::Authorised
        );
        assert_eq!(
            serde_json::from_str::<CreditNoteStatus>(r#""PAID""#).unwrap(),
            CreditNoteStatus::Paid
        );
        assert_eq!(
            serde_json::from_str::<CreditNoteStatus>(r#""VOIDED""#).unwrap(),
            CreditNoteStatus::Voided
        );
    }

    #[test]
    fn quote_status_serde() {
        assert_eq!(
            serde_json::from_str::<QuoteStatus>(r#""DRAFT""#).unwrap(),
            QuoteStatus::Draft
        );
        assert_eq!(
            serde_json::from_str::<QuoteStatus>(r#""SENT""#).unwrap(),
            QuoteStatus::Sent
        );
        assert_eq!(
            serde_json::from_str::<QuoteStatus>(r#""ACCEPTED""#).unwrap(),
            QuoteStatus::Accepted
        );
        assert_eq!(
            serde_json::from_str::<QuoteStatus>(r#""INVOICED""#).unwrap(),
            QuoteStatus::Invoiced
        );
    }

    #[test]
    fn purchase_order_status_serde() {
        assert_eq!(
            serde_json::from_str::<PurchaseOrderStatus>(r#""DRAFT""#).unwrap(),
            PurchaseOrderStatus::Draft
        );
        assert_eq!(
            serde_json::from_str::<PurchaseOrderStatus>(r#""BILLED""#).unwrap(),
            PurchaseOrderStatus::Billed
        );
        assert_eq!(
            serde_json::from_str::<PurchaseOrderType>(r#""PURCHASEORDER""#).unwrap(),
            PurchaseOrderType::PurchaseOrder
        );
    }

    #[test]
    fn tax_rate_status_serde() {
        assert_eq!(
            serde_json::from_str::<TaxRateStatus>(r#""ACTIVE""#).unwrap(),
            TaxRateStatus::Active
        );
        assert_eq!(
            serde_json::from_str::<TaxRateStatus>(r#""DELETED""#).unwrap(),
            TaxRateStatus::Deleted
        );
        assert_eq!(
            serde_json::from_str::<TaxRateStatus>(r#""ARCHIVED""#).unwrap(),
            TaxRateStatus::Archived
        );
    }

    #[test]
    fn tracking_category_status_serde() {
        assert_eq!(
            serde_json::from_str::<TrackingCategoryStatus>(r#""ACTIVE""#).unwrap(),
            TrackingCategoryStatus::Active
        );
        assert_eq!(
            serde_json::from_str::<TrackingCategoryStatus>(r#""ARCHIVED""#).unwrap(),
            TrackingCategoryStatus::Archived
        );
    }

    #[test]
    fn manual_journal_status_serde() {
        assert_eq!(
            serde_json::from_str::<ManualJournalStatus>(r#""DRAFT""#).unwrap(),
            ManualJournalStatus::Draft
        );
        assert_eq!(
            serde_json::from_str::<ManualJournalStatus>(r#""POSTED""#).unwrap(),
            ManualJournalStatus::Posted
        );
        assert_eq!(
            serde_json::from_str::<ManualJournalStatus>(r#""VOIDED""#).unwrap(),
            ManualJournalStatus::Voided
        );
    }

    #[test]
    fn organisation_type_serde() {
        assert_eq!(
            serde_json::from_str::<OrganisationType>(r#""COMPANY""#).unwrap(),
            OrganisationType::Company
        );
        assert_eq!(
            serde_json::from_str::<OrganisationType>(r#""SOLETRADER""#).unwrap(),
            OrganisationType::SoleTrader
        );
        assert_eq!(
            serde_json::from_str::<OrganisationType>(r#""CLUBSOCIETY""#).unwrap(),
            OrganisationType::ClubSociety
        );
    }

    #[test]
    fn organisation_class_serde() {
        assert_eq!(
            serde_json::from_str::<OrganisationClass>(r#""DEMO""#).unwrap(),
            OrganisationClass::Demo
        );
        assert_eq!(
            serde_json::from_str::<OrganisationClass>(r#""PREMIUM_20""#).unwrap(),
            OrganisationClass::Premium20
        );
        assert_eq!(
            serde_json::from_str::<OrganisationClass>(r#""GST_CASHBOOK""#).unwrap(),
            OrganisationClass::GstCashbook
        );
        assert_eq!(
            serde_json::from_str::<OrganisationClass>(r#""SOMETHING_NEW""#).unwrap(),
            OrganisationClass::Unknown
        );
    }

    #[test]
    fn country_code_common() {
        assert_eq!(
            serde_json::from_str::<CountryCode>(r#""US""#).unwrap(),
            CountryCode::UnitedStates
        );
        assert_eq!(
            serde_json::from_str::<CountryCode>(r#""GB""#).unwrap(),
            CountryCode::UnitedKingdom
        );
        assert_eq!(
            serde_json::from_str::<CountryCode>(r#""NZ""#).unwrap(),
            CountryCode::NewZealand
        );
        assert_eq!(
            serde_json::from_str::<CountryCode>(r#""AU""#).unwrap(),
            CountryCode::Australia
        );
        assert_eq!(
            serde_json::from_str::<CountryCode>(r#""DE""#).unwrap(),
            CountryCode::Germany
        );
        assert_eq!(
            serde_json::from_str::<CountryCode>(r#""JP""#).unwrap(),
            CountryCode::Japan
        );
        assert_eq!(
            serde_json::from_str::<CountryCode>(r#""SG""#).unwrap(),
            CountryCode::Singapore
        );
        assert_eq!(
            serde_json::from_str::<CountryCode>(r#""AE""#).unwrap(),
            CountryCode::UnitedArabEmirates
        );
    }

    #[test]
    fn country_code_unknown() {
        assert_eq!(
            serde_json::from_str::<CountryCode>(r#""XX""#).unwrap(),
            CountryCode::Unknown
        );
        assert_eq!(
            serde_json::from_str::<CountryCode>(r#""INVALID""#).unwrap(),
            CountryCode::Unknown
        );
    }

    #[test]
    fn country_code_round_trip() {
        let nz = CountryCode::NewZealand;
        let json = serde_json::to_string(&nz).unwrap();
        assert_eq!(json, r#""NZ""#);
        let parsed: CountryCode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, CountryCode::NewZealand);
    }

    #[test]
    fn timezone_common() {
        assert_eq!(
            serde_json::from_str::<TimeZone>(r#""NEWZEALANDSTANDARDTIME""#).unwrap(),
            TimeZone::NewZealandStandard
        );
        assert_eq!(
            serde_json::from_str::<TimeZone>(r#""AUSEASTERNSTANDARDTIME""#).unwrap(),
            TimeZone::AusEasternStandard
        );
        assert_eq!(
            serde_json::from_str::<TimeZone>(r#""GMTSTANDARDTIME""#).unwrap(),
            TimeZone::GmtStandard
        );
        assert_eq!(
            serde_json::from_str::<TimeZone>(r#""PACIFICSTANDARDTIME""#).unwrap(),
            TimeZone::PacificStandard
        );
        assert_eq!(
            serde_json::from_str::<TimeZone>(r#""EASTERNSTANDARDTIME""#).unwrap(),
            TimeZone::EasternStandard
        );
        assert_eq!(
            serde_json::from_str::<TimeZone>(r#""UTC""#).unwrap(),
            TimeZone::Utc
        );
        assert_eq!(
            serde_json::from_str::<TimeZone>(r#""INDIASTANDARDTIME""#).unwrap(),
            TimeZone::IndiaStandard
        );
        assert_eq!(
            serde_json::from_str::<TimeZone>(r#""SOUTHAFRICASTANDARDTIME""#).unwrap(),
            TimeZone::SouthAfricaStandard
        );
    }

    #[test]
    fn timezone_unknown() {
        assert_eq!(
            serde_json::from_str::<TimeZone>(r#""SOMECUSTOMTIMEZONE""#).unwrap(),
            TimeZone::Unknown
        );
    }

    #[test]
    fn timezone_round_trip() {
        let tz = TimeZone::NewZealandStandard;
        let json = serde_json::to_string(&tz).unwrap();
        assert_eq!(json, r#""NEWZEALANDSTANDARDTIME""#);
        let parsed: TimeZone = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, TimeZone::NewZealandStandard);
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
