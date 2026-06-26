use serde::{Serialize, de::DeserializeOwned};

use crate::core::error::{ConfigError, Result};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum JsonFormat {
    Compact,
    Pretty,
}

pub fn serialize<T>(value: &T, format: JsonFormat) -> Result<String>
where
    T: Serialize,
{
    match format {
        JsonFormat::Compact => serde_json::to_string(value),
        JsonFormat::Pretty => serde_json::to_string_pretty(value),
    }
    .map_err(ConfigError::Serialize)
}

pub fn deserialize<T>(content: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    serde_json::from_str(content).map_err(ConfigError::Deserialize)
}
