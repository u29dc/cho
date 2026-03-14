//! Health/readiness command.

use std::time::Instant;

use serde::Serialize;

use cho_sdk::auth::AuthManager;
use cho_sdk::client::FreeAgentClient;
use secrecy::SecretString;

use crate::audit::AuditLogger;
use crate::envelope;
use crate::output::{OutputFormat, OutputMode, format_value};

use super::utils::AppConfig;

/// Health check item.
#[derive(Debug, Serialize)]
struct Check {
    id: &'static str,
    label: &'static str,
    status: &'static str,
    severity: &'static str,
    detail: String,
    fix: String,
}

/// Health summary.
#[derive(Debug, Serialize)]
struct Summary {
    pass: usize,
    warn: usize,
    fail: usize,
}

/// Health response payload.
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    checks: Vec<Check>,
    summary: Summary,
}

/// Runs health checks and returns process exit code.
pub async fn run(output_mode: OutputMode, start: Instant, audit: &AuditLogger) -> i32 {
    let mut checks = Vec::new();

    checks.push(check_home());
    checks.push(check_config());
    checks.push(check_credentials());
    checks.push(check_audit());
    checks.push(check_auth_token().await);

    let pass = checks.iter().filter(|c| c.status == "pass").count();
    let warn = checks.iter().filter(|c| c.status == "warn").count();
    let fail = checks.iter().filter(|c| c.status == "fail").count();

    let blocked = checks
        .iter()
        .any(|check| check.status == "fail" && check.severity == "blocking");

    let status = if blocked {
        "blocked"
    } else if warn > 0 || fail > 0 {
        "degraded"
    } else {
        "ready"
    };

    let payload = HealthResponse {
        status,
        checks,
        summary: Summary { pass, warn, fail },
    };

    let output = match output_mode {
        OutputMode::Json => {
            envelope::emit_success("health.check", &payload, start, None, None, None)
        }
        OutputMode::Text => render_human(&payload),
        OutputMode::Table => format_value(&tabular_payload(&payload), OutputFormat::Table),
        OutputMode::Csv => format_value(&tabular_payload(&payload), OutputFormat::Csv),
    };

    println!("{output}");
    let _ = audit.log_command_output("health.check", &output);

    if blocked { 2 } else { 0 }
}

fn check_home() -> Check {
    match cho_sdk::home::ensure_cho_home() {
        Ok(path) => Check {
            id: "home",
            label: "Cho home directory",
            status: "pass",
            severity: "blocking",
            detail: format!("{}", path.display()),
            fix: "Create the directory and ensure it is writable".to_string(),
        },
        Err(err) => Check {
            id: "home",
            label: "Cho home directory",
            status: "fail",
            severity: "blocking",
            detail: err.to_string(),
            fix: "Set CHO_HOME or ensure HOME/TOOLS_HOME are valid".to_string(),
        },
    }
}

fn check_config() -> Check {
    match AppConfig::load() {
        Ok(_) => Check {
            id: "config",
            label: "Config file",
            status: "pass",
            severity: "info",
            detail: "Configuration loaded".to_string(),
            fix: "Run `cho config set <key> <value>`".to_string(),
        },
        Err(err) => Check {
            id: "config",
            label: "Config file",
            status: "fail",
            severity: "blocking",
            detail: err.to_string(),
            fix: "Repair ~/.tools/cho/config.toml or remove malformed file".to_string(),
        },
    }
}

fn check_credentials() -> Check {
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(err) => {
            return Check {
                id: "credentials",
                label: "OAuth credentials",
                status: "fail",
                severity: "blocking",
                detail: err.to_string(),
                fix: "Set auth.client_id and auth.client_secret".to_string(),
            };
        }
    };

    let client_id = config.resolve_client_id();
    let client_secret = config.resolve_client_secret();

    match (client_id, client_secret) {
        (Some(_), Some(_)) => Check {
            id: "credentials",
            label: "OAuth credentials",
            status: "pass",
            severity: "blocking",
            detail: "Client ID and secret available".to_string(),
            fix: "Run `cho config set auth.client_id ...` and `cho config set auth.client_secret ...`"
                .to_string(),
        },
        _ => Check {
            id: "credentials",
            label: "OAuth credentials",
            status: "fail",
            severity: "blocking",
            detail: "Missing client_id or client_secret".to_string(),
            fix: "Set CHO_CLIENT_ID/CHO_CLIENT_SECRET env vars or config values".to_string(),
        },
    }
}

