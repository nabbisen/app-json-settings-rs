use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
    Platform(String),
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(e: serde_json::Error) -> Self {
        if e.is_data() || e.is_syntax() {
            ConfigError::Deserialize(e)
        } else {
            ConfigError::Serialize(e)
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "I/O error: {e}"),
            ConfigError::Serialize(e) => write!(f, "JSON serialization error: {e}"),
            ConfigError::Deserialize(e) => write!(f, "JSON deserialization error: {e}"),
            ConfigError::Platform(e) => write!(f, "platform error: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::Serialize(e) | ConfigError::Deserialize(e) => Some(e),
            ConfigError::Platform(_) => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, ConfigError>;
