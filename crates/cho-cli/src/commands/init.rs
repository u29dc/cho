//! Interactive first-time setup command.
//!
//! `cho init` replaces the manual multi-step setup (create app, login, pick tenant,
//! set config) with a single guided flow. Runs before `CliContext` construction
//! since it establishes the client_id and tenant_id that `CliContext` requires.

use std::io::{IsTerminal, Write as _, stderr, stdin};
use std::path::PathBuf;

use clap::Args;

use cho_sdk::auth::AuthManager;
use cho_sdk::auth::storage;
use cho_sdk::client::XeroClient;
use cho_sdk::config::SdkConfig;
use cho_sdk::error::ChoSdkError;
use cho_sdk::http::rate_limit::RateLimitConfig;

/// Interactive first-time setup: authenticate and select a tenant.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Xero OAuth 2.0 client ID (from developer.xero.com).
    #[arg(long, env = "CHO_CLIENT_ID")]
    client_id: Option<String>,

    /// Port for localhost callback server (0 = auto).
    #[arg(long, default_value = "0")]
    port: u16,
}

/// Runs the interactive init flow.
pub async fn run(args: &InitArgs) -> cho_sdk::error::Result<()> {
    // Ensure tracing is at info level so PKCE auth URL is visible.
    let filter = tracing_subscriber::EnvFilter::new("info");
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(stderr)
        .try_init();

    // 1. TTY gate
    ensure_tty()?;

    // 2. Detect existing config
    let config_path = config_file_path()?;
    let existing = load_existing_config(&config_path);

    if let Some(ref existing) = existing
        && existing.client_id.is_some()
        && existing.tenant_id.is_some()
    {
        emsg(&format!(
            "Existing configuration found at {}",
            config_path.display()
        ));
        if let Some(ref cid) = existing.client_id {
            emsg(&format!("  client_id: {}...", truncate_id(cid)));
        }
        if let Some(ref tid) = existing.tenant_id {
            emsg(&format!("  tenant_id: {}...", truncate_id(tid)));
        }
        if !prompt_yes_no("Reconfigure?", false)? {
            emsg("Aborted.");
            return Ok(());
        }
    }

    // 3. Resolve client_id
    let client_id = resolve_client_id(args, existing.as_ref())?;

    // 4. PKCE auth
    emsg("");
    emsg("Starting OAuth 2.0 PKCE authentication...");
    emsg("Your browser will open to authorize cho with Xero.");
    emsg("");

    let auth = AuthManager::new(client_id.clone());
    auth.login_pkce(args.port).await?;

    emsg("");
    emsg("Authentication successful!");

    // 5. Fetch tenants
    emsg("");
    emsg("Fetching connected organisations...");

    let config = SdkConfig::default();
    let client = XeroClient::builder()
        .config(config)
        .tenant_id(String::new())
        .auth_manager(auth)
        .rate_limit(RateLimitConfig::default())
        .build()?;

    let connections = client.identity().connections().await?;

    // 6. Tenant selection
    let (tenant_id, tenant_name) = match connections.len() {
        0 => {
            return Err(ChoSdkError::Config {
                message: "No connected organisations found. \
                          Authorize your app for at least one organisation at \
                          https://developer.xero.com"
                    .to_string(),
            });
        }
        1 => {
            let conn = &connections[0];
            let tid = conn.tenant_id.map(|id| id.to_string()).unwrap_or_default();
            let name = conn.tenant_name.as_deref().unwrap_or("Unknown").to_string();
            emsg(&format!("Selected organisation: {name} ({tid})"));
            (tid, name)
        }
        n => {
            emsg("");
            emsg("Multiple organisations found:");
            for (i, conn) in connections.iter().enumerate() {
                let name = conn.tenant_name.as_deref().unwrap_or("Unknown");
                let tid = conn.tenant_id.map(|id| id.to_string()).unwrap_or_default();
                emsg(&format!("  {}: {} ({})", i + 1, name, tid));
            }
            let choice = prompt_number("Select organisation", 1, n)?;
            let conn = &connections[choice - 1];
            let tid = conn.tenant_id.map(|id| id.to_string()).unwrap_or_default();
            let name = conn.tenant_name.as_deref().unwrap_or("Unknown").to_string();
            (tid, name)
        }
    };

    // 7. Write config
    write_config(&config_path, &client_id, &tenant_id, existing.as_ref())?;

    // 8. Store client_id in keyring (best effort)
    let _ = storage::store_client_id(&client_id);

    // 9. Success summary
    emsg("");
    emsg("Setup complete!");
    emsg(&format!("  Config: {}", config_path.display()));
    emsg(&format!("  Client: {}...", truncate_id(&client_id)));
    emsg(&format!("  Tenant: {} ({})", tenant_name, tenant_id));
    emsg("");
    emsg("Try: cho invoices list --limit 5");

    Ok(())
}

