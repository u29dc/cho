//! Shared SDK models.

use serde::{Deserialize, Serialize};

/// Paginated list result from a FreeAgent resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResult {
    /// Collected items.
    pub items: Vec<serde_json::Value>,
    /// Total count from `X-Total-Count` when provided.
    pub total: Option<usize>,
    /// True if more pages are available.
    pub has_more: bool,
    /// Last fetched page number.
    pub page: u32,
    /// Last used page size.
    pub per_page: u32,
}

/// Pagination settings for list operations.
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    /// Page size to request from FreeAgent (`1..=100`).
    pub per_page: u32,
    /// Maximum total items to return (`0` means no cap).
    pub limit: usize,
    /// Fetch all pages regardless of `limit`.
    pub all: bool,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            per_page: 100,
            limit: 100,
            all: false,
        }
    }
}

impl Pagination {
    /// Returns a pagination config that fetches all pages.
    pub fn all() -> Self {
        Self {
            per_page: 100,
            limit: 0,
            all: true,
        }
    }
}

/// Auth token status summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStatus {
    /// Whether an access token is currently valid.
    pub authenticated: bool,
    /// Access token expiry timestamp in RFC 3339 format.
    pub expires_at: Option<String>,
    /// Approximate seconds remaining.
    pub expires_in_seconds: Option<i64>,
    /// High-level token lifecycle state.
    pub token_state: Option<String>,
    /// Whether a refresh token is available and likely valid.
    pub can_refresh: Option<bool>,
    /// Whether the token is close enough to expiry that refresh is advisable.
    pub needs_refresh: Option<bool>,
}

/// Trusted auth/session status summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    /// Whether the API session was confirmed usable.
    pub authenticated: bool,
    /// Alias for authenticated for readability in finance/health surfaces.
    pub session_usable: bool,
    /// Whether the cached access token was valid before any refresh attempt.
    pub cached_authenticated: bool,
    /// Access token expiry timestamp in RFC 3339 format.
    pub expires_at: Option<String>,
    /// Approximate seconds remaining.
    pub expires_in_seconds: Option<i64>,
    /// High-level token lifecycle state.
    pub token_state: String,
    /// Whether a refresh token is available and likely valid.
    pub can_refresh: bool,
    /// Whether a refresh was attempted as part of the trusted check.
    pub refresh_attempted: bool,
    /// Whether the attempted refresh succeeded.
    pub refresh_succeeded: bool,
    /// Ordered list of checks that contributed to the decision.
    pub checked_via: Vec<String>,
    /// Probe endpoint used to confirm the session can read data.
    pub probe_endpoint: Option<String>,
    /// Probe/refresh error when the session could not be confirmed.
    pub probe_error: Option<String>,
}

/// Evidence item used for reconciliation output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentaryEvidence {
    /// Evidence source kind.
    pub source: String,
    /// Related URL when available.
    pub url: Option<String>,
    /// Evidence date when available.
    pub dated_on: Option<String>,
    /// Evidence amount when available.
    pub amount: Option<String>,
    /// Human-readable description when available.
    pub description: Option<String>,
}

/// Status-trust fields attached to tax payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxStatusTrust {
    /// FreeAgent-reported status or closest inferred equivalent.
    pub system_status: String,
    /// System that reported the status.
    pub status_source: String,
    /// True when bank evidence matched the obligation.
    pub bank_reconciled: bool,
    /// Convenience inverse of bank_reconciled.
    pub not_bank_reconciled: bool,
    /// Evidence references gathered during reconciliation.
    pub documentary_evidence: Vec<DocumentaryEvidence>,
    /// Confidence label for downstream agents.
    pub confidence: String,
    /// Human-readable warning or reconciliation note.
    pub warning: Option<String>,
}

/// Unified tax-calendar entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxCalendarEntry {
    /// Obligation kind (`corporation-tax`, `vat`, `payroll`, `self-assessment`, ...).
    pub kind: String,
    /// Human-readable label.
    pub label: String,
    /// Source tool or endpoint family.
    pub source_tool: String,
    /// Normalized date for calendar ordering and rendering.
    pub event_date: Option<String>,
    /// Event classification (`payment_event`, `refund_event`, `filing_event`, `status_record`).
    pub event_type: String,
    /// True when the item represents a cash obligation.
    pub is_cash_obligation: bool,
    /// True when the item represents a filing/submission obligation.
    pub is_filing_obligation: bool,
    /// Whether the current data source can support bank reconciliation.
    pub can_bank_reconcile: bool,
    /// Optional period end date.
    pub period_ends_on: Option<String>,
    /// Optional due date used for calendar ordering.
    pub due_on: Option<String>,
    /// Optional amount due.
    pub amount: Option<String>,
    /// Status-trust fields.
    pub status_trust: TaxStatusTrust,
    /// Raw source record retained for auditing/debugging.
    pub raw: serde_json::Value,
}

/// Tax-calendar response payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxCalendar {
    /// Ordered obligation entries.
    pub items: Vec<TaxCalendarEntry>,
}

/// Reconciliation state for one obligation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationItem {
    /// Obligation calendar entry.
    pub obligation: TaxCalendarEntry,
    /// Final reconciliation status (`matched`, `unmatched`, `ambiguous`, `likely_stale`,
    /// `cannot_reconcile_with_current_data_source`, `not_a_payment_obligation`).
    pub reconciliation_status: String,
    /// Candidate bank transactions that looked related but were not selected.
    pub related_candidates: Vec<DocumentaryEvidence>,
}

/// Reconciliation summary counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationSummary {
    /// Matched items count.
    pub matched: usize,
    /// Unmatched items count.
    pub unmatched: usize,
    /// Ambiguous items count.
    pub ambiguous: usize,
    /// Likely-stale items count.
    pub likely_stale: usize,
    /// Payment obligations that cannot be reconciled from current data sources.
    pub cannot_reconcile_with_current_data_source: usize,
    /// Non-cash events that were intentionally excluded from bank matching.
    pub not_a_payment_obligation: usize,
}

/// HMRC reconciliation response payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationReport {
    /// Reconciled obligations.
    pub items: Vec<ReconciliationItem>,
    /// Aggregate reconciliation counts.
    pub summary: ReconciliationSummary,
}
