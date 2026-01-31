//! App JSON Settings

use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::fs::{read_to_string, remove_dir, remove_file, File};
use std::io::Write;
use std::path::PathBuf;

use dir::{config_dir_filepath, exe_dir_filepath, json_load};

mod dir;
#[cfg(test)]
mod tests;

/// default file name
const DEFAULT_FILENAME: &str = "settings.json";

/// core
#[derive(Serialize, Deserialize)]
pub struct JsonSettings {
    filepath: PathBuf,
}

/// i/o
#[derive(Serialize, Deserialize)]
pub struct KeyValue {
    key: Option<String>,
    pub value: Option<Value>,
    file_exists: bool,
    key_exists: bool,
}

impl JsonSettings {
    /// create instance
    pub fn new(filepath: &PathBuf) -> JsonSettings {
        JsonSettings {
            filepath: filepath.to_owned(),
        }
    }

    /// create instance to manage file in executable dir
    pub fn exe_dir() -> JsonSettings {
        let filepath = exe_dir_filepath(DEFAULT_FILENAME);
        JsonSettings::new(&filepath)
    }

    /// create instance to manage file in executable dir
    pub fn exe_dir_with_filename(filename: &str) -> JsonSettings {
        let filepath = exe_dir_filepath(filename);
        JsonSettings::new(&filepath)
    }

    /// create instance to manage file in user config dir
    pub fn config_dir() -> JsonSettings {
        let filepath = config_dir_filepath(DEFAULT_FILENAME);
        JsonSettings::new(&filepath)
    }

    /// create instance to manage file in user config dir
    pub fn config_dir_with_filename(filename: &str) -> JsonSettings {
        let filepath = config_dir_filepath(filename);
        JsonSettings::new(&filepath)
    }

    /// get value from key if exists
    pub fn read_by_key(&self, key: &str) -> Result<KeyValue, Box<dyn std::error::Error>> {
        let filepath = &self.filepath;

        if !filepath.exists() {
            return Ok(KeyValue {
                key: None,
                value: None,
                file_exists: false,
                key_exists: false,
            });
        }

        let json = json_load(&filepath)?;
        if let Some(value) = json.get(key) {
            Ok(KeyValue {
                key: Some(key.to_owned()),
                value: Some(value.to_owned()),
                file_exists: true,
                key_exists: true,
            })
        } else {
            return Ok(KeyValue {
                key: Some(key.to_owned()),
                value: None,
                file_exists: true,
                key_exists: false,
            });
        }
    }

    /// append or update value bound to key
    pub fn write_by_key(&self, key: &str, value: &Value) -> Result<(), std::io::Error> {
        let filepath = &self.filepath;

        let mut current_settings = if filepath.exists() {
            let file_text = read_to_string(&filepath)?;
            serde_json::from_str(&file_text).unwrap_or_default()
        } else {
            Value::Object(serde_json::Map::new())
        };

        let map = current_settings.as_object_mut().unwrap();
        map.insert(key.to_owned(), value.to_owned());

        let updated_settings = serde_json::to_string_pretty(&current_settings)?;

        let mut file = File::create(&filepath)?;
        file.write_all(updated_settings.as_bytes())?;

        Ok(())
    }

    /// remove settings file
    pub fn remove(&self, remove_dir_if_empty: bool) {
        remove_file(&self.filepath).expect("Failed to remove settings file");
        if !remove_dir_if_empty {
            match remove_dir(self.filepath.parent().unwrap()) {
                Ok(_) => (),
                Err(_) => (), // dir is not empty
            }
        }
    }
}
