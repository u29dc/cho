//! XeroClient: the primary entry point for the Xero SDK.
//!
//! Provides a builder-pattern client that handles authentication, rate
//! limiting, and request dispatching. Resource-specific API handles are
//! accessed via namespaced methods: `client.invoices()`, `client.contacts()`, etc.

use std::sync::Arc;

use serde::Serialize;
use serde::de::DeserializeOwned;
use tracing::{debug, warn};

use crate::auth::AuthManager;
use crate::config::SdkConfig;
use crate::error::{ChoSdkError, Result};
use crate::http::pagination::{ListResult, PaginatedResponse, PaginationParams, has_more_pages};
use crate::http::rate_limit::{RateLimitConfig, RateLimiter};
use crate::http::request::{self, ListParams};

/// The main Xero API client.
///
/// Manages authentication, rate limiting, and provides typed access to
/// all Xero API resources.
///
/// # Example
///
/// ```rust,no_run
/// use cho_sdk::client::XeroClient;
/// use cho_sdk::config::SdkConfig;
///
/// # async fn example() -> cho_sdk::error::Result<()> {
/// let client = XeroClient::builder()
///     .client_id("your-client-id")
///     .tenant_id("your-tenant-id")
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct XeroClient {
    /// SDK configuration (base URL, timeouts, retries).
    config: SdkConfig,

    /// HTTP client for making requests.
    http_client: reqwest::Client,

    /// Authentication manager.
    auth: Arc<AuthManager>,

    /// Rate limiter.
    rate_limiter: Arc<RateLimiter>,

    /// Active tenant ID.
    tenant_id: String,
}

impl XeroClient {
    /// Returns a new client builder.
    pub fn builder() -> XeroClientBuilder {
        XeroClientBuilder::default()
    }

    /// Returns the current tenant ID.
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Returns the SDK configuration.
    pub fn config(&self) -> &SdkConfig {
        &self.config
    }

    /// Returns a reference to the auth manager.
    pub fn auth(&self) -> &AuthManager {
        &self.auth
    }

    /// Sets the active tenant ID.
    pub fn set_tenant_id(&mut self, tenant_id: impl Into<String>) {
        self.tenant_id = tenant_id.into();
    }

    // --- Resource API handles ---

