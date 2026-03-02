//! Data access layer for `cho-tui`.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::Duration;

use cho_sdk::api::specs::{ResourceSpec, by_name};
use cho_sdk::auth::AuthManager;
use cho_sdk::client::{FreeAgentClient, RequestPolicy};
use cho_sdk::error::ChoSdkError;
use cho_sdk::models::{ListResult, Pagination};
use chrono::{DateTime, Datelike, NaiveDate};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::AppConfig;
use crate::routes::{RouteDefinition, RouteKind};

/// Data payload loaded for a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutePayload {
    /// List payload and pagination metadata.
    List {
        /// Items.
        items: Vec<Value>,
        /// Total from API when present.
        total: Option<usize>,
        /// Whether more pages are available.
        has_more: bool,
    },
    /// Generic JSON object payload.
    Object(Value),
    /// Informational text payload.
    Message(String),
}

/// Dynamic query context that influences route fetching.
#[derive(Debug, Clone)]
pub struct FetchContext {
    /// Selected bank account URL for bank-ledger views.
    pub bank_account_filter: Option<String>,
    /// Selected self-assessment user id/url.
    pub self_assessment_user: Option<String>,
    /// Selected resource target ids for get-only resources.
    pub resource_targets: HashMap<String, String>,
    /// Payroll year context.
    pub payroll_year: i32,
    /// Payroll period context.
    pub payroll_period: i32,
}

impl Default for FetchContext {
    fn default() -> Self {
        Self {
            bank_account_filter: None,
            self_assessment_user: None,
            resource_targets: HashMap::new(),
            payroll_year: chrono::Utc::now().year(),
            payroll_period: 1,
        }
    }
}

/// Route load options for interactive fetches.
#[derive(Debug, Clone, Copy)]
pub struct RouteLoadOptions {
    /// Max number of rows retained client-side.
    pub limit: usize,
    /// Requested page size for list endpoints.
    pub per_page: usize,
    /// Request timeout/retry policy.
    pub request_policy: RequestPolicy,
}

#[derive(Debug, Clone)]
struct BankAccountScope {
    url: String,
    name: String,
}

impl RouteLoadOptions {
    /// Full load defaults for explicit actions.
    pub fn full(limit: usize) -> Self {
        Self {
            limit,
            per_page: limit.clamp(1, 100),
            request_policy: RequestPolicy::default(),
        }
    }

    /// Fast-preview defaults for nav hover interactions.
    pub fn preview(limit: usize, timeout: Duration, retries: u32) -> Self {
        Self {
            limit,
            per_page: limit.clamp(1, 100),
            request_policy: RequestPolicy {
                timeout_override: Some(timeout),
                max_retries_override: Some(retries),
            },
        }
    }
}

/// Runtime API facade.
pub struct ApiEngine {
    runtime: tokio::runtime::Runtime,
    client: Option<FreeAgentClient>,
    app_config: AppConfig,
    startup_warnings: Vec<String>,
}

impl ApiEngine {
    /// Builds an API engine and attempts client initialization.
    pub fn new() -> Result<Self, String> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime for cho-tui: {e}"))?;

        let mut startup_warnings = Vec::new();
        let app_config = match AppConfig::load() {
            Ok(config) => config,
            Err(err) => {
                startup_warnings.push(format!(
                    "config.load failed; running with defaults ({})",
                    err
                ));
                AppConfig::default()
            }
        };

        let mut client = None;
        let client_id = app_config.resolve_client_id();
        let client_secret = app_config.resolve_client_secret();
        if client_id.is_none() || client_secret.is_none() {
            startup_warnings.push(
                "Missing OAuth credentials (set CHO_CLIENT_ID / CHO_CLIENT_SECRET or config auth.*)"
                    .to_string(),
            );
        } else if let (Some(client_id), Some(client_secret)) = (client_id, client_secret) {
            let sdk_config = app_config.sdk_config();
            match AuthManager::new(
                client_id,
                SecretString::new(client_secret.into()),
                sdk_config.clone(),
            ) {
                Ok(auth) => {
                    if let Err(err) = runtime.block_on(auth.load_stored_tokens()) {
                        startup_warnings
                            .push(format!("token.load failed (run `cho auth login`): {err}"));
                    }

                    match FreeAgentClient::builder()
                        .config(sdk_config)
                        .auth_manager(auth)
                        .build()
                    {
                        Ok(built) => client = Some(built),
                        Err(err) => {
                            startup_warnings.push(format!("client.build failed: {err}"));
                        }
                    }
                }
                Err(err) => startup_warnings.push(format!("auth.init failed: {err}")),
            }
        }

