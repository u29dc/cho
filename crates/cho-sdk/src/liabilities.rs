//! Finance-oriented liabilities, status-trust, and reconciliation helpers.

use chrono::{Datelike, NaiveDate, Utc};
use serde_json::{Map, Value};

use crate::api::specs::by_name;
use crate::client::FreeAgentClient;
use crate::error::{ChoSdkError, Result};
use crate::models::{
    DocumentaryEvidence, Pagination, ReconciliationItem, ReconciliationReport,
    ReconciliationSummary, TaxCalendar, TaxCalendarEntry, TaxStatusTrust,
};

const HMRC_PAYMENT_TOKENS: &[&str] = &[
    "hmrc",
    "cumbernauld",
    "shipley",
    "corporation tax",
    "vat",
    "self assessment",
    "self-assessment",
    "paye",
    "national insurance",
];

/// Calendar assembly options.
#[derive(Debug, Clone, Default)]
pub struct TaxCalendarOptions {
    /// Optional self-assessment user id/url to merge into the calendar.
    pub user: Option<String>,
    /// Optional payroll year override.
    pub payroll_year: Option<i32>,
}

/// Reconciliation options.
#[derive(Debug, Clone)]
pub struct ReconcileOptions {
    /// Optional self-assessment user id/url to merge into the reconciliation set.
    pub user: Option<String>,
    /// Optional payroll year override.
    pub payroll_year: Option<i32>,
    /// Match window around the due date.
    pub match_window_days: i64,
}

impl Default for ReconcileOptions {
    fn default() -> Self {
        Self {
            user: None,
            payroll_year: None,
            match_window_days: 45,
        }
    }
}

fn build_calendar_entry(
    mut raw: Value,
    kind_fn: fn(&Value, &str) -> String,
    kind_fallback: &str,
    label_fallback: &str,
    source_tool: &str,
    can_bank_reconcile: bool,
) -> TaxCalendarEntry {
    let trust = annotate_tax_response(&mut raw);
    let kind = kind_fn(&raw, kind_fallback);
    let label = derive_label(&raw, label_fallback);
    let period_ends_on = extract_first_string(
        &raw,
        &["period_ends_on", "period_end", "ends_on", "year_end"],
    )
    .map(normalize_date_like);
    let due_on = extract_due_date(&raw);
    let amount = extract_amount_string(&raw);
    let classification = classify_event(
        &raw,
        &kind,
        &label,
        period_ends_on.as_deref(),
        due_on.as_deref(),
        amount.as_deref(),
        can_bank_reconcile,
    );

    TaxCalendarEntry {
        kind,
        label,
        source_tool: source_tool.to_string(),
        event_date: classification.event_date,
        event_type: classification.event_type,
        is_cash_obligation: classification.is_cash_obligation,
        is_filing_obligation: classification.is_filing_obligation,
        can_bank_reconcile: classification.can_bank_reconcile,
        period_ends_on,
        due_on,
        amount,
        status_trust: trust,
        raw,
    }
}

fn self_assessment_payment_entries(raw: &Value) -> Vec<TaxCalendarEntry> {
    let label = derive_label(raw, "Self Assessment");
    let period_ends_on = extract_first_string(raw, &["period_ends_on", "period_end", "ends_on"])
        .map(normalize_date_like);
    let base_source = "self-assessment-returns.list";

    let payment_arrays = ["payments", "payment_dates"]
        .iter()
        .filter_map(|key| raw.get(*key).and_then(Value::as_array))
        .collect::<Vec<_>>();

    let mut out = Vec::new();
    for payments in payment_arrays {
        for payment in payments {
            let payment_date = extract_due_date(payment)
                .or_else(|| extract_due_date(raw))
                .or_else(|| period_ends_on.clone());
            let amount = extract_amount_string(payment).or_else(|| extract_amount_string(raw));
            let mut merged = raw.clone();
            if let Value::Object(map) = &mut merged {
                map.insert("payment_record".to_string(), payment.clone());
                if let Some(payment_date) = &payment_date {
                    map.entry("due_on".to_string())
                        .or_insert_with(|| Value::String(payment_date.clone()));
                }
                if let Some(amount) = &amount {
                    map.entry("amount_due".to_string())
                        .or_insert_with(|| Value::String(amount.clone()));
                }
                map.insert(
                    "description".to_string(),
                    Value::String(format!("{label} payment")),
                );
            }
            out.push(build_calendar_entry(
                merged,
                |_value, _fallback| "self-assessment".to_string(),
                "self-assessment",
                "Self Assessment payment",
                base_source,
                false,
            ));
        }
    }

    out
}

