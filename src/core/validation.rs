use crate::core::error::{ConfigError, Result};

/// Returns `true` when `value` is safe to use as a single file name.
///
/// This intentionally rejects path separators and drive separators regardless
/// of the host OS so tests behave consistently on Windows, macOS, and Unix.
pub fn is_plain_file_name(value: &str) -> bool {
    is_safe_path_component(value)
}

/// Returns `true` when `value` is safe to append as one path component.
pub fn is_safe_path_component(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && !value.contains('/')
        && !value.contains('\\')
        && !value.contains(':')
        && !value.chars().any(char::is_control)
}

pub fn validate_plain_file_name(value: &str) -> Result<&str> {
    if is_plain_file_name(value) {
        Ok(value)
    } else {
        Err(ConfigError::InvalidPathComponent(value.to_string()))
    }
}

pub fn validate_path_component(value: &str) -> Result<&str> {
    if is_safe_path_component(value) {
        Ok(value)
    } else {
        Err(ConfigError::InvalidPathComponent(value.to_string()))
    }
}