    /// Returns the Invoices API handle.
    pub fn invoices(&self) -> crate::api::invoices::InvoicesApi<'_> {
        crate::api::invoices::InvoicesApi::new(self)
    }

    /// Returns the Contacts API handle.
    pub fn contacts(&self) -> crate::api::contacts::ContactsApi<'_> {
        crate::api::contacts::ContactsApi::new(self)
    }

    /// Returns the Payments API handle.
    pub fn payments(&self) -> crate::api::payments::PaymentsApi<'_> {
        crate::api::payments::PaymentsApi::new(self)
    }

    /// Returns the Bank Transactions API handle.
    pub fn bank_transactions(&self) -> crate::api::bank_transactions::BankTransactionsApi<'_> {
        crate::api::bank_transactions::BankTransactionsApi::new(self)
    }

    /// Returns the Accounts API handle.
    pub fn accounts(&self) -> crate::api::accounts::AccountsApi<'_> {
        crate::api::accounts::AccountsApi::new(self)
    }

    /// Returns the Reports API handle.
    pub fn reports(&self) -> crate::api::reports::ReportsApi<'_> {
        crate::api::reports::ReportsApi::new(self)
    }

    /// Returns the Identity API handle (connections/tenants).
    pub fn identity(&self) -> crate::api::identity::IdentityApi<'_> {
        crate::api::identity::IdentityApi::new(self)
    }

    /// Returns the Credit Notes API handle.
    pub fn credit_notes(&self) -> crate::api::credit_notes::CreditNotesApi<'_> {
        crate::api::credit_notes::CreditNotesApi::new(self)
    }

    /// Returns the Quotes API handle.
    pub fn quotes(&self) -> crate::api::quotes::QuotesApi<'_> {
        crate::api::quotes::QuotesApi::new(self)
    }

    /// Returns the Purchase Orders API handle.
    pub fn purchase_orders(&self) -> crate::api::purchase_orders::PurchaseOrdersApi<'_> {
        crate::api::purchase_orders::PurchaseOrdersApi::new(self)
    }

    /// Returns the Items API handle.
    pub fn items(&self) -> crate::api::items::ItemsApi<'_> {
        crate::api::items::ItemsApi::new(self)
    }

    /// Returns the Tax Rates API handle.
    pub fn tax_rates(&self) -> crate::api::tax_rates::TaxRatesApi<'_> {
        crate::api::tax_rates::TaxRatesApi::new(self)
    }

    /// Returns the Currencies API handle.
    pub fn currencies(&self) -> crate::api::currencies::CurrenciesApi<'_> {
        crate::api::currencies::CurrenciesApi::new(self)
    }

    /// Returns the Tracking Categories API handle.
    pub fn tracking_categories(
        &self,
    ) -> crate::api::tracking_categories::TrackingCategoriesApi<'_> {
        crate::api::tracking_categories::TrackingCategoriesApi::new(self)
    }

    /// Returns the Organisations API handle.
    pub fn organisations(&self) -> crate::api::organisations::OrganisationsApi<'_> {
        crate::api::organisations::OrganisationsApi::new(self)
    }

    /// Returns the Manual Journals API handle.
    pub fn manual_journals(&self) -> crate::api::manual_journals::ManualJournalsApi<'_> {
        crate::api::manual_journals::ManualJournalsApi::new(self)
    }

    /// Returns the Prepayments API handle.
    pub fn prepayments(&self) -> crate::api::prepayments::PrepaymentsApi<'_> {
        crate::api::prepayments::PrepaymentsApi::new(self)
    }

    /// Returns the Overpayments API handle.
    pub fn overpayments(&self) -> crate::api::overpayments::OverpaymentsApi<'_> {
        crate::api::overpayments::OverpaymentsApi::new(self)
    }

    /// Returns the Linked Transactions API handle.
    pub fn linked_transactions(
        &self,
    ) -> crate::api::linked_transactions::LinkedTransactionsApi<'_> {
        crate::api::linked_transactions::LinkedTransactionsApi::new(self)
    }

    /// Returns the Budgets API handle.
    pub fn budgets(&self) -> crate::api::budgets::BudgetsApi<'_> {
        crate::api::budgets::BudgetsApi::new(self)
    }

    /// Returns the Repeating Invoices API handle.
    pub fn repeating_invoices(&self) -> crate::api::repeating_invoices::RepeatingInvoicesApi<'_> {
        crate::api::repeating_invoices::RepeatingInvoicesApi::new(self)
    }

    // --- Internal request methods ---

    /// Makes a GET request to a Xero API endpoint with query parameters.
    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T> {
        let url = format!("{}{path}", self.config.base_url);
        self.request_with_retry(reqwest::Method::GET, &url, query, None)
            .await
    }

    /// Makes a GET request with an optional `If-Modified-Since` header.
    pub(crate) async fn get_with_modified_since<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
        if_modified_since: Option<&str>,
    ) -> Result<T> {
        let url = format!("{}{path}", self.config.base_url);
        self.request_with_retry(reqwest::Method::GET, &url, query, if_modified_since)
            .await
    }

    /// Makes a PUT request to a Xero API endpoint with a JSON body.
    ///
    /// Returns [`ChoSdkError::WriteNotAllowed`] if writes are disabled in config.
    pub(crate) async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        idempotency_key: Option<&str>,
    ) -> Result<T> {
        self.check_writes_allowed()?;
        let url = format!("{}{path}", self.config.base_url);
        self.request_with_body(reqwest::Method::PUT, &url, body, idempotency_key)
            .await
    }

    /// Makes a POST request to a Xero API endpoint with a JSON body.
    ///
    /// Returns [`ChoSdkError::WriteNotAllowed`] if writes are disabled in config.
    pub(crate) async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        idempotency_key: Option<&str>,
    ) -> Result<T> {
        self.check_writes_allowed()?;
        let url = format!("{}{path}", self.config.base_url);
        self.request_with_body(reqwest::Method::POST, &url, body, idempotency_key)
            .await
    }

    /// Checks whether write operations are allowed by the SDK configuration.
    fn check_writes_allowed(&self) -> Result<()> {
        if !self.config.allow_writes {
            return Err(ChoSdkError::WriteNotAllowed {
                message: "Write operations are disabled. Set allow_writes=true in SdkConfig or config file to enable.".to_string(),
            });
        }
        Ok(())
    }

    /// Makes a GET request to an absolute URL (e.g., Identity API).
    pub(crate) async fn get_absolute<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)],
    ) -> Result<T> {
        self.request_with_retry(reqwest::Method::GET, url, query, None)
            .await
    }

    /// Core request method with retry logic for 429 and transient errors.
    async fn request_with_retry<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        url: &str,
        query: &[(&str, String)],
        if_modified_since: Option<&str>,
    ) -> Result<T> {
        let max_retries = self.config.max_retries;
        let mut refresh_attempted = false;

        for attempt in 0..=max_retries {
            let guard = self.rate_limiter.acquire().await?;

            let access_token = self.auth.get_access_token().await?;
            let headers = request::build_headers(&access_token, &self.tenant_id, if_modified_since);

            let result = self
                .http_client
                .request(method.clone(), url)
                .headers(headers)
                .query(query)
                .send()
                .await;

            let response = match result {
                Ok(r) => r,
                Err(e) => {
                    if attempt < max_retries && is_transient_error(&e) {
                        let delay = backoff_delay(attempt);
                        warn!(
                            "Request failed (attempt {}/{}), retrying in {delay:?}: {e}",
                            attempt + 1,
                            max_retries + 1
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(ChoSdkError::Network(e));
                }
            };

            // Update rate limiter with response headers
            guard.complete(response.headers()).await;

            let status = response.status();

            // Handle specific status codes
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                if attempt < max_retries {
                    self.rate_limiter
                        .handle_rate_limited(response.headers())
                        .await?;
                    continue;
                }
                return Err(ChoSdkError::RateLimited {
                    retry_after: response
                        .headers()
                        .get("Retry-After")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(60),
                });
            }

            if status == reqwest::StatusCode::UNAUTHORIZED {
                // Try to refresh token once, regardless of which attempt we're on
                if !refresh_attempted {
                    refresh_attempted = true;
                    debug!("Got 401, attempting token refresh");
                    match self.auth.refresh().await {
                        Ok(()) => continue,
                        Err(_) => {
                            return Err(ChoSdkError::TokenExpired {
                                message: "Token expired and refresh failed".to_string(),
                            });
                        }
                    }
                }
                return Err(ChoSdkError::TokenExpired {
                    message: "Authentication failed".to_string(),
                });
            }

            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(ChoSdkError::NotFound {
                    resource: url.to_string(),
                    id: String::new(),
                });
            }

            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                let validation_errors = extract_validation_errors(&body);
                return Err(ChoSdkError::ApiError {
                    status: status.as_u16(),
                    message: body,
                    validation_errors,
                });
            }

            // Parse successful response
            let body = response.text().await.map_err(ChoSdkError::Network)?;
            return serde_json::from_str::<T>(&body).map_err(|e| ChoSdkError::Parse {
                message: format!(
                    "Failed to parse response: {e}\nBody: {}",
                    truncate(&body, 500)
                ),
            });
        }

        Err(ChoSdkError::ApiError {
            status: 0,
            message: "Max retries exceeded".to_string(),
            validation_errors: Vec::new(),
        })
    }

    /// Core request method with JSON body and retry logic for write operations.
    async fn request_with_body<T: DeserializeOwned, B: Serialize>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: &B,
        idempotency_key: Option<&str>,
    ) -> Result<T> {
        // Validate Idempotency-Key length (Xero max 128 chars per OpenAPI spec)
        if let Some(key) = idempotency_key
            && key.len() > 128
        {
            return Err(ChoSdkError::Config {
                message: format!(
                    "Idempotency-Key exceeds maximum length of 128 characters (got {})",
                    key.len()
                ),
            });
        }

        let max_retries = self.config.max_retries;
        let mut refresh_attempted = false;
        let json_body = serde_json::to_string(body).map_err(|e| ChoSdkError::Parse {
            message: format!("Failed to serialize request body: {e}"),
        })?;

        for attempt in 0..=max_retries {
            let guard = self.rate_limiter.acquire().await?;

            let access_token = self.auth.get_access_token().await?;
            let mut headers = request::build_headers(&access_token, &self.tenant_id, None);

            // Add Idempotency-Key header if provided
            if let Some(key) = idempotency_key
                && let Ok(value) = reqwest::header::HeaderValue::from_str(key)
            {
                headers.insert("Idempotency-Key", value);
            }

            let result = self
                .http_client
                .request(method.clone(), url)
                .headers(headers)
                .body(json_body.clone())
                .send()
                .await;

            let response = match result {
                Ok(r) => r,
                Err(e) => {
                    // Only retry write operations when an idempotency key is provided,
                    // to prevent duplicate mutations on transient network errors.
                    let can_retry = attempt < max_retries
                        && is_transient_error(&e)
                        && idempotency_key.is_some();
                    if can_retry {
                        let delay = backoff_delay(attempt);
                        warn!(
                            "Request failed (attempt {}/{}), retrying in {delay:?}: {e}",
                            attempt + 1,
                            max_retries + 1
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(ChoSdkError::Network(e));
                }
            };

            // Update rate limiter with response headers
            guard.complete(response.headers()).await;

            let status = response.status();

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                if attempt < max_retries {
                    self.rate_limiter
                        .handle_rate_limited(response.headers())
                        .await?;
                    continue;
                }
                return Err(ChoSdkError::RateLimited {
                    retry_after: response
                        .headers()
                        .get("Retry-After")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(60),
                });
            }

            if status == reqwest::StatusCode::UNAUTHORIZED {
                // Try to refresh token once, regardless of which attempt we're on
                if !refresh_attempted {
                    refresh_attempted = true;
                    debug!("Got 401, attempting token refresh");
                    match self.auth.refresh().await {
                        Ok(()) => continue,
                        Err(_) => {
                            return Err(ChoSdkError::TokenExpired {
                                message: "Token expired and refresh failed".to_string(),
                            });
                        }
                    }
                }
                return Err(ChoSdkError::TokenExpired {
                    message: "Authentication failed".to_string(),
                });
            }

            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(ChoSdkError::NotFound {
                    resource: url.to_string(),
                    id: String::new(),
                });
            }

            if !status.is_success() {
                let resp_body = response.text().await.unwrap_or_default();
                let validation_errors = extract_validation_errors(&resp_body);
                return Err(ChoSdkError::ApiError {
                    status: status.as_u16(),
                    message: resp_body,
                    validation_errors,
                });
            }

            // Parse successful response
            let resp_body = response.text().await.map_err(ChoSdkError::Network)?;
            return serde_json::from_str::<T>(&resp_body).map_err(|e| ChoSdkError::Parse {
                message: format!(
                    "Failed to parse response: {e}\nBody: {}",
                    truncate(&resp_body, 500)
                ),
            });
        }

        Err(ChoSdkError::ApiError {
            status: 0,
            message: "Max retries exceeded".to_string(),
            validation_errors: Vec::new(),
        })
    }

    /// Fetches all pages of a paginated endpoint, respecting the item limit.
    ///
    /// Returns a [`ListResult`] containing the collected items and the
    /// pagination metadata from the last page fetched.
    pub(crate) async fn get_all_pages<R: PaginatedResponse>(
        &self,
        path: &str,
        base_params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<R::Item>> {
        let mut all_items = Vec::new();
        let mut page: u32 = 1;
        let limit = pagination.limit;
        let mut last_pagination = None;

        loop {
            let params = base_params
                .clone()
                .with_page(page)
                .with_page_size(pagination.page_size);
            let query = params.to_query_pairs();

            let response: R = self
                .get_with_modified_since(path, &query, base_params.if_modified_since.as_deref())
                .await?;
            let pag = response.pagination().cloned();
            last_pagination = pag.clone();
            let items = response.into_items();

            let item_count = items.len();
            all_items.extend(items);

            // Check if we've hit the limit
            if limit > 0 && all_items.len() >= limit {
                all_items.truncate(limit);
                break;
            }

            // Check if there are more pages
            if item_count == 0 || !has_more_pages(pag.as_ref(), page) {
                break;
            }

            page += 1;
        }

        Ok(ListResult {
            items: all_items,
            pagination: last_pagination,
        })
    }
}

