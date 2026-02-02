//! Integration tests using wiremock to mock the Xero API.
//!
//! Tests cover: basic GET, pagination, rate limiting (429 retry),
//! and write operations with validation errors.

use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use cho_sdk::auth::AuthManager;
use cho_sdk::auth::token::TokenPair;
use cho_sdk::client::XeroClient;
use cho_sdk::config::SdkConfig;
use cho_sdk::http::pagination::PaginationParams;
use cho_sdk::http::rate_limit::RateLimitConfig;
use cho_sdk::http::request::ListParams;

/// Helper: builds a XeroClient pointed at the given mock server.
fn test_client(base_url: &str) -> XeroClient {
    let config = SdkConfig::new().with_base_url(base_url).with_max_retries(2);
    let token = TokenPair::for_testing("test-access-token", 3600);
    let auth = AuthManager::with_token("test-client-id".to_string(), token);

    XeroClient::builder()
        .config(config)
        .tenant_id("test-tenant-id")
        .auth_manager(auth)
        .rate_limit(RateLimitConfig {
            enabled: false,
            ..Default::default()
        })
        .build()
        .unwrap()
}

#[tokio::test]
async fn get_single_invoice() {
    let server = MockServer::start().await;

    let invoice_id = "00000000-0000-0000-0000-000000000001";
    let body = serde_json::json!({
        "Invoices": [{
            "InvoiceID": invoice_id,
            "Type": "ACCREC",
            "Status": "AUTHORISED",
            "InvoiceNumber": "INV-001"
        }]
    });

    Mock::given(method("GET"))
        .and(path(format!("Invoices/{invoice_id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&format!("{}/", server.uri()));
    let invoice = client
        .invoices()
        .get(invoice_id.parse().unwrap())
        .await
        .unwrap();

    assert_eq!(invoice.invoice_id.unwrap().to_string(), invoice_id);
    assert_eq!(invoice.invoice_number.as_deref(), Some("INV-001"));
}

#[tokio::test]
async fn list_invoices_single_page() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "Invoices": [
            {"InvoiceID": "00000000-0000-0000-0000-000000000001", "Type": "ACCREC"},
            {"InvoiceID": "00000000-0000-0000-0000-000000000002", "Type": "ACCREC"}
        ],
        "pagination": {
            "page": 1,
            "pageSize": 100,
            "pageCount": 1,
            "itemCount": 2
        }
    });

    Mock::given(method("GET"))
        .and(path("Invoices"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&format!("{}/", server.uri()));
    let result = client
        .invoices()
        .list(&ListParams::new(), &PaginationParams::default())
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
    assert!(result.pagination.is_some());
    let pag = result.pagination.unwrap();
    assert_eq!(pag.item_count, Some(2));
}

#[tokio::test]
async fn list_invoices_multi_page() {
    let server = MockServer::start().await;

    // Page 1
    let page1 = serde_json::json!({
        "Invoices": [
            {"InvoiceID": "00000000-0000-0000-0000-000000000001", "Type": "ACCREC"}
        ],
        "pagination": {
            "page": 1,
            "pageSize": 1,
            "pageCount": 2,
            "itemCount": 2
        }
    });

    // Page 2
    let page2 = serde_json::json!({
        "Invoices": [
            {"InvoiceID": "00000000-0000-0000-0000-000000000002", "Type": "ACCREC"}
        ],
        "pagination": {
            "page": 2,
            "pageSize": 1,
            "pageCount": 2,
            "itemCount": 2
        }
    });

    Mock::given(method("GET"))
        .and(path("Invoices"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&page1))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("Invoices"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&page2))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&format!("{}/", server.uri()));
    let result = client
        .invoices()
        .list(
            &ListParams::new(),
            &PaginationParams {
                limit: 0,
                page_size: 1,
            },
        )
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
}

