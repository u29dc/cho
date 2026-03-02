//! Token storage helpers.

use std::io::Write;

use crate::error::{ChoSdkError, Result};
use crate::home;

use super::token::StoredTokens;

/// Loads stored tokens from file storage.
pub fn load_tokens() -> Result<Option<StoredTokens>> {
    load_from_file()
}

/// Stores tokens in file storage.
pub fn store_tokens(tokens: &StoredTokens) -> Result<()> {
    store_to_file(tokens)
}

/// Clears stored tokens from file storage.
pub fn clear_tokens() -> Result<()> {
    clear_file()
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

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .map_err(|e| ChoSdkError::Config {
            message: format!("Failed opening token file {}: {e}", path.display()),
        })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).map_err(|e| {
            ChoSdkError::Config {
                message: format!(
                    "Failed setting secure permissions on {}: {e}",
                    path.display()
                ),
            }
        })?;
    }

    file.write_all(raw.as_bytes())
        .map_err(|e| ChoSdkError::Config {
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