        Ok(Self {
            runtime,
            client,
            app_config,
            startup_warnings,
        })
    }

    /// Returns true when writes are enabled in config.
    pub fn writes_allowed(&self) -> bool {
        self.app_config.safety.allow_writes
    }

    /// Startup warnings captured during initialization.
    pub fn startup_warnings(&self) -> &[String] {
        &self.startup_warnings
    }

    /// Returns true if client exists and auth token is currently valid.
    pub fn is_authenticated(&self) -> bool {
        let Some(client) = &self.client else {
            return false;
        };
        self.runtime.block_on(client.auth().is_authenticated())
    }

    /// Fetches data for the provided route.
    pub fn fetch_route(
        &self,
        route: &RouteDefinition,
        context: &FetchContext,
        limit: usize,
    ) -> Result<RoutePayload, String> {
        self.fetch_route_with_options(route, context, RouteLoadOptions::full(limit))
    }

    /// Fetches data for the provided route with interactive options.
    pub fn fetch_route_with_options(
        &self,
        route: &RouteDefinition,
        context: &FetchContext,
        options: RouteLoadOptions,
    ) -> Result<RoutePayload, String> {
        match route.kind {
            RouteKind::Resource(spec) => self.fetch_resource(spec, route, context, options),
            RouteKind::CompanyGet => {
                self.fetch_object("company", "company.get", options.request_policy)
            }
            RouteKind::CompanyTaxTimeline => self.fetch_object(
                "company/tax_timeline",
                "company.tax-timeline",
                options.request_policy,
            ),
            RouteKind::CompanyBusinessCategories => self.fetch_object(
                "company/business_categories",
                "company.business-categories",
                options.request_policy,
            ),
            RouteKind::ReportProfitAndLoss => self.fetch_object(
                "accounting/profit_and_loss/summary",
                "reports.profit-and-loss",
                options.request_policy,
            ),
            RouteKind::ReportBalanceSheet => self.fetch_object(
                "accounting/balance_sheet",
                "reports.balance-sheet",
                options.request_policy,
            ),
            RouteKind::ReportTrialBalance => self.fetch_object(
                "accounting/trial_balance/summary",
                "reports.trial-balance",
                options.request_policy,
            ),
            RouteKind::ReportCashflow => self.fetch_object_with_query(
                "cashflow",
                &[("months", "12")],
                "reports.cashflow",
                options.request_policy,
            ),
            RouteKind::ExpenseMileageSettings => self.fetch_object(
                "expenses/mileage_settings",
                "expenses.mileage-settings",
                options.request_policy,
            ),
            RouteKind::SelfAssessmentReturns => {
                self.fetch_self_assessment_returns(context, options)
            }
            RouteKind::PayrollPeriods => {
                self.fetch_payroll_periods(context.payroll_year, options.request_policy)
            }
            RouteKind::PayrollPeriodDetail => self.fetch_payroll_period_detail(
                context.payroll_year,
                context.payroll_period,
                options.request_policy,
            ),
            RouteKind::PayrollProfiles => {
                self.fetch_payroll_profiles(context.payroll_year, options.request_policy)
            }
            RouteKind::AuthStatus => self.fetch_auth_status(),
            RouteKind::Health => Ok(self.fetch_health_snapshot()),
            RouteKind::Config => Ok(RoutePayload::Object(self.app_config.as_redacted_json())),
        }
    }

    /// Fetches one resource item by id/url.
    pub fn fetch_resource_item(
        &self,
        spec: cho_sdk::api::specs::ResourceSpec,
        id: &str,
    ) -> Result<RoutePayload, String> {
        let client = self.client()?;
        let value = self
            .runtime
            .block_on(client.resource(spec).get(id))
            .map_err(|e| format!("{}.get failed: {e}", spec.name))?;
        Ok(RoutePayload::Object(value))
    }

    /// Fetches one self-assessment return by user and period end date.
    pub fn fetch_self_assessment_item(
        &self,
        user: &str,
        period_ends_on: &str,
    ) -> Result<RoutePayload, String> {
        let client = self.client()?;
        let value = self
            .runtime
            .block_on(client.get_json(
                &format!(
                    "users/{}/self_assessment_returns/{}",
                    user_id_segment(user),
                    encode_path_segment(period_ends_on)
                ),
                &[],
            ))
            .map_err(|e| format!("self-assessment-returns.get failed: {e}"))?;
        let payload = value
            .get("self_assessment_return")
            .cloned()
            .unwrap_or(value);
        Ok(RoutePayload::Object(payload))
    }

    fn fetch_resource(
        &self,
        spec: cho_sdk::api::specs::ResourceSpec,
        route: &RouteDefinition,
        context: &FetchContext,
        options: RouteLoadOptions,
    ) -> Result<RoutePayload, String> {
        let client = self.client()?;

        if spec.name == "categories" {
            let value = self
                .runtime
                .block_on(client.get_json_with_policy("categories", &[], options.request_policy))
                .map_err(|e| format!("categories.list failed: {e}"))?;
            let items = flatten_category_groups(&value);
            let capped = cap_items(items, options.limit);
            return Ok(RoutePayload::List {
                items: capped.items,
                total: Some(capped.total),
                has_more: capped.has_more,
            });
        }

        if spec.capabilities.list {
            if spec.name == "bank-transactions" || spec.name == "bank-transaction-explanations" {
                return self.fetch_bank_resource(spec, context, options);
            }

            let query = Vec::<(String, String)>::new();
            let pagination = Pagination {
                per_page: options.per_page.clamp(1, 100) as u32,
                limit: options.limit,
                all: false,
            };

            let result = self
                .runtime
                .block_on(client.resource(spec).list_with_policy(
                    &query,
                    pagination,
                    options.request_policy,
                ))
                .map_err(|e| format!("{}.list failed: {e}", spec.name))?;

            let mut items = result.items;
            sort_items_by_latest_date(&mut items);

            return Ok(RoutePayload::List {
                items,
                total: result.total,
                has_more: result.has_more,
            });
        }

        if spec.capabilities.get {
            let Some(id) = context
                .resource_targets
                .get(&route.id)
                .filter(|value| !value.trim().is_empty())
            else {
                return Ok(RoutePayload::Message(format!(
                    "{} requires an item id/url (Cmd/Ctrl+P -> Set target id)",
                    route.label
                )));
            };

            let value = self
                .runtime
                .block_on(
                    client
                        .resource(spec)
                        .get_with_policy(id, options.request_policy),
                )
                .map_err(|e| format!("{}.get failed: {e}", spec.name))?;
            return Ok(RoutePayload::Object(value));
        }

        Ok(RoutePayload::Message(
            "This route has no read-only surface in the current API model.".to_string(),
        ))
    }

    fn fetch_bank_resource(
        &self,
        spec: ResourceSpec,
        context: &FetchContext,
        options: RouteLoadOptions,
    ) -> Result<RoutePayload, String> {
        let client = self.client()?;
        let account_scope = self.resolve_bank_account_scope(context, options.request_policy)?;
        if account_scope.is_empty() {
            return Ok(RoutePayload::Message(
                "No bank accounts found. Open Bank Accounts and refresh.".to_string(),
            ));
        }

        let fetch_all = options.limit >= 100;
        let pagination = if fetch_all {
            Pagination::all()
        } else {
            Pagination {
                per_page: options.per_page.clamp(1, 100) as u32,
                limit: options.limit,
                all: false,
            }
        };

        let mut items = Vec::<Value>::new();
        for account in account_scope {
            let query = vec![("bank_account".to_string(), account.url.clone())];
            let result = self
                .runtime
                .block_on(client.resource(spec).list_with_policy(
                    &query,
                    pagination,
                    options.request_policy,
                ))
                .map_err(|e| format!("{}.list failed: {e}", spec.name))?;

            for mut item in result.items {
                annotate_bank_account(&mut item, &account);
                if spec.name == "bank-transactions" {
                    annotate_review_marker(&mut item);
                    annotate_transaction_descriptions(&mut item);
                }
                items.push(item);
            }
        }

        sort_items_by_latest_date(&mut items);
        let total = items.len();
        if !fetch_all && options.limit > 0 && total > options.limit {
            items.truncate(options.limit);
            return Ok(RoutePayload::List {
                items,
                total: Some(total),
                has_more: true,
            });
        }

        Ok(RoutePayload::List {
            items,
            total: Some(total),
            has_more: false,
        })
    }

    fn resolve_bank_account_scope(
        &self,
        context: &FetchContext,
        policy: RequestPolicy,
    ) -> Result<Vec<BankAccountScope>, String> {
        if let Some(bank_account) = context
            .bank_account_filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            return Ok(vec![BankAccountScope {
                url: bank_account.clone(),
                name: "Filtered account".to_string(),
            }]);
        }

        let spec = by_name("bank-accounts")
            .ok_or_else(|| "Missing bank-accounts resource spec".to_string())?;
        let client = self.client()?;
        let result = self
            .runtime
            .block_on(
                client
                    .resource(spec)
                    .list_with_policy(&[], Pagination::all(), policy),
            )
            .map_err(|e| format!("bank-accounts.list failed: {e}"))?;

        let mut accounts = Vec::new();
        for item in result.items {
            let Some(url) = infer_item_identifier(&item) else {
                continue;
            };
            let name = bank_account_display_name(&item);
            accounts.push(BankAccountScope { url, name });
        }

        Ok(accounts)
    }

    fn fetch_object(
        &self,
        path: &str,
        tool: &str,
        policy: RequestPolicy,
    ) -> Result<RoutePayload, String> {
        let client = self.client()?;
        let value = self
            .runtime
            .block_on(client.get_json_with_policy(path, &[], policy))
            .map_err(|e| format!("{tool} failed: {e}"))?;
        Ok(RoutePayload::Object(value))
    }

    fn fetch_object_with_query(
        &self,
        path: &str,
        query: &[(&str, &str)],
        tool: &str,
        policy: RequestPolicy,
    ) -> Result<RoutePayload, String> {
        let client = self.client()?;
        let query = query
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<_>>();
        let value = self
            .runtime
            .block_on(client.get_json_with_policy(path, &query, policy))
            .map_err(|e| format!("{tool} failed: {e}"))?;
        Ok(RoutePayload::Object(value))
    }

    fn fetch_self_assessment_returns(
        &self,
        context: &FetchContext,
        options: RouteLoadOptions,
    ) -> Result<RoutePayload, String> {
        let client = self.client()?;
        let Some(user) = context
            .self_assessment_user
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        else {
            return Ok(RoutePayload::Message(
                "Set a self-assessment user first (Cmd/Ctrl+P -> Set self-assessment user)"
                    .to_string(),
            ));
        };

        let path = format!("users/{}/self_assessment_returns", user_id_segment(user));
        let pagination = Pagination {
            per_page: options.per_page.clamp(1, 100) as u32,
            limit: options.limit,
            all: false,
        };
        let result = self
            .runtime
            .block_on(client.list_paginated_with_policy(
                &path,
                "self_assessment_returns",
                &[],
                pagination,
                options.request_policy,
            ))
            .map_err(|e| format!("self-assessment-returns.list failed: {e}"))?;

        let mut items = result.items;
        sort_items_by_latest_date(&mut items);

        Ok(RoutePayload::List {
            items,
            total: result.total,
            has_more: result.has_more,
        })
    }

    fn fetch_payroll_periods(
        &self,
        year: i32,
        policy: RequestPolicy,
    ) -> Result<RoutePayload, String> {
        let client = self.client()?;
        let value = self
            .runtime
            .block_on(client.get_json_with_policy(&format!("payroll/{year}"), &[], policy))
            .map_err(|e| format!("payroll.periods failed: {e}"))?;
        Ok(RoutePayload::Object(value))
    }

    fn fetch_payroll_period_detail(
        &self,
        year: i32,
        period: i32,
        policy: RequestPolicy,
    ) -> Result<RoutePayload, String> {
        let client = self.client()?;
        let value = self
            .runtime
            .block_on(client.get_json_with_policy(&format!("payroll/{year}/{period}"), &[], policy))
            .map_err(|e| format!("payroll.period failed: {e}"))?;
        Ok(RoutePayload::Object(value))
    }

    fn fetch_payroll_profiles(
        &self,
        year: i32,
        policy: RequestPolicy,
    ) -> Result<RoutePayload, String> {
        let client = self.client()?;
        let value = self
            .runtime
            .block_on(client.get_json_with_policy(&format!("payroll_profiles/{year}"), &[], policy))
            .map_err(|e| format!("payroll-profiles.list failed: {e}"))?;
        Ok(RoutePayload::Object(value))
    }

    fn fetch_auth_status(&self) -> Result<RoutePayload, String> {
        let Some(client) = &self.client else {
            return Ok(RoutePayload::Object(serde_json::json!({
                "authenticated": false,
                "reason": "API client unavailable (credentials missing or initialization failed)"
            })));
        };

        let loaded = self
            .runtime
            .block_on(client.auth().load_stored_tokens())
            .unwrap_or(false);
        let status = self.runtime.block_on(client.auth().status());
        Ok(RoutePayload::Object(serde_json::json!({
            "loaded_token": loaded,
            "status": status
        })))
    }

    fn fetch_health_snapshot(&self) -> RoutePayload {
        let mut checks = vec![
            check_home(),
            check_config(),
            check_credentials(&self.app_config),
            check_client(self.client.is_some()),
            check_writes(self.writes_allowed()),
        ];

        if !self.startup_warnings.is_empty() {
            checks.push(serde_json::json!({
                "id": "startup",
                "label": "Startup warnings",
                "status": "warn",
                "severity": "info",
                "detail": self.startup_warnings.join(" | "),
                "fix": "Resolve warnings and refresh"
            }));
        }

        let pass = checks
            .iter()
            .filter(|item| item.get("status").and_then(Value::as_str) == Some("pass"))
            .count();
        let warn = checks
            .iter()
            .filter(|item| item.get("status").and_then(Value::as_str) == Some("warn"))
            .count();
        let fail = checks
            .iter()
            .filter(|item| item.get("status").and_then(Value::as_str) == Some("fail"))
            .count();

        let blocked = checks.iter().any(|item| {
            item.get("status").and_then(Value::as_str) == Some("fail")
                && item.get("severity").and_then(Value::as_str) == Some("blocking")
        });

        let status = if blocked {
            "blocked"
        } else if warn > 0 || fail > 0 {
            "degraded"
        } else {
            "ready"
        };

        RoutePayload::Object(serde_json::json!({
            "status": status,
            "checks": checks,
            "summary": {
                "pass": pass,
                "warn": warn,
                "fail": fail
            }
        }))
    }

    fn client(&self) -> Result<&FreeAgentClient, String> {
        self.client.as_ref().ok_or_else(|| {
            "API client unavailable. Check credentials/config and open System > Health".to_string()
        })
    }
}

