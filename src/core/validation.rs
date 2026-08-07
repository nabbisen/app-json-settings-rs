use crate::core::error::{ConfigError, Result};

/// Windows reserved device names. Windows treats these as device names in
/// any directory, at any case, and for the stem of any file name -- so
/// `NUL.txt` refers to the null device just as `NUL` does.
const RESERVED_DEVICE_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Returns `true` when `value`'s stem (the portion before the first `.`)
/// case-insensitively matches a Windows reserved device name.
///
/// Checked on every platform, not just Windows, so that the same value is
/// either accepted or rejected regardless of host OS -- see
/// [`is_safe_path_component`]'s documentation for why that consistency
/// matters here.
fn is_reserved_device_name(value: &str) -> bool {
    let stem = value.split('.').next().unwrap_or(value);
    RESERVED_DEVICE_NAMES
        .iter()
        .any(|reserved| stem.eq_ignore_ascii_case(reserved))
}

/// Returns `true` when `value` is safe to use as a single file name.
///
/// This intentionally rejects path separators and drive separators regardless
/// of the host OS so tests behave consistently on Windows, macOS, and Unix.
pub fn is_plain_file_name(value: &str) -> bool {
    is_safe_path_component(value)
}

/// Returns `true` when `value` is safe to append as one path component.
///
/// This also rejects Windows reserved device names (`CON`, `PRN`, `AUX`,
/// `NUL`, `COM1`-`COM9`, `LPT1`-`LPT9`, case-insensitively, including with
/// an extension such as `NUL.txt`) on every platform, not just Windows --
/// consistent with this function's existing OS-independent behavior. On
/// Windows these names refer to devices rather than files in any directory,
/// so a value that passes here but is later used as a file name would fail,
/// or silently target a device, only there.
pub fn is_safe_path_component(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && !value.contains('/')
        && !value.contains('\\')
        && !value.contains(':')
        && !value.chars().any(char::is_control)
        && !is_reserved_device_name(value)
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

#[cfg(test)]
mod tests;