#[derive(Debug, Clone)]
struct PaymentCandidate {
    evidence: DocumentaryEvidence,
    normalized_text: String,
    amount: Option<f64>,
    dated_on: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
struct EventClassification {
    event_date: Option<String>,
    event_type: String,
    is_cash_obligation: bool,
    is_filing_obligation: bool,
    can_bank_reconcile: bool,
}

/// Reusable liabilities/reconciliation surface.
pub struct LiabilitiesService<'a> {
    client: &'a FreeAgentClient,
}

impl<'a> LiabilitiesService<'a> {
    /// Creates a new liabilities service.
    pub(crate) fn new(client: &'a FreeAgentClient) -> Self {
        Self { client }
    }

    /// Builds a merged tax calendar across company, payroll, and optional self-assessment inputs.
    pub async fn tax_calendar(&self, options: &TaxCalendarOptions) -> Result<TaxCalendar> {
        let mut items = self.company_calendar_items().await?;
        items.extend(self.payroll_items(options.payroll_year).await?);

        if let Some(user) = options
            .user
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            items.extend(self.self_assessment_items(user).await?);
        }

        items.sort_by(calendar_entry_sort_key);
        Ok(TaxCalendar { items })
    }

    /// Reconciles likely HMRC-related bank payments against known tax liabilities.
    pub async fn reconcile_hmrc(&self, options: &ReconcileOptions) -> Result<ReconciliationReport> {
        let calendar = self
            .tax_calendar(&TaxCalendarOptions {
                user: options.user.clone(),
                payroll_year: options.payroll_year,
            })
            .await?;
        let payments = self.hmrc_payment_candidates().await?;

        let mut matched = 0;
        let mut unmatched = 0;
        let mut ambiguous = 0;
        let mut likely_stale = 0;
        let mut cannot_reconcile_with_current_data_source = 0;
        let mut not_a_payment_obligation = 0;
        let mut items = Vec::with_capacity(calendar.items.len());

        for entry in calendar.items {
            let item = reconcile_entry(entry, &payments, options.match_window_days);
            match item.reconciliation_status.as_str() {
                "matched" => matched += 1,
                "ambiguous" => ambiguous += 1,
                "likely_stale" => likely_stale += 1,
                "cannot_reconcile_with_current_data_source" => {
                    cannot_reconcile_with_current_data_source += 1
                }
                "not_a_payment_obligation" => not_a_payment_obligation += 1,
                _ => unmatched += 1,
            }
            items.push(item);
        }

        Ok(ReconciliationReport {
            items,
            summary: ReconciliationSummary {
                matched,
                unmatched,
                ambiguous,
                likely_stale,
                cannot_reconcile_with_current_data_source,
                not_a_payment_obligation,
            },
        })
    }

    async fn company_calendar_items(&self) -> Result<Vec<TaxCalendarEntry>> {
        let value = self.client.get_json("company/tax_timeline", &[]).await?;
        Ok(flatten_value_items(&value)
            .into_iter()
            .map(|raw| {
                build_calendar_entry(
                    raw,
                    infer_tax_kind,
                    "company-tax",
                    "Company obligation",
                    "company.tax-timeline",
                    true,
                )
            })
            .collect())
    }

    async fn self_assessment_items(&self, user: &str) -> Result<Vec<TaxCalendarEntry>> {
        let value = self
            .client
            .list_paginated(
                &format!("users/{}/self_assessment_returns", user_id_segment(user)),
                "self_assessment_returns",
                &[],
                Pagination::all(),
            )
            .await?;

        let mut out = Vec::new();
        for raw in value.items {
            let payment_entries = self_assessment_payment_entries(&raw);
            if payment_entries.is_empty() {
                out.push(build_calendar_entry(
                    raw,
                    |_value, _fallback| "self-assessment".to_string(),
                    "self-assessment",
                    "Self Assessment",
                    "self-assessment-returns.list",
                    false,
                ));
            } else {
                out.extend(payment_entries);
            }
        }

        Ok(out)
    }

    async fn payroll_items(&self, payroll_year: Option<i32>) -> Result<Vec<TaxCalendarEntry>> {
        let year = payroll_year.unwrap_or_else(|| Utc::now().year());
        let value = self
            .client
            .get_json(&format!("payroll/{year}"), &[])
            .await?;

        Ok(flatten_value_items(&value)
            .into_iter()
            .map(|raw| {
                build_calendar_entry(
                    raw,
                    |_value, _fallback| "payroll".to_string(),
                    "payroll",
                    &format!("PAYE / Payroll {year}"),
                    "payroll.periods",
                    true,
                )
            })
            .collect())
    }

    async fn hmrc_payment_candidates(&self) -> Result<Vec<PaymentCandidate>> {
        let bank_accounts_spec = by_name("bank-accounts").ok_or_else(|| ChoSdkError::Config {
            message: "Missing bank-accounts resource spec".to_string(),
        })?;
        let bank_transactions_spec =
            by_name("bank-transactions").ok_or_else(|| ChoSdkError::Config {
                message: "Missing bank-transactions resource spec".to_string(),
            })?;

        let bank_accounts = self
            .client
            .resource(bank_accounts_spec)
            .list(&[], Pagination::all())
            .await?;
        let bank_transactions_api = self.client.resource(bank_transactions_spec);

        let mut out = Vec::new();
        for account in bank_accounts.items {
            let Some(bank_account_url) = infer_item_identifier(&account) else {
                continue;
            };

            let result = bank_transactions_api
                .list(
                    &[("bank_account".to_string(), bank_account_url.clone())],
                    Pagination::all(),
                )
                .await?;

            for item in result.items {
                if let Some(candidate) = payment_candidate_from_value(&item)
                    && looks_like_hmrc_payment(&candidate.normalized_text)
                {
                    out.push(candidate);
                }
            }
        }

        Ok(out)
    }
}