impl std::fmt::Debug for XeroClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XeroClient")
            .field("config", &self.config)
            .field("tenant_id", &self.tenant_id)
            .finish()
    }
}

/// Builder for constructing a [`XeroClient`].
#[derive(Default)]
pub struct XeroClientBuilder {
    config: Option<SdkConfig>,
    client_id: Option<String>,
    tenant_id: Option<String>,
    rate_limit_config: Option<RateLimitConfig>,
    auth_manager: Option<AuthManager>,
}

impl XeroClientBuilder {
    /// Sets the SDK configuration.
    pub fn config(mut self, config: SdkConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Sets the OAuth client ID.
    pub fn client_id(mut self, id: impl Into<String>) -> Self {
        self.client_id = Some(id.into());
        self
    }

    /// Sets the active tenant ID.
    pub fn tenant_id(mut self, id: impl Into<String>) -> Self {
        self.tenant_id = Some(id.into());
        self
    }

    /// Sets a custom rate limit configuration.
    pub fn rate_limit(mut self, config: RateLimitConfig) -> Self {
        self.rate_limit_config = Some(config);
        self
    }

    /// Sets a pre-configured auth manager.
    pub fn auth_manager(mut self, auth: AuthManager) -> Self {
        self.auth_manager = Some(auth);
        self
    }

    /// Builds the XeroClient.
    pub fn build(self) -> Result<XeroClient> {
        let config = self.config.unwrap_or_default();
        let client_id = self.client_id.unwrap_or_default();
        let tenant_id = self.tenant_id.unwrap_or_default();

        let rate_limiter = RateLimiter::new(self.rate_limit_config.unwrap_or_default());

        let auth = self
            .auth_manager
            .unwrap_or_else(|| AuthManager::new(client_id));

        let http_client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(ChoSdkError::Network)?;

        Ok(XeroClient {
            config,
            http_client,
            auth: Arc::new(auth),
            rate_limiter: Arc::new(rate_limiter),
            tenant_id,
        })
    }
}

/// Returns true if the error is transient and the request should be retried.
fn is_transient_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect()
}

