//! Shared `cho` home path helpers.

use std::path::PathBuf;

use crate::error::{ChoSdkError, Result};

/// Resolves the `cho` home directory.
///
/// Resolution order:
/// 1. `CHO_HOME`
/// 2. `TOOLS_HOME/cho`
/// 3. `$HOME/.tools/cho`
pub fn resolve_cho_home() -> Result<PathBuf> {
    if let Ok(cho_home) = std::env::var("CHO_HOME") {
        return Ok(PathBuf::from(cho_home));
    }

    if let Ok(tools_home) = std::env::var("TOOLS_HOME") {
        return Ok(PathBuf::from(tools_home).join("cho"));
    }

    let home = std::env::var("HOME").map_err(|_| ChoSdkError::Config {
        message: "HOME environment variable is not set".to_string(),
    })?;

    Ok(PathBuf::from(home).join(".tools").join("cho"))
}

/// Ensures the `cho` home directory exists.
pub fn ensure_cho_home() -> Result<PathBuf> {
    let path = resolve_cho_home()?;
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| ChoSdkError::Config {
            message: format!(
                "Failed to create cho home directory {}: {e}",
                path.display()
            ),
        })?;
    }
    Ok(path)
}

/// Path to config TOML.
pub fn config_path() -> Result<PathBuf> {
    Ok(ensure_cho_home()?.join("config.toml"))
}

/// Path to command history log.
pub fn history_log_path() -> Result<PathBuf> {
    Ok(ensure_cho_home()?.join("history.log"))
}

/// Path to token store file.
pub fn token_path() -> Result<PathBuf> {
    Ok(ensure_cho_home()?.join("tokens.json"))
}

/// Path to TUI route cache file.
pub fn tui_cache_path() -> Result<PathBuf> {
    Ok(ensure_cho_home()?.join("tui-cache.json"))
}