/// Adds status-trust fields to tax-like JSON values and returns the top-level trust summary.
pub fn annotate_tax_response(value: &mut Value) -> TaxStatusTrust {
    annotate_tax_value_recursive(value);
    build_tax_status_trust(value)
}

fn annotate_tax_value_recursive(value: &mut Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                annotate_tax_value_recursive(item);
            }
        }
        Value::Object(map) => {
            for child in map.values_mut() {
                annotate_tax_value_recursive(child);
            }

            if should_annotate_tax_object(map) {
                let trust = build_tax_status_trust(&Value::Object(map.clone()));
                insert_tax_status_trust(map, &trust);
            }
        }
        _ => {}
    }
}

fn build_tax_status_trust(value: &Value) -> TaxStatusTrust {
    let system_status = derive_system_status(value);
    let warning =
        Some("FreeAgent-reported status only; no bank reconciliation performed".to_string());

    TaxStatusTrust {
        system_status,
        status_source: "freeagent".to_string(),
        bank_reconciled: false,
        not_bank_reconciled: true,
        documentary_evidence: vec![],
        confidence: "low".to_string(),
        warning,
    }
}

fn insert_tax_status_trust(map: &mut Map<String, Value>, trust: &TaxStatusTrust) {
    map.insert(
        "system_status".to_string(),
        Value::String(trust.system_status.clone()),
    );
    map.insert(
        "status_source".to_string(),
        Value::String(trust.status_source.clone()),
    );
    map.insert(
        "bank_reconciled".to_string(),
        Value::Bool(trust.bank_reconciled),
    );
    map.insert(
        "not_bank_reconciled".to_string(),
        Value::Bool(trust.not_bank_reconciled),
    );
    map.insert(
        "documentary_evidence".to_string(),
        serde_json::to_value(&trust.documentary_evidence)
            .unwrap_or_else(|_| Value::Array(Vec::new())),
    );
    map.insert(
        "confidence".to_string(),
        Value::String(trust.confidence.clone()),
    );
    if let Some(warning) = &trust.warning {
        map.insert("status_warning".to_string(), Value::String(warning.clone()));
    }
}

