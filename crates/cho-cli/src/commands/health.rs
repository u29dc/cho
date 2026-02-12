//! Health check command for agent readiness gating.
//!
//! `cho health --json` returns structured readiness status so agents can
//! verify prerequisites before making API calls.

use std::time::Instant;

use serde::Serialize;

use cho_sdk::auth::storage;

use crate::envelope;

/// Health check result for a single component.
#[derive(Serialize)]
struct Check {
    id: &'static str,
    label: &'static str,
    status: &'static str,
    severity: &'static str,
    detail: String,
    fix: &'static str,
}

/// Summary counts.
#[derive(Serialize)]
struct Summary {
    pass: usize,
    warn: usize,
    fail: usize,
}

/// Overall health response.
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    checks: Vec<Check>,
    summary: Summary,
}

/// Runs the health check.
///
/// Returns the process exit code (0 = ready/degraded, 2 = blocked).
pub async fn run(json_mode: bool, start: Instant) -> i32 {
    let mut checks = Vec::new();

    // 1. Config file check
    checks.push(check_config());

    // 2. Auth check
    checks.push(check_auth().await);

    // 3. Tenant check
    checks.push(check_tenant());

    // 4. Keyring check
    checks.push(check_keyring());

    // Compute summary
    let pass = checks.iter().filter(|c| c.status == "pass").count();
    let warn = checks.iter().filter(|c| c.status == "warn").count();
    let fail = checks.iter().filter(|c| c.status == "fail").count();

    let has_blocking_fail = checks
        .iter()
        .any(|c| c.status == "fail" && c.severity == "blocking");

    let status = if has_blocking_fail {
        "blocked"
    } else if warn > 0 || fail > 0 {
        "degraded"
    } else {
        "ready"
    };

    let response = HealthResponse {
        status,
        checks,
        summary: Summary { pass, warn, fail },
    };

    if json_mode {
        let json_value = serde_json::to_value(&response).unwrap_or_default();
        let output = envelope::emit_success("health", json_value, start, None, None, None);
        println!("{output}");
    } else {
        print_health_text(&response);
    }

    if has_blocking_fail { 2 } else { 0 }
}

/// Checks if the config file exists and parses.
fn check_config() -> Check {
    match storage::config_dir() {
        Ok(dir) => {
            let path = dir.join("config.toml");
            if path.exists() {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match content.parse::<toml::Table>() {
                        Ok(_) => Check {
                            id: "config",
                            label: "Configuration file",
                            status: "pass",
                            severity: "blocking",
                            detail: format!("{}", path.display()),
                            fix: "cho init",
                        },
                        Err(e) => Check {
                            id: "config",
                            label: "Configuration file",
                            status: "fail",
                            severity: "blocking",
                            detail: format!("Parse error: {e}"),
                            fix: "cho init",
                        },
                    },
                    Err(e) => Check {
                        id: "config",
                        label: "Configuration file",
                        status: "fail",
                        severity: "blocking",
                        detail: format!("Read error: {e}"),
                        fix: "cho init",
                    },
                }
            } else {
                Check {
                    id: "config",
                    label: "Configuration file",
                    status: "fail",
                    severity: "blocking",
                    detail: format!("Not found at {}", path.display()),
                    fix: "cho init",
                }
            }
        }
        Err(_) => Check {
            id: "config",
            label: "Configuration file",
            status: "fail",
            severity: "blocking",
            detail: "Cannot determine config directory".to_string(),
            fix: "cho init",
        },
    }
}

