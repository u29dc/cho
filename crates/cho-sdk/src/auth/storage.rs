//! Token storage helpers.

use tracing::warn;

use crate::error::{ChoSdkError, Result};
use crate::home;

use super::token::StoredTokens;

const SERVICE_NAME: &str = "cho";
const TOKENS_KEY: &str = "freeagent_tokens";

/// Loads stored tokens from keyring, then fallback file.
pub fn load_tokens() -> Result<Option<StoredTokens>> {
    match load_from_keyring() {
        Ok(Some(tokens)) => return Ok(Some(tokens)),
        Ok(None) => {}
        Err(err) => {
            tracing::debug!("keyring token load failed: {err}");
        }
    }

    load_from_file()
}

/// Stores tokens to keyring, falling back to file if keyring is unavailable.
pub fn store_tokens(tokens: &StoredTokens) -> Result<()> {
    match store_to_keyring(tokens) {
        Ok(()) => Ok(()),
        Err(keyring_err) => {
            warn!("Keyring unavailable, using file fallback token storage: {keyring_err}");
            store_to_file(tokens)
        }
    }
}

/// Clears stored tokens from keyring and file fallback.
pub fn clear_tokens() -> Result<()> {
    if let Err(err) = clear_keyring() {
        tracing::debug!("keyring clear failed: {err}");
    }

    if let Err(err) = clear_file() {
        tracing::debug!("token file clear failed: {err}");
    }

    Ok(())
}

fn load_from_keyring() -> Result<Option<StoredTokens>> {
    let entry = keyring::Entry::new(SERVICE_NAME, TOKENS_KEY).map_err(|e| ChoSdkError::Config {
        message: format!("Failed to initialize keyring entry: {e}"),
    })?;

    match entry.get_secret() {
        Ok(bytes) => {
            let raw = String::from_utf8(bytes).map_err(|e| ChoSdkError::Config {
                message: format!("Stored token bytes are not UTF-8: {e}"),
            })?;
            let tokens =
                serde_json::from_str::<StoredTokens>(&raw).map_err(|e| ChoSdkError::Config {
                    message: format!("Failed parsing stored keyring tokens: {e}"),
                })?;
            Ok(Some(tokens))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(ChoSdkError::Config {
            message: format!("Failed reading tokens from keyring: {e}"),
        }),
    }
}

fn store_to_keyring(tokens: &StoredTokens) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, TOKENS_KEY).map_err(|e| ChoSdkError::Config {
        message: format!("Failed to initialize keyring entry: {e}"),
    })?;

    let raw = serde_json::to_string(tokens).map_err(|e| ChoSdkError::Config {
        message: format!("Failed serializing tokens for keyring: {e}"),
    })?;

    entry
        .set_secret(raw.as_bytes())
        .map_err(|e| ChoSdkError::Config {
            message: format!("Failed writing tokens to keyring: {e}"),
        })
}

fn clear_keyring() -> Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, TOKENS_KEY).map_err(|e| ChoSdkError::Config {
        message: format!("Failed to initialize keyring entry: {e}"),
    })?;

    entry.delete_credential().map_err(|e| ChoSdkError::Config {
        message: format!("Failed deleting keyring token entry: {e}"),
    })
}

fn load_from_file() -> Result<Option<StoredTokens>> {
    let path = home::token_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let raw = std::fs::read_to_string(&path).map_err(|e| ChoSdkError::Config {
        message: format!("Failed reading token file {}: {e}", path.display()),
    })?;

    let tokens = serde_json::from_str::<StoredTokens>(&raw).map_err(|e| ChoSdkError::Config {
        message: format!("Failed parsing token file {}: {e}", path.display()),
    })?;

    Ok(Some(tokens))
}

fn store_to_file(tokens: &StoredTokens) -> Result<()> {
    let path = home::token_path()?;
    let raw = serde_json::to_string(tokens).map_err(|e| ChoSdkError::Config {
        message: format!("Failed serializing tokens for file storage: {e}"),
    })?;

    std::fs::write(&path, raw).map_err(|e| ChoSdkError::Config {
        message: format!("Failed writing token file {}: {e}", path.display()),
    })
}

fn clear_file() -> Result<()> {
    let path = home::token_path()?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| ChoSdkError::Config {
            message: format!("Failed deleting token file {}: {e}", path.display()),
        })?;
    }
    Ok(())
}