fn check_audit() -> Check {
    let run_id = uuid::Uuid::new_v4().to_string();
    match AuditLogger::new(run_id).and_then(|logger| logger.verify_writable()) {
        Ok(()) => Check {
            id: "audit",
            label: "Audit log",
            status: "pass",
            severity: "blocking",
            detail: "history.log is writable".to_string(),
            fix: "Ensure ~/.tools/cho/history.log is writable".to_string(),
        },
        Err(err) => Check {
            id: "audit",
            label: "Audit log",
            status: "fail",
            severity: "blocking",
            detail: err.to_string(),
            fix: "Fix permissions for ~/.tools/cho/history.log".to_string(),
        },
    }
}

async fn check_auth_token() -> Check {
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(err) => {
            return Check {
                id: "auth",
                label: "Auth token",
                status: "fail",
                severity: "blocking",
                detail: err.to_string(),
                fix: "Run `cho auth login`".to_string(),
            };
        }
    };

    let Some(client_id) = config.resolve_client_id() else {
        return Check {
            id: "auth",
            label: "Auth token",
            status: "fail",
            severity: "blocking",
            detail: "Missing client id".to_string(),
            fix: "Set auth.client_id before login".to_string(),
        };
    };

    let Some(client_secret) = config.resolve_client_secret() else {
        return Check {
            id: "auth",
            label: "Auth token",
            status: "fail",
            severity: "blocking",
            detail: "Missing client secret".to_string(),
            fix: "Set auth.client_secret before login".to_string(),
        };
    };

    let auth = match AuthManager::new(
        client_id,
        SecretString::new(client_secret.into()),
        config.sdk_config(),
    ) {
        Ok(auth) => auth,
        Err(err) => {
            return Check {
                id: "auth",
                label: "Auth token",
                status: "fail",
                severity: "blocking",
                detail: err.to_string(),
                fix: "Fix auth credentials and config".to_string(),
            };
        }
    };

    let loaded = auth.load_stored_tokens().await.unwrap_or(false);
    if !loaded {
        return Check {
            id: "auth",
            label: "Auth token",
            status: "fail",
            severity: "blocking",
            detail: "No stored token found".to_string(),
            fix: "Run `cho auth login`".to_string(),
        };
    }

    let client = match FreeAgentClient::builder()
        .config(config.sdk_config())
        .auth_manager(auth)
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            return Check {
                id: "auth",
                label: "Auth token",
                status: "fail",
                severity: "blocking",
                detail: err.to_string(),
                fix: "Fix auth credentials and config".to_string(),
            };
        }
    };

    let status = client.session_status().await;
    if status.session_usable {
        let detail = if status.refresh_succeeded {
            format!(
                "Session usable after refresh+probe, expires_at={}",
                status.expires_at.unwrap_or_else(|| "unknown".to_string())
            )
        } else {
            format!(
                "Session usable, expires_at={}",
                status.expires_at.unwrap_or_else(|| "unknown".to_string())
            )
        };

        Check {
            id: "auth",
            label: "Auth token",
            status: if status.refresh_succeeded {
                "warn"
            } else {
                "pass"
            },
            severity: "blocking",
            detail,
            fix: "Run `cho auth refresh` or `cho auth login` if the session becomes unusable"
                .to_string(),
        }
    } else {
        Check {
            id: "auth",
            label: "Auth token",
            status: "fail",
            severity: "blocking",
            detail: status
                .probe_error
                .unwrap_or_else(|| "Trusted auth probe failed".to_string()),
            fix: "Run `cho auth login`".to_string(),
        }
    }
}

fn render_human(response: &HealthResponse) -> String {
    let mut out = String::new();
    out.push_str(&format!("status: {}\n\n", response.status));

    for check in &response.checks {
        let marker = match check.status {
            "pass" => "[+]",
            "warn" => "[~]",
            "fail" => "[x]",
            _ => "[?]",
        };

        out.push_str(&format!(
            "{} {:<20} {}\n",
            marker, check.label, check.detail
        ));
        if check.status != "pass" {
            out.push_str(&format!("    fix: {}\n", check.fix));
        }
    }

    out.push('\n');
    out.push_str(&format!(
        "summary: {} pass, {} warn, {} fail",
        response.summary.pass, response.summary.warn, response.summary.fail
    ));

    out
}

fn tabular_payload(response: &HealthResponse) -> serde_json::Value {
    serde_json::Value::Array(
        response
            .checks
            .iter()
            .map(|check| {
                serde_json::json!({
                    "overall_status": response.status,
                    "pass": response.summary.pass,
                    "warn": response.summary.warn,
                    "fail": response.summary.fail,
                    "id": check.id,
                    "label": check.label,
                    "status": check.status,
                    "severity": check.severity,
                    "detail": check.detail,
                    "fix": check.fix,
                })
            })
            .collect(),
    )
}