fn cap_items(items: Vec<Value>, limit: usize) -> CappedItems {
    let total = items.len();
    if limit == 0 || total <= limit {
        return CappedItems {
            items,
            total,
            has_more: false,
        };
    }

    let mut items = items;
    items.truncate(limit);
    CappedItems {
        items,
        total,
        has_more: true,
    }
}

fn annotate_bank_account(item: &mut Value, account: &BankAccountScope) {
    let Value::Object(map) = item else {
        return;
    };

    map.entry("_bank_account_url".to_string())
        .or_insert_with(|| Value::String(account.url.clone()));
    map.entry("_bank_account_name".to_string())
        .or_insert_with(|| Value::String(account.name.clone()));
}

fn annotate_review_marker(item: &mut Value) {
    let requires_review = transaction_requires_review(item);
    let Value::Object(map) = item else {
        return;
    };

    map.insert(
        "_review_marker".to_string(),
        Value::String(if requires_review { " ●" } else { "" }.to_string()),
    );
    map.insert("_requires_review".to_string(), Value::Bool(requires_review));
}

fn annotate_transaction_descriptions(item: &mut Value) {
    let raw = transaction_raw_description(item);
    let submitted = first_explanation_description(item);

    let Value::Object(map) = item else {
        return;
    };

    map.entry("_description_raw".to_string())
        .or_insert_with(|| Value::String(raw.unwrap_or_default()));
    map.entry("_description_submitted".to_string())
        .or_insert_with(|| Value::String(submitted.unwrap_or_default()));
}

