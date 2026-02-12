//! Shared utilities for CLI command implementations.

/// Maximum JSON file size (50 MB).
const MAX_JSON_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Reads and parses a JSON file into the specified type.
///
/// Rejects files larger than 50 MB to prevent OOM on malformed input.
pub fn read_json_file<T: serde::de::DeserializeOwned>(
    path: &std::path::Path,
) -> cho_sdk::error::Result<T> {
    let metadata = std::fs::metadata(path).map_err(|e| cho_sdk::error::ChoSdkError::Config {
        message: format!("Failed to read file {}: {e}", path.display()),
    })?;

    if metadata.len() > MAX_JSON_FILE_SIZE {
        return Err(cho_sdk::error::ChoSdkError::Config {
            message: format!(
                "File {} is too large ({} bytes, max {} bytes)",
                path.display(),
                metadata.len(),
                MAX_JSON_FILE_SIZE,
            ),
        });
    }

    let content =
        std::fs::read_to_string(path).map_err(|e| cho_sdk::error::ChoSdkError::Config {
            message: format!("Failed to read file {}: {e}", path.display()),
        })?;
    serde_json::from_str(&content).map_err(|e| cho_sdk::error::ChoSdkError::Parse {
        message: format!("Failed to parse JSON from {}: {e}", path.display()),
    })
}

/// Validates that a string is a valid YYYY-MM-DD date.
///
/// Used to prevent OData injection via `--from`/`--to`/`--date` flags
/// that are interpolated into `DateTime()` expressions.
pub fn validate_date(value: &str, flag: &str) -> cho_sdk::error::Result<()> {
    // Quick structural check: exactly 10 chars, dashes at positions 4 and 7
    let bytes = value.as_bytes();
    let valid = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit());

    if !valid {
        return Err(cho_sdk::error::ChoSdkError::Config {
            message: format!("Invalid date format for {flag}: \"{value}\". Expected YYYY-MM-DD."),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_date_valid() {
        assert!(validate_date("2024-01-01", "--from").is_ok());
        assert!(validate_date("2024-12-31", "--to").is_ok());
        assert!(validate_date("1999-06-15", "--date").is_ok());
    }

    #[test]
    fn validate_date_invalid() {
        assert!(validate_date("not-a-date", "--from").is_err());
        assert!(validate_date("2024/01/01", "--from").is_err());
        assert!(validate_date("2024-1-1", "--from").is_err());
        assert!(validate_date("", "--from").is_err());
    }

    #[test]
    fn validate_date_injection_attempts() {
        assert!(validate_date("2024-01-01') OR 1=1--", "--from").is_err());
        assert!(validate_date("2024-01-01;DROP", "--from").is_err());
        assert!(validate_date("${evil}", "--from").is_err());
    }

    #[test]
    fn read_json_file_missing() {
        let result = read_json_file::<serde_json::Value>(std::path::Path::new("/nonexistent"));
        assert!(result.is_err());
    }
}
