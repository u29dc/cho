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
}
