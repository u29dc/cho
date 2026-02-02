//! Token storage: OS keyring primary, JSON file fallback.
//!
//! Tokens are stored in the OS keychain via the `keyring` crate when available.
//! When the keychain is not accessible (headless Linux, CI, containers), tokens
//! are written to `~/.config/cho/tokens.json` with `0600` permissions.
//!
//! # Security note
//!
//! The file fallback stores tokens as **plaintext JSON** on disk. While file
//! permissions are restricted to `0600` (owner-only read/write), any process
//! running as the same user can read the tokens. The spec originally called for
//! encrypted file storage (`tokens.enc`), but this was simplified to plaintext
//! JSON for the MVP. Consider using the OS keychain (macOS Keychain, GNOME
//! Keyring, Windows Credential Manager) in production environments. A warning
//! is emitted via `tracing::warn!` when the file fallback is used for storage.

use std::path::PathBuf;

use tracing::{debug, warn};

use super::token::StoredTokens;

/// Keyring service name.
const SERVICE: &str = "cho";

/// Keyring username for the token blob.
const USERNAME: &str = "tokens";

/// Loads stored tokens from the OS keychain or file fallback.
pub(crate) fn load_tokens() -> crate::error::Result<Option<StoredTokens>> {
    // Try keyring first
    match load_from_keyring() {
        Ok(Some(tokens)) => {
            debug!("Loaded tokens from OS keychain");
            return Ok(Some(tokens));
        }
        Ok(None) => {
            debug!("No tokens in OS keychain");
        }
        Err(e) => {
            debug!("Keychain unavailable: {e}, trying file fallback");
        }
    }

    // File fallback
    match load_from_file() {
        Ok(Some(tokens)) => {
            debug!("Loaded tokens from file fallback");
            Ok(Some(tokens))
        }
        Ok(None) => {
            debug!("No stored tokens found");
            Ok(None)
        }
        Err(e) => {
            warn!("Failed to load tokens from file: {e}");
            Ok(None)
        }
    }
}

/// Stores tokens to the OS keychain and file fallback.
pub(crate) fn store_tokens(tokens: &StoredTokens) -> crate::error::Result<()> {
    let json = serde_json::to_string(tokens).map_err(|e| crate::error::ChoSdkError::Config {
        message: format!("Failed to serialize tokens: {e}"),
    })?;

    // Try keyring first
    match store_to_keyring(&json) {
        Ok(()) => {
            debug!("Stored tokens in OS keychain");
            return Ok(());
        }
        Err(e) => {
            debug!("Keychain storage failed: {e}, using file fallback");
        }
    }

    // File fallback only when keyring is unavailable
    store_to_file(&json)?;
    warn!(
        "Tokens stored as plaintext JSON at ~/.config/cho/tokens.json (0600 permissions). \
         Use OS keychain for production environments."
    );

    Ok(())
}

/// Removes stored tokens from all backends.
pub fn clear_tokens() -> crate::error::Result<()> {
    // Try keyring
    if let Err(e) = clear_keyring() {
        debug!("Keychain clear failed (may not exist): {e}");
    }

    // Try file
    if let Err(e) = clear_file() {
        debug!("File clear failed (may not exist): {e}");
    }

    Ok(())
}

// --- Keyring backend ---

fn load_from_keyring() -> crate::error::Result<Option<StoredTokens>> {
    let entry =
        keyring::Entry::new(SERVICE, USERNAME).map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Keyring entry error: {e}"),
        })?;

    match entry.get_secret() {
        Ok(bytes) => {
            let json = String::from_utf8(bytes).map_err(|e| crate::error::ChoSdkError::Config {
                message: format!("Keyring data is not valid UTF-8: {e}"),
            })?;
            let tokens: StoredTokens =
                serde_json::from_str(&json).map_err(|e| crate::error::ChoSdkError::Config {
                    message: format!("Keyring data parse error: {e}"),
                })?;
            Ok(Some(tokens))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(crate::error::ChoSdkError::Config {
            message: format!("Keyring read error: {e}"),
        }),
    }
}