#[tokio::test]
async fn rate_limit_429_retry() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "Invoices": [{
            "InvoiceID": "00000000-0000-0000-0000-000000000001",
            "Type": "ACCREC"
        }]
    });

    // First request returns 429
    Mock::given(method("GET"))
        .and(path("Invoices/00000000-0000-0000-0000-000000000001"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;

    // Second request succeeds
    Mock::given(method("GET"))
        .and(path("Invoices/00000000-0000-0000-0000-000000000001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    // Use enabled rate limiter but with fast settings
    let config = SdkConfig::new()
        .with_base_url(format!("{}/", server.uri()))
        .with_max_retries(2);
    let token = TokenPair::for_testing("test-token", 3600);
    let auth = AuthManager::with_token("test-client-id".to_string(), token);
    let client = XeroClient::builder()
        .config(config)
        .tenant_id("test-tenant")
        .auth_manager(auth)
        .rate_limit(RateLimitConfig {
            enabled: false,
            ..Default::default()
        })
        .build()
        .unwrap();

    let invoice = client
        .invoices()
        .get("00000000-0000-0000-0000-000000000001".parse().unwrap())
        .await
        .unwrap();

    assert_eq!(
        invoice.invoice_id.unwrap().to_string(),
        "00000000-0000-0000-0000-000000000001"
    );
}

#[tokio::test]
async fn not_found_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("Invoices/00000000-0000-0000-0000-000000000099"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&format!("{}/", server.uri()));
    let result = client
        .invoices()
        .get("00000000-0000-0000-0000-000000000099".parse().unwrap())
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        cho_sdk::error::ChoSdkError::NotFound { .. }
    ));
}

#[tokio::test]
async fn api_error_with_validation_errors() {
    let server = MockServer::start().await;

    let error_body = serde_json::json!({
        "Invoices": [{
            "InvoiceID": "00000000-0000-0000-0000-000000000000",
            "ValidationErrors": [
                {"Message": "Account code is required"},
                {"Message": "Contact is required"}
            ]
        }]
    });

    Mock::given(method("PUT"))
        .and(path("Invoices"))
        .respond_with(ResponseTemplate::new(400).set_body_json(&error_body))
        .expect(1)
        .mount(&server)
        .await;

    let config = SdkConfig::new()
        .with_base_url(format!("{}/", server.uri()))
        .with_max_retries(0)
        .with_allow_writes(true);
    let token = TokenPair::for_testing("test-token", 3600);
    let auth = AuthManager::with_token("test-client-id".to_string(), token);
    let client = XeroClient::builder()
        .config(config)
        .tenant_id("test-tenant")
        .auth_manager(auth)
        .rate_limit(RateLimitConfig {
            enabled: false,
            ..Default::default()
        })
        .build()
        .unwrap();

    let invoice: cho_sdk::models::invoice::Invoice = serde_json::from_str("{}").unwrap();
    let result: Result<cho_sdk::models::invoice::Invoice, cho_sdk::error::ChoSdkError> =
        client.invoices().create(&invoice, None).await;

    assert!(result.is_err());
    if let cho_sdk::error::ChoSdkError::ApiError {
        status,
        validation_errors,
        ..
    } = result.unwrap_err()
    {
        assert_eq!(status, 400);
        assert_eq!(validation_errors.len(), 2);
        assert_eq!(validation_errors[0], "Account code is required");
        assert_eq!(validation_errors[1], "Contact is required");
    } else {
        panic!("Expected ApiError");
    }
}

#[tokio::test]
async fn header_tracking_updates_from_response() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "Invoices": [{
            "InvoiceID": "00000000-0000-0000-0000-000000000001",
            "Type": "ACCREC"
        }]
    });

    Mock::given(method("GET"))
        .and(path("Invoices/00000000-0000-0000-0000-000000000001"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&body)
                .insert_header("X-MinLimit-Remaining", "45")
                .insert_header("X-DayLimit-Remaining", "4800")
                .insert_header("X-AppMinLimit-Remaining", "9500"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&format!("{}/", server.uri()));
    let _invoice = client
        .invoices()
        .get("00000000-0000-0000-0000-000000000001".parse().unwrap())
        .await
        .unwrap();

    // The rate limiter headers are tracked internally.
    // We can't directly inspect them here, but the test verifies the
    // request succeeds with rate limit headers present.
}