/// Checks if auth tokens are available and not expired.
async fn check_auth() -> Check {
    let client_id = std::env::var("CHO_CLIENT_ID").unwrap_or_default();
    if client_id.is_empty() {
        // Try loading from config
        let has_client_id = storage::config_dir()
            .ok()
            .and_then(|d| std::fs::read_to_string(d.join("config.toml")).ok())
            .and_then(|c| c.parse::<toml::Table>().ok())
            .and_then(|t| {
                t.get("auth")?
                    .as_table()?
                    .get("client_id")?
                    .as_str()
                    .map(|s| !s.is_empty())
            })
            .unwrap_or(false);

        if !has_client_id {
            return Check {
                id: "auth",
                label: "Authentication",
                status: "fail",
                severity: "blocking",
                detail: "No client_id configured".to_string(),
                fix: "cho init",
            };
        }
    }

    // Try loading tokens
    let auth = cho_sdk::auth::AuthManager::new(std::env::var("CHO_CLIENT_ID").unwrap_or_default());

    match auth.load_stored_tokens().await {
        Ok(true) => {
            if auth.is_authenticated().await {
                Check {
                    id: "auth",
                    label: "Authentication",
                    status: "pass",
                    severity: "blocking",
                    detail: "Token loaded and valid".to_string(),
                    fix: "cho auth login",
                }
            } else {
                Check {
                    id: "auth",
                    label: "Authentication",
                    status: "fail",
                    severity: "blocking",
                    detail: "Token expired".to_string(),
                    fix: "cho auth login",
                }
            }
        }
        Ok(false) => Check {
            id: "auth",
            label: "Authentication",
            status: "fail",
            severity: "blocking",
            detail: "No stored tokens found".to_string(),
            fix: "cho auth login",
        },
        Err(e) => Check {
            id: "auth",
            label: "Authentication",
            status: "fail",
            severity: "blocking",
            detail: format!("Token load failed: {e}"),
            fix: "cho auth login",
        },
    }
}

/// Checks if a tenant ID is configured.
fn check_tenant() -> Check {
    let from_env = std::env::var("CHO_TENANT_ID")
        .ok()
        .filter(|s| !s.is_empty());

    if from_env.is_some() {
        return Check {
            id: "tenant",
            label: "Tenant configured",
            status: "pass",
            severity: "blocking",
            detail: "Set via CHO_TENANT_ID".to_string(),
            fix: "cho config set auth.tenant_id <ID>",
        };
    }

    let from_config = storage::config_dir()
        .ok()
        .and_then(|d| std::fs::read_to_string(d.join("config.toml")).ok())
        .and_then(|c| c.parse::<toml::Table>().ok())
        .and_then(|t| {
            t.get("auth")?
                .as_table()?
                .get("tenant_id")?
                .as_str()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        });

    match from_config {
        Some(tid) => Check {
            id: "tenant",
            label: "Tenant configured",
            status: "pass",
            severity: "blocking",
            detail: format!("{}...", &tid[..tid.len().min(8)]),
            fix: "cho config set auth.tenant_id <ID>",
        },
        None => Check {
            id: "tenant",
            label: "Tenant configured",
            status: "fail",
            severity: "blocking",
            detail: "No tenant_id in config or environment".to_string(),
            fix: "cho config set auth.tenant_id <ID>",
        },
    }
}

/// Checks if the OS keyring is accessible by attempting to load the client_id.
fn check_keyring() -> Check {
    match storage::load_client_id() {
        Ok(Some(_)) => Check {
            id: "keyring",
            label: "Token storage",
            status: "pass",
            severity: "info",
            detail: "OS keyring available".to_string(),
            fix: "",
        },
        Ok(None) => Check {
            id: "keyring",
            label: "Token storage",
            status: "pass",
            severity: "info",
            detail: "OS keyring available (no stored client_id)".to_string(),
            fix: "",
        },
        Err(_) => Check {
            id: "keyring",
            label: "Token storage",
            status: "warn",
            severity: "info",
            detail: "OS keyring unavailable; tokens not persisted".to_string(),
            fix: "",
        },
    }
}

/// Prints health status in human-readable format.
fn print_health_text(response: &HealthResponse) {
    let status_label = match response.status {
        "ready" => "READY",
        "degraded" => "DEGRADED",
        "blocked" => "BLOCKED",
        s => s,
    };

    eprintln!("  Status: {status_label}");
    eprintln!();

    for check in &response.checks {
        let indicator = match check.status {
            "pass" => "+",
            "warn" => "~",
            "fail" => "x",
            _ => "?",
        };
        eprintln!("  [{indicator}] {:<22} {}", check.label, check.detail);
        if check.status == "fail" && !check.fix.is_empty() {
            eprintln!("      Fix: {}", check.fix);
        }
    }

    eprintln!();
    eprintln!(
        "  {} pass, {} warn, {} fail",
        response.summary.pass, response.summary.warn, response.summary.fail
    );
}
