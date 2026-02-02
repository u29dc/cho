//! HTTP request builder with auth and tenant header injection.
//!
//! Constructs Xero API requests with the required `Authorization`,
//! `xero-tenant-id`, and `Content-Type` headers, plus optional query
//! parameters for filtering, ordering, and pagination.

use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

/// Query parameters for list endpoints.
#[derive(Debug, Clone, Default)]
pub struct ListParams {
    /// OData-like where filter expression.
    pub where_filter: Option<String>,

    /// Order expression (e.g., "Date DESC").
    pub order: Option<String>,

    /// Page number (1-indexed).
    pub page: Option<u32>,

    /// Page size (max 100 for most endpoints).
    pub page_size: Option<u32>,

    /// If-Modified-Since header value (ISO 8601).
    pub if_modified_since: Option<String>,

    /// Whether to return summary only (fewer fields).
    pub summary_only: Option<bool>,

    /// Search term (Contacts endpoint).
    pub search_term: Option<String>,

    /// Decimal precision for unit amounts (2 or 4).
    pub unitdp: Option<u32>,
}

impl ListParams {
    /// Creates empty list parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the where filter.
    pub fn with_where(mut self, filter: impl Into<String>) -> Self {
        self.where_filter = Some(filter.into());
        self
    }

    /// Sets the order expression.
    pub fn with_order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Sets the page number.
    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the page size.
    pub fn with_page_size(mut self, size: u32) -> Self {
        self.page_size = Some(size);
        self
    }

    /// Sets summary-only mode.
    pub fn with_summary_only(mut self, summary: bool) -> Self {
        self.summary_only = Some(summary);
        self
    }

    /// Sets the search term.
    pub fn with_search_term(mut self, term: impl Into<String>) -> Self {
        self.search_term = Some(term.into());
        self
    }

    /// Builds the query string pairs for appending to the request URL.
    pub fn to_query_pairs(&self) -> Vec<(&str, String)> {
        let mut pairs = Vec::new();

        if let Some(ref w) = self.where_filter {
            pairs.push(("where", w.clone()));
        }
        if let Some(ref o) = self.order {
            pairs.push(("order", o.clone()));
        }
        if let Some(p) = self.page {
            pairs.push(("page", p.to_string()));
        }
        if let Some(ps) = self.page_size {
            pairs.push(("pageSize", ps.to_string()));
        }
        if let Some(true) = self.summary_only {
            pairs.push(("summaryOnly", "true".to_string()));
        }
        if let Some(ref term) = self.search_term {
            pairs.push(("searchTerm", term.clone()));
        }
        if let Some(udp) = self.unitdp {
            pairs.push(("unitdp", udp.to_string()));
        }

        pairs
    }
}

/// Builds the standard headers for a Xero API request.
///
/// If `if_modified_since` is provided, adds the `If-Modified-Since` header.
///
/// Returns an error if the access token or tenant ID contain non-ASCII
/// characters that cannot be represented in an HTTP header value.
pub fn build_headers(
    access_token: &str,
    tenant_id: &str,
    if_modified_since: Option<&str>,
) -> crate::error::Result<HeaderMap> {
    let mut headers = HeaderMap::new();

    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {access_token}")).map_err(|e| {
            crate::error::ChoSdkError::Config {
                message: format!("Invalid access token for HTTP header: {e}"),
            }
        })?,
    );

    headers.insert(
        "xero-tenant-id",
        HeaderValue::from_str(tenant_id).map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Invalid tenant ID for HTTP header: {e}"),
        })?,
    );

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    if let Some(since) = if_modified_since
        && let Ok(value) = HeaderValue::from_str(since)
    {
        headers.insert(reqwest::header::IF_MODIFIED_SINCE, value);
    }

    Ok(headers)
}

/// Report-specific query parameters.
#[derive(Debug, Clone, Default)]
pub struct ReportParams {
    /// Report date (YYYY-MM-DD).
    pub date: Option<String>,