/// Calculates exponential backoff delay for a given attempt.
fn backoff_delay(attempt: u32) -> std::time::Duration {
    let base = 1u64 << attempt.min(4); // 1, 2, 4, 8, 16 seconds
    std::time::Duration::from_secs(base)
}

/// Attempts to extract validation error messages from a Xero API error response.
///
/// Xero error responses may contain `ValidationErrors` arrays on resource items,
/// with each error having a `Message` field.
fn extract_validation_errors(body: &str) -> Vec<String> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(body) else {
        return Vec::new();
    };

    let mut errors = Vec::new();

    // Xero embeds ValidationErrors inside resource arrays
    if let Some(obj) = json.as_object() {
        for (_key, value) in obj {
            if let Some(items) = value.as_array() {
                for item in items {
                    if let Some(ve) = item.get("ValidationErrors").and_then(|v| v.as_array()) {
                        for err in ve {
                            if let Some(msg) = err.get("Message").and_then(|m| m.as_str()) {
                                errors.push(msg.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    errors
}

/// Truncates a string to a maximum length, appending "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Find a valid UTF-8 boundary at or before max_len
        let boundary = s
            .char_indices()
            .take_while(|(i, _)| *i < max_len)
            .last()
            .map_or(0, |(i, ch)| i + ch.len_utf8());
        format!("{}...", &s[..boundary])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_creates_client() {
        let client = XeroClient::builder()
            .client_id("test-id")
            .tenant_id("test-tenant")
            .build()
            .unwrap();
        assert_eq!(client.tenant_id(), "test-tenant");
    }

    #[test]
    fn builder_with_config() {
        let config = SdkConfig::default().with_timeout_secs(60);
        let client = XeroClient::builder()
            .config(config)
            .client_id("test")
            .tenant_id("tenant")
            .build()
            .unwrap();
        assert_eq!(client.config().timeout, std::time::Duration::from_secs(60));
    }

    #[test]
    fn backoff_delay_exponential() {
        assert_eq!(backoff_delay(0), std::time::Duration::from_secs(1));
        assert_eq!(backoff_delay(1), std::time::Duration::from_secs(2));
        assert_eq!(backoff_delay(2), std::time::Duration::from_secs(4));
        assert_eq!(backoff_delay(3), std::time::Duration::from_secs(8));
        assert_eq!(backoff_delay(4), std::time::Duration::from_secs(16));
        // Capped at 16
        assert_eq!(backoff_delay(5), std::time::Duration::from_secs(16));
    }

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string() {
        assert_eq!(truncate("hello world", 5), "hello...");
    }

    #[test]
    fn truncate_utf8_safety() {
        // "café!" has a multi-byte 'é' (2 bytes: 0xC3 0xA9)
        let s = "café!";
        assert_eq!(s.len(), 6); // 3 ASCII + 2-byte é + 1 ASCII
        // max_len=4: char_indices (0,'c'),(1,'a'),(2,'f'),(3,'é')
        // all have index < 4, so boundary = 3 + 2 = 5 (includes full 'é')
        let result = truncate(s, 4);
        assert_eq!(result, "café...");
        // max_len=3: only (0,'c'),(1,'a'),(2,'f') have index < 3
        // boundary = 2 + 1 = 3
        let result2 = truncate(s, 3);
        assert_eq!(result2, "caf...");
    }

    #[test]
    fn extract_validation_errors_from_xero_response() {
        let body = r#"{
            "Invoices": [{
                "InvoiceID": "00000000-0000-0000-0000-000000000000",
                "ValidationErrors": [
                    {"Message": "Account code '999' is not a valid code"},
                    {"Message": "Contact is required"}
                ]
            }]
        }"#;
        let errors = extract_validation_errors(body);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0], "Account code '999' is not a valid code");
        assert_eq!(errors[1], "Contact is required");
    }

    #[test]
    fn extract_validation_errors_empty_on_invalid_json() {
        assert!(extract_validation_errors("not json").is_empty());
        assert!(extract_validation_errors("{}").is_empty());
    }

    #[test]
    fn write_safety_gate_blocks_by_default() {
        let client = XeroClient::builder()
            .client_id("test")
            .tenant_id("tenant")
            .build()
            .unwrap();
        let result = client.check_writes_allowed();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ChoSdkError::WriteNotAllowed { .. }));
    }

    #[test]
    fn write_safety_gate_allows_when_enabled() {
        let config = SdkConfig::default().with_allow_writes(true);
        let client = XeroClient::builder()
            .config(config)
            .client_id("test")
            .tenant_id("tenant")
            .build()
            .unwrap();
        assert!(client.check_writes_allowed().is_ok());
    }
}