fn should_annotate_tax_object(map: &Map<String, Value>) -> bool {
    [
        "status",
        "payment_status",
        "period_ends_on",
        "due_on",
        "payment_date",
        "deadline",
        "corporation_tax_return",
        "vat_return",
        "self_assessment_return",
        "payroll",
        "tax_type",
    ]
    .iter()
    .any(|key| map.contains_key(*key))
}

fn reconcile_entry(
    mut entry: TaxCalendarEntry,
    payments: &[PaymentCandidate],
    match_window_days: i64,
) -> ReconciliationItem {
    if entry.event_type != "payment_event" || !entry.is_cash_obligation {
        entry.status_trust.warning =
            Some("Not a payment obligation; excluded from HMRC bank reconciliation".to_string());
        return ReconciliationItem {
            obligation: entry,
            reconciliation_status: "not_a_payment_obligation".to_string(),
            related_candidates: vec![],
        };
    }

    if !entry.can_bank_reconcile {
        entry.status_trust.warning = Some(
            "Cannot reconcile with current data source; cho only has company-side bank evidence"
                .to_string(),
        );
        entry.status_trust.confidence = "low".to_string();
        return ReconciliationItem {
            obligation: entry,
            reconciliation_status: "cannot_reconcile_with_current_data_source".to_string(),
            related_candidates: vec![],
        };
    }

    let target_amount = entry.amount.as_deref().and_then(parse_amount_like);
    let target_date = entry
        .event_date
        .as_deref()
        .and_then(parse_date_like)
        .or_else(|| entry.period_ends_on.as_deref().and_then(parse_date_like));

    let mut exact = Vec::new();
    let mut related = Vec::new();

    for payment in payments {
        let score = match_score(
            &entry,
            payment,
            target_amount,
            target_date,
            match_window_days,
        );
        if score >= 3 {
            exact.push((score, payment.evidence.clone()));
        } else if score > 0 {
            related.push(payment.evidence.clone());
        }
    }

    exact.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.dated_on.cmp(&right.1.dated_on))
    });

    let reconciliation_status = if exact.len() > 1 {
        "ambiguous"
    } else if let Some((score, evidence)) = exact.first() {
        let stale = status_looks_unpaid(&entry.status_trust.system_status);
        entry.status_trust.status_source = "freeagent+bank_reconciliation".to_string();
        entry.status_trust.bank_reconciled = true;
        entry.status_trust.not_bank_reconciled = false;
        entry.status_trust.documentary_evidence = vec![evidence.clone()];
        entry.status_trust.confidence = if *score >= 5 {
            "high".to_string()
        } else {
            "medium".to_string()
        };
        entry.status_trust.warning = if stale {
            Some(
                "FreeAgent still reports unpaid/open, but a matching HMRC-like payment exists"
                    .to_string(),
            )
        } else {
            Some("Matched against HMRC-like bank activity".to_string())
        };

        if stale { "likely_stale" } else { "matched" }
    } else {
        entry.status_trust.warning = Some(
            "FreeAgent-reported status only; no matching HMRC-like bank payment found".to_string(),
        );
        "unmatched"
    };

    ReconciliationItem {
        obligation: entry,
        reconciliation_status: reconciliation_status.to_string(),
        related_candidates: related,
    }
}

