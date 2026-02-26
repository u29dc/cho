//! Append-only CLI audit log.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use fs2::FileExt;

use cho_sdk::client::{HttpObserver, HttpRequestEvent, HttpResponseEvent};
use cho_sdk::error::{ChoSdkError, Result};
use cho_sdk::home;

/// CLI audit logger backed by `history.log`.
#[derive(Clone)]
pub struct AuditLogger {
    path: PathBuf,
    run_id: String,
    lock: Arc<Mutex<()>>,
}

impl AuditLogger {
    /// Creates an audit logger for the given run id.
    pub fn new(run_id: String) -> Result<Self> {
        let path = home::history_log_path()?;

        if !path.exists() {
            std::fs::File::create(&path).map_err(|e| ChoSdkError::Config {
                message: format!("Failed creating history log {}: {e}", path.display()),
            })?;
        }

        Ok(Self {
            path,
            run_id,
            lock: Arc::new(Mutex::new(())),
        })
    }

    /// Logs an event with arbitrary fields.
    pub fn log_event(&self, event: &str, fields: &[(&str, String)]) -> Result<()> {
        let _guard = self.lock.lock().map_err(|_| ChoSdkError::Config {
            message: "Audit log lock was poisoned".to_string(),
        })?;

        let timestamp = Utc::now().to_rfc3339();
        let mut line = format!("{} | run={} | event={}", timestamp, self.run_id, event);

        for (key, value) in fields {
            line.push_str(" | ");
            line.push_str(key);
            line.push('=');
            line.push_str(&escape_field(value));
        }

        line.push('\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| ChoSdkError::Config {
                message: format!("Failed opening history log {}: {e}", self.path.display()),
            })?;

        file.lock_exclusive().map_err(|e| ChoSdkError::Config {
            message: format!("Failed locking history log {}: {e}", self.path.display()),
        })?;

        let write_result = file
            .write_all(line.as_bytes())
            .map_err(|e| ChoSdkError::Config {
                message: format!("Failed writing history log {}: {e}", self.path.display()),
            });

        let unlock_result = FileExt::unlock(&file).map_err(|e| ChoSdkError::Config {
            message: format!("Failed unlocking history log {}: {e}", self.path.display()),
        });

        write_result?;
        unlock_result?;

        Ok(())
    }

    /// Logs command invocation start.
    pub fn log_command_start(&self, tool: &str, argv: &[String]) -> Result<()> {
        self.log_event(
            "command.start",
            &[
                ("tool", tool.to_string()),
                ("argv", sanitize_argv(argv).join(" ")),
            ],
        )
    }

    /// Logs structured command input.
    pub fn log_command_input(&self, tool: &str, input: &str) -> Result<()> {
        self.log_event(
            "command.input",
            &[("tool", tool.to_string()), ("input", input.to_string())],
        )
    }

    /// Logs command output payload.
    pub fn log_command_output(&self, tool: &str, output: &str) -> Result<()> {
        self.log_event(
            "command.output",
            &[("tool", tool.to_string()), ("output", output.to_string())],
        )
    }

    /// Logs command completion.
    pub fn log_command_end(&self, tool: &str, exit_code: i32, elapsed_ms: u64) -> Result<()> {
        self.log_event(
            "command.end",
            &[
                ("tool", tool.to_string()),
                ("exit_code", exit_code.to_string()),
                ("elapsed_ms", elapsed_ms.to_string()),
            ],
        )
    }

    /// Verifies append access to the log file.
    pub fn verify_writable(&self) -> Result<()> {
        self.log_event("audit.verify", &[("status", "ok".to_string())])
    }
}

impl HttpObserver for AuditLogger {
    fn on_request(&self, event: &HttpRequestEvent) {
        let _ = self.log_event(
            "http.request",
            &[
                ("method", event.method.clone()),
                ("url", event.url.clone()),
                (
                    "query",
                    event
                        .query
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join("&"),
                ),
                ("has_body", event.has_body.to_string()),
                ("mutating", event.mutating.to_string()),
            ],
        );
    }

    fn on_response(&self, event: &HttpResponseEvent) {
        let _ = self.log_event(
            "http.response",
            &[
                ("method", event.method.clone()),
                ("url", event.url.clone()),
                (
                    "status",
                    event
                        .status
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                ),
                ("elapsed_ms", event.elapsed_ms.to_string()),
                (
                    "retry_after",
                    event
                        .retry_after
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                ),
                (
                    "error",
                    event.error.clone().unwrap_or_else(|| "none".to_string()),
                ),
            ],
        );
    }
}

fn escape_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('|', "\\|")
}

fn sanitize_argv(argv: &[String]) -> Vec<String> {
    let mut sanitized = Vec::with_capacity(argv.len());
    let mut redact_next = false;

    for arg in argv {
        if redact_next {
            sanitized.push("[REDACTED]".to_string());
            redact_next = false;
            continue;
        }

        if arg == "--client-secret" {
            sanitized.push(arg.clone());
            redact_next = true;
            continue;
        }

        if let Some((prefix, _)) = arg.split_once("--client-secret=") {
            let _ = prefix;
            sanitized.push("--client-secret=[REDACTED]".to_string());
            continue;
        }

        sanitized.push(arg.clone());
    }

    sanitized
}
