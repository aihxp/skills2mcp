use crate::error::{Result, SxmcError};

/// Resolve a secret value that may use env: or file: prefixes.
///
/// Supported formats:
/// - `env:VAR_NAME` — read from environment variable
/// - `file:/path/to/secret` — read from file (trimmed)
/// - anything else — returned as-is (literal value)
pub fn resolve_secret(value: &str) -> Result<String> {
    if let Some(var_name) = value.strip_prefix("env:") {
        std::env::var(var_name)
            .map_err(|_| SxmcError::Other(format!("Environment variable '{}' not set", var_name)))
    } else if let Some(path) = value.strip_prefix("file:") {
        std::fs::read_to_string(path)
            .map(|s| s.trim().to_string())
            .map_err(|e| SxmcError::Other(format!("Failed to read secret file '{}': {}", path, e)))
    } else {
        Ok(value.to_string())
    }
}

/// Resolve a header value, applying secret resolution to the value part.
/// Input format: "Key:Value" where Value can use env:/file: prefixes.
pub fn resolve_header(header: &str) -> Result<(String, String)> {
    let (key, value) = header.split_once(':').ok_or_else(|| {
        SxmcError::Other(format!(
            "Invalid header format '{}' — expected Key:Value",
            header
        ))
    })?;

    let resolved = resolve_secret(value.trim())?;
    Ok((key.trim().to_string(), resolved))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_literal() {
        assert_eq!(resolve_secret("my-token-123").unwrap(), "my-token-123");
    }

    #[test]
    fn test_resolve_env() {
        std::env::set_var("SXMC_TEST_SECRET", "secret_value_42");
        assert_eq!(
            resolve_secret("env:SXMC_TEST_SECRET").unwrap(),
            "secret_value_42"
        );
        std::env::remove_var("SXMC_TEST_SECRET");
    }

    #[test]
    fn test_resolve_env_missing() {
        assert!(resolve_secret("env:SXMC_NONEXISTENT_VAR_99999").is_err());
    }

    #[test]
    fn test_resolve_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secret.txt");
        std::fs::write(&path, "file_secret\n").unwrap();
        assert_eq!(
            resolve_secret(&format!("file:{}", path.display())).unwrap(),
            "file_secret"
        );
    }

    #[test]
    fn test_resolve_header() {
        let (key, val) = resolve_header("Authorization: Bearer token123").unwrap();
        assert_eq!(key, "Authorization");
        assert_eq!(val, "Bearer token123");
    }

    #[test]
    fn test_resolve_header_with_env() {
        std::env::set_var("SXMC_TEST_TOKEN", "bearer_abc");
        let (key, val) = resolve_header("Authorization: env:SXMC_TEST_TOKEN").unwrap();
        assert_eq!(key, "Authorization");
        assert_eq!(val, "bearer_abc");
        std::env::remove_var("SXMC_TEST_TOKEN");
    }
}