fn match_score(
    entry: &TaxCalendarEntry,
    payment: &PaymentCandidate,
    target_amount: Option<f64>,
    target_date: Option<NaiveDate>,
    match_window_days: i64,
) -> i32 {
    let mut score = 0;

    if payment.normalized_text.contains("hmrc") {
        score += 1;
    }

    if kind_matches_text(&entry.kind, &payment.normalized_text) {
        score += 1;
    }

    if let (Some(expected), Some(actual)) = (target_amount, payment.amount)
        && (expected - actual).abs() < 0.01
    {
        score += 2;
    }

    if let (Some(expected), Some(actual)) = (target_date, payment.dated_on) {
        let delta = (expected - actual).num_days().abs();
        if delta <= match_window_days {
            score += 2;
        } else if delta <= match_window_days * 2 {
            score += 1;
        }
    }

    score
}

fn kind_matches_text(kind: &str, normalized_text: &str) -> bool {
    match kind {
        "corporation-tax" => normalized_text.contains("corporation tax"),
        "vat" => normalized_text.contains("vat"),
        "self-assessment" => {
            normalized_text.contains("self assessment")
                || normalized_text.contains("self-assessment")
        }
        "payroll" => {
            normalized_text.contains("paye")
                || normalized_text.contains("national insurance")
                || normalized_text.contains("ni")
        }
        _ => false,
    }
}

fn payment_candidate_from_value(value: &Value) -> Option<PaymentCandidate> {
    let description = payment_description(value);
    if description.trim().is_empty() {
        return None;
    }

    Some(PaymentCandidate {
        evidence: DocumentaryEvidence {
            source: "bank-transaction".to_string(),
            url: infer_item_identifier(value),
            dated_on: extract_first_string(
                value,
                &["dated_on", "payment_date", "due_on", "created_at"],
            )
            .map(normalize_date_like),
            amount: extract_amount_string(value),
            description: Some(description.clone()),
        },
        normalized_text: description.to_ascii_lowercase(),
        amount: extract_amount_string(value)
            .as_deref()
            .and_then(parse_amount_like),
        dated_on: extract_first_string(
            value,
            &["dated_on", "payment_date", "due_on", "created_at"],
        )
        .as_deref()
        .and_then(parse_date_like),
    })
}

fn looks_like_hmrc_payment(normalized_text: &str) -> bool {
    HMRC_PAYMENT_TOKENS
        .iter()
        .any(|token| normalized_text.contains(token))
}

fn payment_description(value: &Value) -> String {
    let mut parts = Vec::new();

    for key in [
        "description",
        "raw_description",
        "bank_description",
        "original_description",
    ] {
        if let Some(value) = value.get(key).and_then(Value::as_str)
            && !value.trim().is_empty()
        {
            parts.push(value.trim().to_string());
        }
    }

    if let Some(explanations) = value
        .get("bank_transaction_explanations")
        .and_then(Value::as_array)
    {
        for explanation in explanations {
            if let Some(value) = explanation.get("description").and_then(Value::as_str)
                && !value.trim().is_empty()
            {
                parts.push(value.trim().to_string());
            }
        }
    }

    parts.join(" ")
}