fn transaction_requires_review(item: &Value) -> bool {
    if item
        .get("marked_for_review")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return true;
    }

    item.get("bank_transaction_explanations")
        .and_then(Value::as_array)
        .is_some_and(|explanations| {
            explanations.iter().any(|explanation| {
                explanation
                    .get("marked_for_review")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
        })
}

fn transaction_raw_description(item: &Value) -> Option<String> {
    for key in [
        "raw_description",
        "bank_description",
        "description",
        "original_description",
    ] {
        if let Some(value) = item.get(key).and_then(Value::as_str)
            && !value.trim().is_empty()
        {
            return Some(value.to_string());
        }
    }
    None
}

fn first_explanation_description(item: &Value) -> Option<String> {
    if let Some(value) = item
        .get("bank_transaction_explanation")
        .and_then(|value| value.get("description"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        return Some(value.to_string());
    }

    item.get("bank_transaction_explanations")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find_map(|entry| {
                entry
                    .get("description")
                    .and_then(Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .map(str::to_string)
            })
        })
}

fn infer_item_identifier(value: &Value) -> Option<String> {
    if let Some(url) = value.get("url").and_then(Value::as_str) {
        return Some(url.to_string());
    }

    if let Some(id) = value.get("id").and_then(Value::as_str) {
        return Some(id.to_string());
    }

    if let Some(id) = value.get("id").and_then(Value::as_i64) {
        return Some(id.to_string());
    }

    None
}

fn bank_account_display_name(item: &Value) -> String {
    if let Some(name) = item.get("name").and_then(Value::as_str)
        && !name.trim().is_empty()
    {
        return name.to_string();
    }

    let bank_name = item
        .get("bank_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let account_number = item
        .get("account_number")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if bank_name.is_empty() && account_number.is_empty() {
        "Bank Account".to_string()
    } else if account_number.is_empty() {
        bank_name.to_string()
    } else if bank_name.is_empty() {
        account_number.to_string()
    } else {
        format!("{bank_name} ({account_number})")
    }
}

fn sort_items_by_latest_date(items: &mut [Value]) {
    let Some(date_key) = infer_date_key(items) else {
        return;
    };

    items.sort_by(|left, right| {
        compare_date_values(
            left.get(date_key).and_then(parse_date_value),
            right.get(date_key).and_then(parse_date_value),
        )
    });
}

fn infer_date_key(items: &[Value]) -> Option<&'static str> {
    const DATE_KEYS: &[&str] = &[
        "dated_on",
        "date",
        "created_at",
        "updated_at",
        "period_ends_on",
        "period_end",
        "starts_on",
        "ends_on",
        "due_on",
        "paid_on",
        "submitted_on",
        "filed_on",
        "payment_date",
        "statement_date",
    ];

    for key in DATE_KEYS {
        let count = items
            .iter()
            .filter_map(|item| item.get(*key).and_then(parse_date_value))
            .take(2)
            .count();
        if count >= 2 {
            return Some(*key);
        }
    }
    None
}

fn compare_date_values(left: Option<i64>, right: Option<i64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => right.cmp(&left),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn parse_date_value(value: &Value) -> Option<i64> {
    match value {
        Value::String(text) => parse_date_text(text),
        Value::Number(number) => number.as_i64(),
        _ => None,
    }
}

fn parse_date_text(text: &str) -> Option<i64> {
    if let Ok(datetime) = DateTime::parse_from_rfc3339(text) {
        return Some(datetime.timestamp());
    }

    if let Ok(date) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        return date
            .and_hms_opt(0, 0, 0)
            .map(|datetime| datetime.and_utc().timestamp());
    }

    None
}

struct CappedItems {
    items: Vec<Value>,
    total: usize,
    has_more: bool,
}

fn flatten_category_groups(value: &serde_json::Value) -> Vec<serde_json::Value> {
    let Some(object) = value.as_object() else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for (group_name, group_value) in object {
        if let Some(items) = group_value.as_array() {
            for item in items {
                let mut item_value = item.clone();
                if let serde_json::Value::Object(map) = &mut item_value
                    && !map.contains_key("category_group")
                {
                    map.insert(
                        "category_group".to_string(),
                        serde_json::Value::String(group_name.clone()),
                    );
                }
                out.push(item_value);
            }
        } else if group_value.is_object() {
            let mut item_value = group_value.clone();
            if let serde_json::Value::Object(map) = &mut item_value
                && !map.contains_key("category_group")
            {
                map.insert(
                    "category_group".to_string(),
                    serde_json::Value::String(group_name.clone()),
                );
            }
            out.push(item_value);
        }
    }
    out
}

fn user_id_segment(user: &str) -> String {
    let trimmed = user.trim().trim_end_matches('/');
    if (trimmed.starts_with("https://") || trimmed.starts_with("http://"))
        && let Some(id) = trimmed.rsplit('/').next()
    {
        return encode_path_segment(id);
    }
    encode_path_segment(trimmed)
}

fn encode_path_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn check_home() -> serde_json::Value {
    match cho_sdk::home::ensure_cho_home() {
        Ok(path) => serde_json::json!({
            "id": "home",
            "label": "Cho home directory",
            "status": "pass",
            "severity": "blocking",
            "detail": format!("{}", path.display()),
            "fix": "Ensure path remains writable"
        }),
        Err(err) => serde_json::json!({
            "id": "home",
            "label": "Cho home directory",
            "status": "fail",
            "severity": "blocking",
            "detail": err.to_string(),
            "fix": "Set CHO_HOME or ensure HOME/TOOLS_HOME are valid"
        }),
    }
}

fn check_config() -> serde_json::Value {
    match AppConfig::load() {
        Ok(_) => serde_json::json!({
            "id": "config",
            "label": "Config",
            "status": "pass",
            "severity": "info",
            "detail": "Configuration loaded",
            "fix": "Use cho config set for updates"
        }),
        Err(err) => serde_json::json!({
            "id": "config",
            "label": "Config",
            "status": "fail",
            "severity": "blocking",
            "detail": err.to_string(),
            "fix": "Fix config.toml parse/read errors"
        }),
    }
}

fn check_credentials(config: &AppConfig) -> serde_json::Value {
    if config.resolve_client_id().is_some() && config.resolve_client_secret().is_some() {
        serde_json::json!({
            "id": "credentials",
            "label": "OAuth credentials",
            "status": "pass",
            "severity": "blocking",
            "detail": "client_id and client_secret available",
            "fix": "Keep auth values configured"
        })
    } else {
        serde_json::json!({
            "id": "credentials",
            "label": "OAuth credentials",
            "status": "fail",
            "severity": "blocking",
            "detail": "Missing client_id or client_secret",
            "fix": "Set CHO_CLIENT_ID/CHO_CLIENT_SECRET or config auth.*"
        })
    }
}

fn check_client(client_ready: bool) -> serde_json::Value {
    if client_ready {
        serde_json::json!({
            "id": "client",
            "label": "API client",
            "status": "pass",
            "severity": "blocking",
            "detail": "Client initialized",
            "fix": "No action required"
        })
    } else {
        serde_json::json!({
            "id": "client",
            "label": "API client",
            "status": "fail",
            "severity": "blocking",
            "detail": "Client initialization failed",
            "fix": "Inspect credentials and System > Health details"
        })
    }
}

fn check_writes(enabled: bool) -> serde_json::Value {
    if enabled {
        serde_json::json!({
            "id": "writes",
            "label": "Write gate",
            "status": "warn",
            "severity": "info",
            "detail": "allow_writes=true",
            "fix": "Disable writes when using read-only mode"
        })
    } else {
        serde_json::json!({
            "id": "writes",
            "label": "Write gate",
            "status": "pass",
            "severity": "info",
            "detail": "allow_writes=false (read-only mode)",
            "fix": "Enable only when mutation workflow is required"
        })
    }
}

#[allow(dead_code)]
fn _error_code(err: &ChoSdkError) -> &'static str {
    match err {
        ChoSdkError::AuthRequired { .. } => "AUTH_REQUIRED",
        ChoSdkError::TokenExpired { .. } => "TOKEN_EXPIRED",
        ChoSdkError::RateLimited { .. } => "RATE_LIMITED",
        ChoSdkError::NotFound { .. } => "NOT_FOUND",
        ChoSdkError::ApiError { .. } => "API_ERROR",
        ChoSdkError::Network(_) => "NETWORK_ERROR",
        ChoSdkError::Parse { .. } => "PARSE_ERROR",
        ChoSdkError::Config { .. } => "CONFIG_ERROR",
        ChoSdkError::WriteNotAllowed { .. } => "WRITE_NOT_ALLOWED",
    }
}

#[allow(dead_code)]
fn _list_to_payload(result: ListResult) -> RoutePayload {
    RoutePayload::List {
        items: result.items,
        total: result.total,
        has_more: result.has_more,
    }
}
