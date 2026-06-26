use std::fmt;
use std::io;

/// Error type returned by `app-json-settings` operations.
#[derive(Debug)]
pub enum ConfigError {
    /// File-system or stream I/O failed.
    Io(io::Error),
    /// JSON serialization failed while saving a configuration value.
    Serialize(serde_json::Error),
    /// JSON deserialization failed while loading a configuration value.
    Deserialize(serde_json::Error),
    /// A caller-supplied file name or path component is unsafe.
    InvalidPathComponent(String),
    /// A platform-specific storage resolver failed.
    Platform(String),
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "I/O error: {e}"),
            ConfigError::Serialize(e) => write!(f, "JSON serialization error: {e}"),
            ConfigError::Deserialize(e) => write!(f, "JSON deserialization error: {e}"),
            ConfigError::InvalidPathComponent(value) => {
                write!(f, "invalid path component: {value:?}")
            }
            ConfigError::Platform(e) => write!(f, "platform error: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::Serialize(e) | ConfigError::Deserialize(e) => Some(e),
            ConfigError::InvalidPathComponent(_) | ConfigError::Platform(_) => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, ConfigError>;
