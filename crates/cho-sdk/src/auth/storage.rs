//! Token storage: OS keyring required (fail-closed security model).
//!
//! Tokens are stored in the OS keychain via the `keyring` crate. The keychain
//! provides secure, encrypted storage managed by the operating system (macOS
//! Keychain, GNOME Keyring, Windows Credential Manager).
//!
//! # Security model
//!
//! This module uses a **fail-closed** approach: if the OS keychain is not
//! available (headless Linux without a keyring daemon, CI, containers), token
//! storage will fail with an error rather than falling back to plaintext files.
//!
//! For environments without a keychain, consider:
//! - Using `--client-credentials` auth with env vars (no token persistence)
//! - Running a keyring daemon (e.g., `gnome-keyring-daemon --start`)
//! - Using a secrets manager and injecting tokens via environment
//!
//! # Legacy file migration
//!
//! For backwards compatibility, `load_tokens()` will still read tokens from
//! the legacy file location (`~/.config/cho/tokens.json`) if it exists, but
//! will warn the user to re-authenticate. New tokens are never written to disk.

use std::path::PathBuf;

use tracing::{debug, warn};

use super::token::StoredTokens;

/// Keyring service name.
const SERVICE: &str = "cho";

/// Keyring username for the token blob.
const USERNAME: &str = "tokens";

/// Loads stored tokens from the OS keychain (with legacy file migration).
///
/// Tokens are loaded from the OS keychain. For backwards compatibility, if no
/// keychain tokens exist, the legacy file location is checked. If legacy tokens
/// are found, a warning is emitted encouraging re-authentication to migrate to
/// secure keychain storage.
pub(crate) fn load_tokens() -> crate::error::Result<Option<StoredTokens>> {
    // Try keyring first (primary path)
    match load_from_keyring() {
        Ok(Some(tokens)) => {
            debug!("Loaded tokens from OS keychain");
            return Ok(Some(tokens));
        }
        Ok(None) => {
            debug!("No tokens in OS keychain");
        }
        Err(e) => {
            debug!("Keychain unavailable: {e}, checking legacy file");
        }
    }

    // Legacy file migration path - read-only for backwards compatibility
    match load_from_file() {
        Ok(Some(tokens)) => {
            warn!(
                "Loaded tokens from legacy plaintext file (~/.config/cho/tokens.json). \
                 Please run `cho auth login` to migrate to secure OS keychain storage, \
                 then delete the legacy file."
            );
            Ok(Some(tokens))
        }
        Ok(None) => {
            debug!("No stored tokens found");
            Ok(None)
        }
        Err(e) => {
            debug!("Legacy file read failed: {e}");
            Ok(None)
        }
    }
}

/// Stores tokens to the OS keychain (fail-closed, no file fallback).
///
/// # Errors
///
/// Returns an error if the OS keychain is not available. This is intentional:
/// we refuse to store tokens insecurely rather than falling back to plaintext.
pub(crate) fn store_tokens(tokens: &StoredTokens) -> crate::error::Result<()> {
    let json = serde_json::to_string(tokens).map_err(|e| crate::error::ChoSdkError::Config {
        message: format!("Failed to serialize tokens: {e}"),
    })?;

    // Keyring only - fail closed if unavailable
    match store_to_keyring(&json) {
        Ok(()) => {
            debug!("Stored tokens in OS keychain");
            Ok(())
        }
        Err(e) => {
            warn!(
                "OS keychain unavailable. Tokens cannot be persisted securely. \
                 Consider using --client-credentials auth or starting a keyring daemon."
            );
            Err(e)
        }
    }
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
        .or_else(|_| std::env::var("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map_err(|_| crate::error::ChoSdkError::Config {
            message: "Neither XDG_CONFIG_HOME nor HOME environment variable is set".to_string(),
        })?;

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