    /// From date for period reports (YYYY-MM-DD).
    pub from_date: Option<String>,

    /// To date for period reports (YYYY-MM-DD).
    pub to_date: Option<String>,

    /// Number of periods.
    pub periods: Option<u32>,

    /// Timeframe (MONTH, QUARTER, YEAR).
    pub timeframe: Option<String>,

    /// Contact ID for aged reports.
    pub contact_id: Option<String>,

    /// Whether to use standard layout.
    pub standard_layout: Option<bool>,

    /// Whether to show payments for aged reports.
    pub payments_only: Option<bool>,
}

impl ReportParams {
    /// Creates empty report parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds query string pairs.
    pub fn to_query_pairs(&self) -> Vec<(&str, String)> {
        let mut pairs = Vec::new();

        if let Some(ref d) = self.date {
            pairs.push(("date", d.clone()));
        }
        if let Some(ref f) = self.from_date {
            pairs.push(("fromDate", f.clone()));
        }
        if let Some(ref t) = self.to_date {
            pairs.push(("toDate", t.clone()));
        }
        if let Some(p) = self.periods {
            pairs.push(("periods", p.to_string()));
        }
        if let Some(ref tf) = self.timeframe {
            pairs.push(("timeframe", tf.clone()));
        }
        if let Some(ref c) = self.contact_id {
            pairs.push(("contactID", c.clone()));
        }
        if let Some(true) = self.standard_layout {
            pairs.push(("standardLayout", "true".to_string()));
        }
        if let Some(true) = self.payments_only {
            pairs.push(("paymentsOnly", "true".to_string()));
        }

        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_params_empty() {
        let params = ListParams::new();
        assert!(params.to_query_pairs().is_empty());
    }

    #[test]
    fn list_params_with_filters() {
        let params = ListParams::new()
            .with_where("Status==\"AUTHORISED\"")
            .with_order("Date DESC")
            .with_page(2)
            .with_page_size(50);

        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 4);
        assert_eq!(pairs[0], ("where", "Status==\"AUTHORISED\"".to_string()));
        assert_eq!(pairs[1], ("order", "Date DESC".to_string()));
        assert_eq!(pairs[2], ("page", "2".to_string()));
        assert_eq!(pairs[3], ("pageSize", "50".to_string()));
    }

    #[test]
    fn build_headers_contains_required() {
        let headers = build_headers("test_token", "tenant-123", None).unwrap();
        assert!(headers.get(AUTHORIZATION).is_some());
        assert!(headers.get("xero-tenant-id").is_some());
        assert!(headers.get(CONTENT_TYPE).is_some());

        let auth = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
        assert_eq!(auth, "Bearer test_token");

        let tenant = headers.get("xero-tenant-id").unwrap().to_str().unwrap();
        assert_eq!(tenant, "tenant-123");
    }

    #[test]
    fn build_headers_with_if_modified_since() {
        let headers =
            build_headers("test_token", "tenant-123", Some("2024-01-01T00:00:00Z")).unwrap();
        assert!(headers.get(reqwest::header::IF_MODIFIED_SINCE).is_some());
        let since = headers
            .get(reqwest::header::IF_MODIFIED_SINCE)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(since, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn build_headers_rejects_invalid_token() {
        // HeaderValue rejects control characters like newlines
        let result = build_headers("token\ninjection", "tenant-123", None);
        assert!(result.is_err());
    }

    #[test]
    fn build_headers_rejects_invalid_tenant() {
        let result = build_headers("valid_token", "tenant\r\nid", None);
        assert!(result.is_err());
    }

    #[test]
    fn report_params_build() {
        let params = ReportParams {
            date: Some("2024-01-01".to_string()),
            periods: Some(3),
            timeframe: Some("MONTH".to_string()),
            ..Default::default()
        };
        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 3);
    }
}