// --- Helpers ---

/// Aborts if stdin is not a terminal.
fn ensure_tty() -> cho_sdk::error::Result<()> {
    if !stdin().is_terminal() {
        return Err(ChoSdkError::Config {
            message: "cho init requires an interactive terminal.\n\
                      For non-interactive setup, use:\n  \
                      cho auth login\n  \
                      cho config set auth.tenant_id <TENANT_ID>"
                .to_string(),
        });
    }
    Ok(())
}

/// Prints a message to stderr.
fn emsg(msg: &str) {
    eprintln!("{msg}");
}

/// Prompts for text input on stderr, reads from stdin.
fn prompt(msg: &str) -> cho_sdk::error::Result<String> {
    eprint!("{msg}: ");
    stderr().flush().ok();
    let mut buf = String::new();
    stdin()
        .read_line(&mut buf)
        .map_err(|e| ChoSdkError::Config {
            message: format!("Failed to read input: {e}"),
        })?;
    Ok(buf.trim().to_string())
}

/// Prompts for yes/no with a default.
fn prompt_yes_no(msg: &str, default: bool) -> cho_sdk::error::Result<bool> {
    let hint = if default { "[Y/n]" } else { "[y/N]" };
    let input = prompt(&format!("{msg} {hint}"))?;
    if input.is_empty() {
        return Ok(default);
    }
    Ok(input.eq_ignore_ascii_case("y") || input.eq_ignore_ascii_case("yes"))
}

/// Prompts for a number within a range.
fn prompt_number(msg: &str, min: usize, max: usize) -> cho_sdk::error::Result<usize> {
    loop {
        let input = prompt(&format!("{msg} [{min}-{max}]"))?;
        if let Ok(n) = input.parse::<usize>()
            && n >= min
            && n <= max
        {
            return Ok(n);
        }
        emsg(&format!("Please enter a number between {min} and {max}."));
    }
}

/// Truncates an ID string for display (first 8 chars).
fn truncate_id(id: &str) -> &str {
    if id.len() > 8 { &id[..8] } else { id }
}

/// Resolves client_id from: flag > env > existing config > keyring > interactive prompt.
fn resolve_client_id(
    args: &InitArgs,
    existing: Option<&ExistingConfig>,
) -> cho_sdk::error::Result<String> {
    // Flag (already handled by clap env integration)
    if let Some(ref id) = args.client_id
        && !id.is_empty()
    {
        return Ok(id.clone());
    }

    // Existing config
    if let Some(existing) = existing
        && let Some(ref id) = existing.client_id
        && !id.is_empty()
    {
        emsg(&format!(
            "Using client_id from config: {}...",
            truncate_id(id),
        ));
        return Ok(id.clone());
    }

    // Keyring
    if let Ok(Some(id)) = storage::load_client_id()
        && !id.is_empty()
    {
        emsg(&format!(
            "Using client_id from keyring: {}...",
            truncate_id(&id),
        ));
        return Ok(id);
    }

    // Interactive prompt
    emsg("Create a Xero app at https://developer.xero.com/app/manage");
    emsg("Copy the Client ID from your app's configuration page.");
    emsg("");
    let id = prompt("Client ID")?;
    if id.is_empty() {
        return Err(ChoSdkError::Config {
            message: "Client ID is required.".to_string(),
        });
    }
    Ok(id)
}

/// Parsed values from an existing config file.
struct ExistingConfig {
    client_id: Option<String>,
    tenant_id: Option<String>,
    /// The full TOML table, preserved for merging.
    table: toml::Table,
}