fn classify_event(
    raw: &Value,
    kind: &str,
    label: &str,
    period_ends_on: Option<&str>,
    due_on: Option<&str>,
    amount: Option<&str>,
    can_bank_reconcile: bool,
) -> EventClassification {
    let dated_on = extract_first_string(raw, &["dated_on", "date", "event_date"]);
    let payment_date = extract_first_string(raw, &["payment_date", "payable_on"]);
    let filing_deadline =
        extract_first_string(raw, &["filing_due_on", "deadline", "submission_due_on"]);
    let created_at = extract_first_string(raw, &["created_at"]);
    let nature = extract_first_string(raw, &["nature", "label"]).unwrap_or_default();
    let normalized = format!(
        "{} {} {} {}",
        kind,
        label,
        nature,
        extract_first_string(raw, &["description", "title", "name"]).unwrap_or_default()
    )
    .to_ascii_lowercase();

    let has_amount = amount.and_then(parse_amount_like).is_some();
    let payroll_period_record =
        raw.get("frequency").is_some() && raw.get("period").is_some() && !has_amount;
    let refund_event =
        normalized.contains("refund due") || nature.to_ascii_lowercase().contains("refund");
    let filing_event = looks_like_filing_event(&normalized);
    let status_record = looks_like_status_record(&normalized);
    let cash_event = !refund_event
        && !filing_event
        && !payroll_period_record
        && ((normalized.contains("payment") || normalized.contains("paye"))
            || kind == "self-assessment"
            || (has_amount && (kind == "corporation-tax" || kind == "payroll"))
            || (has_amount && normalized.contains("tax due")));

    let event_type = if refund_event {
        "refund_event"
    } else if cash_event {
        "payment_event"
    } else if filing_event {
        "filing_event"
    } else if payroll_period_record || status_record {
        "status_record"
    } else if due_on.is_some() && has_amount {
        "payment_event"
    } else if due_on.is_some() {
        "filing_event"
    } else {
        "status_record"
    };

    let event_date = match event_type {
        "payment_event" => due_on
            .or(payment_date.as_deref())
            .or(dated_on.as_deref())
            .or(filing_deadline.as_deref())
            .map(normalize_date_like),
        "refund_event" => due_on
            .or(payment_date.as_deref())
            .or(dated_on.as_deref())
            .or(period_ends_on)
            .map(normalize_date_like),
        "filing_event" => due_on
            .or(filing_deadline.as_deref())
            .or(dated_on.as_deref())
            .or(period_ends_on)
            .map(normalize_date_like),
        _ => dated_on
            .as_deref()
            .or(period_ends_on)
            .or(due_on)
            .or(created_at.as_deref())
            .map(normalize_date_like),
    };

    EventClassification {
        event_date,
        event_type: event_type.to_string(),
        is_cash_obligation: event_type == "payment_event",
        is_filing_obligation: event_type == "filing_event",
        can_bank_reconcile: can_bank_reconcile && event_type == "payment_event",
    }
}

fn looks_like_filing_event(normalized: &str) -> bool {
    [
        "submission due",
        "submission",
        "submit",
        "filing",
        "file by",
        "filed by",
        "companies house",
        "return due",
        "return filed",
        "final accounts",
        "confirmation statement",
    ]
    .iter()
    .any(|token| normalized.contains(token))
}

fn looks_like_status_record(normalized: &str) -> bool {
    [
        "accounting period ending",
        "period ending",
        "year ending",
        "period end",
        "year end",
    ]
    .iter()
    .any(|token| normalized.contains(token))
}

fn calendar_entry_sort_key(
    left: &TaxCalendarEntry,
    right: &TaxCalendarEntry,
) -> std::cmp::Ordering {
    match (
        left.event_date.as_deref().and_then(parse_date_like),
        right.event_date.as_deref().and_then(parse_date_like),
    ) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => left.label.cmp(&right.label),
    }
}

fn flatten_value_items(value: &Value) -> Vec<Value> {
    if let Some(items) = value.as_array() {
        return items.to_vec();
    }

    let Some(object) = value.as_object() else {
        return vec![value.clone()];
    };

    for preferred in [
        "items",
        "tax_timeline",
        "timeline",
        "events",
        "tax_events",
        "payroll",
        "periods",
        "periodic_payments",
    ] {
        if let Some(items) = object.get(preferred).and_then(Value::as_array) {
            return items.to_vec();
        }
    }

    let arrays = object
        .values()
        .filter_map(Value::as_array)
        .flat_map(|items| items.iter().cloned())
        .collect::<Vec<_>>();
    if !arrays.is_empty() {
        return arrays;
    }

    vec![value.clone()]
}

fn derive_system_status(value: &Value) -> String {
    for key in [
        "system_status",
        "status",
        "payment_status",
        "state",
        "filing_status",
    ] {
        if let Some(value) = value.get(key).and_then(Value::as_str)
            && !value.trim().is_empty()
        {
            return value.trim().to_string();
        }
    }

    for key in ["paid", "filed", "unpaid"] {
        if let Some(value) = value.get(key).and_then(Value::as_bool)
            && value
        {
            return key.to_string();
        }
    }

    "unknown".to_string()
}

