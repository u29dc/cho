use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::{Duration, Utc};
use secrecy::SecretString;
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

use cho_sdk::api::by_name;
use cho_sdk::auth::{AuthManager, token::StoredTokens};
use cho_sdk::client::FreeAgentClient;
use cho_sdk::config::SdkConfig;
use cho_sdk::error::ChoSdkError;
use cho_sdk::models::Pagination;

fn seeded_tokens(access_token: &str, refresh_token: &str) -> StoredTokens {
    StoredTokens {
        access_token: access_token.to_string(),
        refresh_token: Some(refresh_token.to_string()),
        expires_at: Utc::now() + Duration::minutes(30),
        refresh_expires_at: Some(Utc::now() + Duration::hours(1)),
    }
}

async fn build_client(
    server: &MockServer,
    access_token: &str,
    refresh_token: &str,
    max_retries: u32,
    allow_writes: bool,
) -> FreeAgentClient {
    let config = SdkConfig::default()
        .with_base_url(format!("{}/v2/", server.uri()))
        .with_token_url(format!("{}/oauth/token", server.uri()))
        .with_max_retries(max_retries)
        .with_allow_writes(allow_writes);

    let auth = AuthManager::new(
        "client-id".to_string(),
        SecretString::new("client-secret".to_string().into()),
        config.clone(),
    )
    .expect("auth manager must build")
    .with_token_persistence(false);

    auth.set_tokens_in_memory(seeded_tokens(access_token, refresh_token))
        .await;

    FreeAgentClient::builder()
        .config(config)
        .auth_manager(auth)
        .build()
        .expect("client must build")
}

#[tokio::test]
async fn list_paginated_fetches_all_pages_when_all_is_true() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/contacts"))
        .and(query_param("page", "1"))
        .and(query_param("per_page", "2"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("X-Total-Count", "3")
                .insert_header(
                    "Link",
                    format!("<{}/v2/contacts?page=2>; rel=\"next\"", server.uri()),
                )
                .set_body_json(json!({
                    "contacts": [
                        {"url": "https://api.freeagent.com/v2/contacts/1"},
                        {"url": "https://api.freeagent.com/v2/contacts/2"}
                    ]
                })),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v2/contacts"))
        .and(query_param("page", "2"))
        .and(query_param("per_page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "contacts": [{"url": "https://api.freeagent.com/v2/contacts/3"}]
        })))
        .mount(&server)
        .await;

    let client = build_client(&server, "seed-access", "seed-refresh", 0, false).await;

    let result = client
        .list_paginated(
            "contacts",
            "contacts",
            &[],
            Pagination {
                per_page: 2,
                limit: 100,
                all: true,
            },
        )
        .await
        .expect("list request should succeed");

    assert_eq!(result.items.len(), 3);
    assert_eq!(result.total, Some(3));
    assert!(!result.has_more);
    assert_eq!(result.page, 2);
}

#[tokio::test]
async fn list_paginated_respects_limit_and_sets_has_more() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/invoices"))
        .and(query_param("page", "1"))
        .and(query_param("per_page", "2"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("X-Total-Count", "5")
                .insert_header(
                    "Link",
                    format!("<{}/v2/invoices?page=2>; rel=\"next\"", server.uri()),
                )
                .set_body_json(json!({
                    "invoices": [
                        {"url": "https://api.freeagent.com/v2/invoices/1"},
                        {"url": "https://api.freeagent.com/v2/invoices/2"}
                    ]
                })),
        )
        .mount(&server)
        .await;

    let client = build_client(&server, "seed-access", "seed-refresh", 0, false).await;

    let result = client
        .list_paginated(
            "invoices",
            "invoices",
            &[],
            Pagination {
                per_page: 2,
                limit: 2,
                all: false,
            },
        )
        .await
        .expect("list request should succeed");

    assert_eq!(result.items.len(), 2);
    assert_eq!(result.total, Some(5));
    assert!(result.has_more);
}

#[tokio::test]
async fn get_json_refreshes_on_unauthorized_and_retries_with_new_token() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "new-access",
            "token_type": "bearer",
            "expires_in": 3600,
            "refresh_token": "new-refresh"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v2/company"))
        .and(header("authorization", "Bearer old-access"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v2/company"))
        .and(header("authorization", "Bearer new-access"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "company": {"name": "Acme Ltd"}
        })))
        .mount(&server)
        .await;

    let client = build_client(&server, "old-access", "old-refresh", 0, false).await;

    let body = client
        .get_json("company", &[])
        .await
        .expect("request should refresh and succeed");

    assert_eq!(body["company"]["name"], "Acme Ltd");
}

#[derive(Clone)]
struct RateLimitThenSuccess {
    calls: Arc<AtomicUsize>,
}

impl Respond for RateLimitThenSuccess {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        if call == 0 {
            ResponseTemplate::new(429).insert_header("Retry-After", "0")
        } else {
            ResponseTemplate::new(200).set_body_json(json!({
                "company": {"name": "After Retry Ltd"}
            }))
        }
    }
}

#[tokio::test]
async fn get_json_retries_after_rate_limit_and_succeeds() {
    let server = MockServer::start().await;

    let calls = Arc::new(AtomicUsize::new(0));
    Mock::given(method("GET"))
        .and(path("/v2/company"))
        .respond_with(RateLimitThenSuccess {
            calls: Arc::clone(&calls),
        })
        .mount(&server)
        .await;

    let client = build_client(&server, "seed-access", "seed-refresh", 1, false).await;

    let body = client
        .get_json("company", &[])
        .await
        .expect("request should succeed after retry");

    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(body["company"]["name"], "After Retry Ltd");
}

#[tokio::test]
async fn list_paginated_errors_when_collection_key_is_missing() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/contacts"))
        .and(query_param("page", "1"))
        .and(query_param("per_page", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "unexpected_key": []
        })))
        .mount(&server)
        .await;

    let client = build_client(&server, "seed-access", "seed-refresh", 0, false).await;

    let err = client
        .list_paginated("contacts", "contacts", &[], Pagination::default())
        .await
        .expect_err("missing key should fail");

    match err {
        ChoSdkError::Parse { message } => {
            assert!(message.contains("collection key 'contacts'"));
        }
        other => panic!("expected parse error, got {other}"),
    }
}

#[tokio::test]
async fn post_json_rejects_mutating_requests_when_writes_disabled() {
    let server = MockServer::start().await;

    let client = build_client(&server, "seed-access", "seed-refresh", 0, false).await;

    let err = client
        .post_json("contacts", &json!({"contact": {"first_name": "Ada"}}), true)
        .await
        .expect_err("write should be blocked");

    match err {
        ChoSdkError::WriteNotAllowed { message } => {
            assert!(message.contains("allow_writes"));
        }
        other => panic!("expected write-not-allowed error, got {other}"),
    }
}

#[tokio::test]
async fn resource_get_rejects_absolute_id_with_untrusted_origin() {
    let trusted = MockServer::start().await;
    let untrusted = MockServer::start().await;

    let client = build_client(&trusted, "seed-access", "seed-refresh", 0, false).await;
    let spec = by_name("contacts").expect("contacts resource spec must exist");

    let err = client
        .resource(spec)
        .get(&format!("{}/v2/contacts/123", untrusted.uri()))
        .await
        .expect_err("untrusted absolute URL must be rejected");

    match err {
        ChoSdkError::Config { message } => {
            assert!(message.contains("UNTRUSTED_RESOURCE_URL"));
        }
        other => panic!("expected config error, got {other}"),
    }
}