/// Returns the config file path: `~/.config/cho/config.toml`.
fn config_file_path() -> cho_sdk::error::Result<PathBuf> {
    let dir = storage::config_dir()?;
    Ok(dir.join("config.toml"))
}

/// Loads existing config, returning None if file doesn't exist or can't be parsed.
fn load_existing_config(path: &PathBuf) -> Option<ExistingConfig> {
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    let table: toml::Table = content.parse().ok()?;

    let client_id = table
        .get("auth")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("client_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let tenant_id = table
        .get("auth")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("tenant_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(ExistingConfig {
        client_id,
        tenant_id,
        table,
    })
}

/// Writes/merges client_id and tenant_id into the config TOML, preserving other sections.
fn write_config(
    path: &PathBuf,
    client_id: &str,
    tenant_id: &str,
    existing: Option<&ExistingConfig>,
) -> cho_sdk::error::Result<()> {
    let mut table = existing.map(|e| e.table.clone()).unwrap_or_default();

    // Ensure [auth] section exists
    if !table.contains_key("auth") {
        table.insert("auth".to_string(), toml::Value::Table(toml::Table::new()));
    }

    if let Some(auth) = table.get_mut("auth").and_then(|v| v.as_table_mut()) {
        auth.insert(
            "client_id".to_string(),
            toml::Value::String(client_id.to_string()),
        );
        auth.insert(
            "tenant_id".to_string(),
            toml::Value::String(tenant_id.to_string()),
        );
    }

    let content = toml::to_string_pretty(&table).map_err(|e| ChoSdkError::Config {
        message: format!("Failed to serialize config: {e}"),
    })?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ChoSdkError::Config {
            message: format!("Failed to create config directory: {e}"),
        })?;
    }

    std::fs::write(path, content).map_err(|e| ChoSdkError::Config {
        message: format!("Failed to write config file {}: {e}", path.display()),
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_id_short() {
        assert_eq!(truncate_id("abc"), "abc");
    }

    #[test]
    fn truncate_id_long() {
        assert_eq!(truncate_id("abcdefgh-1234-5678"), "abcdefgh");
    }

    #[test]
    fn write_config_creates_new() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        write_config(&path, "my-client-id", "my-tenant-id", None).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let table: toml::Table = content.parse().unwrap();

        let auth = table.get("auth").unwrap().as_table().unwrap();
        assert_eq!(
            auth.get("client_id").unwrap().as_str(),
            Some("my-client-id")
        );
        assert_eq!(
            auth.get("tenant_id").unwrap().as_str(),
            Some("my-tenant-id")
        );
    }

    #[test]
    fn write_config_preserves_existing_sections() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        // Write initial config with extra sections
        let initial = "[defaults]\nformat = \"table\"\n\n[safety]\nallow_writes = true\n";
        std::fs::write(&path, initial).unwrap();

        let existing = load_existing_config(&path).unwrap();
        write_config(&path, "new-id", "new-tenant", Some(&existing)).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let table: toml::Table = content.parse().unwrap();

        // Auth section updated
        let auth = table.get("auth").unwrap().as_table().unwrap();
        assert_eq!(auth.get("client_id").unwrap().as_str(), Some("new-id"));
        assert_eq!(auth.get("tenant_id").unwrap().as_str(), Some("new-tenant"));

        // Other sections preserved
        let defaults = table.get("defaults").unwrap().as_table().unwrap();
        assert_eq!(defaults.get("format").unwrap().as_str(), Some("table"));

        let safety = table.get("safety").unwrap().as_table().unwrap();
        assert_eq!(safety.get("allow_writes").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn load_existing_config_missing_file() {
        let path = PathBuf::from("/tmp/nonexistent-cho-test/config.toml");
        assert!(load_existing_config(&path).is_none());
    }

    #[test]
    fn load_existing_config_partial() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        std::fs::write(&path, "[auth]\nclient_id = \"test-id\"\n").unwrap();

        let config = load_existing_config(&path).unwrap();
        assert_eq!(config.client_id.as_deref(), Some("test-id"));
        assert!(config.tenant_id.is_none());
    }
}
