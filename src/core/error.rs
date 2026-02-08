use std::io;

//
// Error
//
#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
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

pub type Result<T> = std::result::Result<T, ConfigError>;
