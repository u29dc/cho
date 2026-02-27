//! Token storage helpers.

use std::io::Write;

use tracing::warn;

use crate::error::{ChoSdkError, Result};
use crate::home;

use super::token::StoredTokens;

const SERVICE_NAME: &str = "cho";
const TOKENS_KEY: &str = "freeagent_tokens";
const FILE_FALLBACK_ENV: &str = "CHO_ALLOW_INSECURE_FILE_TOKENS";

/// Loads stored tokens from keyring, then fallback file.
pub fn load_tokens() -> Result<Option<StoredTokens>> {
    if !keyring_disabled() {
        match load_from_keyring() {
            Ok(Some(tokens)) => return Ok(Some(tokens)),
            Ok(None) => {
                if file_fallback_allowed()
                    && let Some(tokens) = load_from_file()?
                {
                    match store_to_keyring(&tokens) {
                        Ok(()) => {
                            if let Err(err) = clear_file() {
                                tracing::debug!(
                                    "failed removing migrated plaintext token file: {err}"
                                );
                            }
                        }
                        Err(err) => {
                            warn!("failed migrating plaintext tokens into keyring: {err}");
                        }
                    }
                    return Ok(Some(tokens));
                }
                return Ok(None);
            }
            Err(err) => {
                if file_fallback_allowed() {
                    warn!("Keyring unavailable, using insecure file fallback token storage: {err}");
                    return load_from_file();
                }

                return Err(ChoSdkError::Config {
                    message: format!(
                        "Secure keyring token storage is unavailable: {err}. \
Set {FILE_FALLBACK_ENV}=true to allow plaintext fallback token storage."
                    ),
                });
            }
        }
    }

    if !file_fallback_allowed() {
        return Err(ChoSdkError::Config {
            message: format!(
                "Keyring storage is disabled and plaintext fallback is blocked. \
Set {FILE_FALLBACK_ENV}=true to allow plaintext fallback token storage."
            ),
        });
    }

    load_from_file()
}

/// Stores tokens in secure keyring storage, with optional file fallback.
pub fn store_tokens(tokens: &StoredTokens) -> Result<()> {
    if !keyring_disabled() {
        match store_to_keyring(tokens) {
            Ok(()) => {
                if let Err(err) = clear_file() {
                    tracing::debug!("failed removing stale plaintext token file: {err}");
                }
                return Ok(());
            }
            Err(keyring_err) => {
                if !file_fallback_allowed() {
                    return Err(ChoSdkError::Config {
                        message: format!(
                            "Secure keyring token storage is unavailable: {keyring_err}. \
Set {FILE_FALLBACK_ENV}=true to allow plaintext fallback token storage."
                        ),
                    });
                }
                warn!(
                    "Keyring unavailable, using insecure file fallback token storage: {keyring_err}"
                );
            }
        }
    } else if !file_fallback_allowed() {
        return Err(ChoSdkError::Config {
            message: format!(
                "Keyring storage is disabled and plaintext fallback is blocked. \
Set {FILE_FALLBACK_ENV}=true to allow plaintext fallback token storage."
            ),
        });
    }

    store_to_file(tokens)
}

/// Clears stored tokens from keyring and file fallback.
pub fn clear_tokens() -> Result<()> {
    let mut errors: Vec<String> = Vec::new();

    if !keyring_disabled() {
        if let Err(err) = clear_keyring() {
            errors.push(err.to_string());
        }
    }

    if let Err(err) = clear_file() {
        errors.push(err.to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ChoSdkError::Config {
            message: format!(
                "Failed clearing stored authentication tokens: {}",
                errors.join("; ")
            ),
        })
    }
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

    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(ChoSdkError::Config {
            message: format!("Failed deleting keyring token entry: {e}"),
        }),
    }
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

fn keyring_disabled() -> bool {
    parse_truthy_env(std::env::var("CHO_DISABLE_KEYRING").ok())
}

fn file_fallback_allowed() -> bool {
    parse_truthy_env(std::env::var(FILE_FALLBACK_ENV).ok())
}

fn parse_truthy_env(value: Option<String>) -> bool {
    matches!(
        value.map(|value| value.trim().to_ascii_lowercase()),
        Some(value) if matches!(value.as_str(), "1" | "true" | "yes" | "on")
    )
}

#[cfg(test)]
mod tests {
    use super::parse_truthy_env;

    #[test]
    fn parse_truthy_env_defaults_to_false() {
        assert!(!parse_truthy_env(None));
        assert!(!parse_truthy_env(Some("false".to_string())));
    }

    #[test]
    fn parse_truthy_env_accepts_expected_values() {
        assert!(parse_truthy_env(Some("true".to_string())));
        assert!(parse_truthy_env(Some("1".to_string())));
        assert!(parse_truthy_env(Some("yes".to_string())));
        assert!(parse_truthy_env(Some("on".to_string())));
    }
}