fn infer_tax_kind(value: &Value, fallback: &str) -> String {
    let normalized = [
        extract_first_string(value, &["tax_type", "type", "kind"]),
        extract_first_string(value, &["nature"]),
        extract_first_string(value, &["description", "name", "title"]),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
    .to_ascii_lowercase();

    if normalized.contains("corporation") {
        "corporation-tax".to_string()
    } else if normalized.contains("vat") {
        "vat".to_string()
    } else if normalized.contains("self assessment") || normalized.contains("self-assessment") {
        "self-assessment".to_string()
    } else if normalized.contains("paye")
        || normalized.contains("payroll")
        || normalized.contains("national insurance")
        || normalized.contains("payslip")
        || normalized.contains("paye/ni")
    {
        "payroll".to_string()
    } else {
        fallback.to_string()
    }
}

fn derive_label(value: &Value, fallback: &str) -> String {
    for key in ["title", "description", "name", "tax_type", "type"] {
        if let Some(value) = value.get(key).and_then(Value::as_str)
            && !value.trim().is_empty()
        {
            return value.trim().to_string();
        }
    }

    fallback.to_string()
}

fn extract_due_date(value: &Value) -> Option<String> {
    extract_first_string(
        value,
        &[
            "due_on",
            "due_date",
            "payment_date",
            "deadline",
            "payable_on",
            "next_payment_date",
        ],
    )
    .map(normalize_date_like)
}

fn extract_first_string(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        match value.get(*key) {
            Some(Value::String(raw)) if !raw.trim().is_empty() => return Some(raw.clone()),
            Some(Value::Number(raw)) => return Some(raw.to_string()),
            _ => {}
        }
    }
    None
}

fn extract_amount_string(value: &Value) -> Option<String> {
    for key in [
        "amount_due",
        "amount",
        "value",
        "total_value",
        "gross_value",
        "payment_value",
        "outstanding_value",
        "balance",
    ] {
        match value.get(key) {
            Some(Value::String(raw)) if !raw.trim().is_empty() => return Some(raw.clone()),
            Some(Value::Number(raw)) => return Some(raw.to_string()),
            _ => {}
        }
    }
    None
}

fn infer_item_identifier(value: &Value) -> Option<String> {
    if let Some(url) = value.get("url").and_then(Value::as_str) {
        return Some(url.to_string());
    }

    if let Some(id) = value.get("id").and_then(Value::as_str) {
        return Some(id.to_string());
    }

    value
        .get("id")
        .and_then(Value::as_i64)
        .map(|id| id.to_string())
}

fn parse_amount_like(value: &str) -> Option<f64> {
    let cleaned = value.trim().replace([',', '£', '$'], "");
    cleaned.parse::<f64>().ok().map(f64::abs)
}

fn parse_date_like(value: &str) -> Option<NaiveDate> {
    let normalized = normalize_date_like(value);
    NaiveDate::parse_from_str(&normalized, "%Y-%m-%d").ok()
}

fn normalize_date_like(value: impl AsRef<str>) -> String {
    value.as_ref().chars().take(10).collect()
}

fn status_looks_unpaid(status: &str) -> bool {
    let normalized = status.to_ascii_lowercase();
    ["unpaid", "open", "overdue", "draft", "due", "pending"]
        .iter()
        .any(|token| normalized.contains(token))
}

fn user_id_segment(user: &str) -> String {
    let trimmed = user.trim().trim_end_matches('/');
    if (trimmed.starts_with("https://") || trimmed.starts_with("http://"))
        && let Some(id) = trimmed.rsplit('/').next()
    {
        return url::form_urlencoded::byte_serialize(id.as_bytes()).collect();
    }

    url::form_urlencoded::byte_serialize(trimmed.as_bytes()).collect()
}