fn store_to_keyring(json: &str) -> crate::error::Result<()> {
    let entry =
        keyring::Entry::new(SERVICE, USERNAME).map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Keyring entry error: {e}"),
        })?;

    entry
        .set_secret(json.as_bytes())
        .map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Keyring store error: {e}"),
        })
}

fn clear_keyring() -> crate::error::Result<()> {
    let entry =
        keyring::Entry::new(SERVICE, USERNAME).map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Keyring entry error: {e}"),
        })?;

    entry
        .delete_credential()
        .map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Keyring delete error: {e}"),
        })
}

// --- File fallback ---

/// Returns the token file path: `~/.config/cho/tokens.json`.
fn token_file_path() -> crate::error::Result<PathBuf> {
    let config_dir = dirs_path()?;
    Ok(config_dir.join("tokens.json"))
}

/// Returns the cho config directory, creating it if needed.
fn dirs_path() -> crate::error::Result<PathBuf> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".config")
        });

    let dir = base.join("cho");

    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Failed to create config directory {}: {e}", dir.display()),
        })?;
    }

    Ok(dir)
}

fn load_from_file() -> crate::error::Result<Option<StoredTokens>> {
    let path = token_file_path()?;

    if !path.exists() {
        return Ok(None);
    }

    let content =
        std::fs::read_to_string(&path).map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Failed to read token file {}: {e}", path.display()),
        })?;

    let tokens: StoredTokens =
        serde_json::from_str(&content).map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Failed to parse token file: {e}"),
        })?;

    Ok(Some(tokens))
}

fn store_to_file(json: &str) -> crate::error::Result<()> {
    let path = token_file_path()?;

    std::fs::write(&path, json).map_err(|e| crate::error::ChoSdkError::Config {
        message: format!("Failed to write token file {}: {e}", path.display()),
    })?;

    // Set restrictive permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, permissions).map_err(|e| {
            crate::error::ChoSdkError::Config {
                message: format!("Failed to set token file permissions: {e}"),
            }
        })?;
    }

    Ok(())
}

fn clear_file() -> crate::error::Result<()> {
    let path = token_file_path()?;

    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Failed to remove token file: {e}"),
        })?;
    }

    Ok(())
}

/// Returns the path to the cho config directory.
pub fn config_dir() -> crate::error::Result<PathBuf> {
    dirs_path()
}

/// Stores a client ID to the config file for later retrieval.
pub fn store_client_id(client_id: &str) -> crate::error::Result<()> {
    let entry = keyring::Entry::new(SERVICE, "client_id").map_err(|e| {
        crate::error::ChoSdkError::Config {
            message: format!("Keyring entry error: {e}"),
        }
    })?;

    match entry.set_secret(client_id.as_bytes()) {
        Ok(()) => Ok(()),
        Err(e) => {
            debug!("Keychain client_id storage failed: {e}, skipping");
            Ok(())
        }
    }
}

/// Loads the stored client ID.
pub fn load_client_id() -> crate::error::Result<Option<String>> {
    let entry = keyring::Entry::new(SERVICE, "client_id").map_err(|e| {
        crate::error::ChoSdkError::Config {
            message: format!("Keyring entry error: {e}"),
        }
    })?;

    match entry.get_secret() {
        Ok(bytes) => {
            let id = String::from_utf8(bytes).map_err(|e| crate::error::ChoSdkError::Config {
                message: format!("Client ID is not valid UTF-8: {e}"),
            })?;
            Ok(Some(id))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => {
            debug!("Keychain client_id read failed: {e}");
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_file_path_is_under_config() {
        let path = token_file_path().unwrap();
        assert!(path.to_string_lossy().contains("cho"));
        assert!(path.to_string_lossy().ends_with("tokens.json"));
    }

    #[test]
    fn stored_tokens_serialize_round_trip() {
        let tokens = StoredTokens {
            access_token: "access123".to_string(),
            refresh_token: Some("refresh456".to_string()),
            expires_in: 1800,
            issued_at: 1700000000,
        };
        let json = serde_json::to_string(&tokens).unwrap();
        let parsed: StoredTokens = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.access_token, "access123");
        assert_eq!(parsed.refresh_token.as_deref(), Some("refresh456"));
    }
}
