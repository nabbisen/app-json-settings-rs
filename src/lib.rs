//! App JSON Settings

use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::fs::{create_dir_all, read_to_string, remove_dir, remove_file, File};
use std::io::{Read, Write};
use std::path::PathBuf;

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

/// user config dir
pub fn config_dir() -> PathBuf {
    let current_exe = std::env::current_exe().unwrap();
    let filename = current_exe.file_name().unwrap().to_str().unwrap();
    config_root_dir().join(filename)
}

/// settings file path in executable dir
fn exe_dir_filepath(filename: &str) -> PathBuf {
    let exec_filepath = std::env::current_exe().expect("Failed to get exec path");
    let dirpath = exec_filepath
        .parent()
        .expect("Failed to get exec parent dir path");
    dirpath.join(filename)
}

/// settings file path in config dir
fn config_dir_filepath(filename: &str) -> PathBuf {
    let dirpath = config_dir();
    if !dirpath.exists() {
        create_dir_all(&dirpath).expect("Failed to create app dir in user config");
    }
    dirpath.join(filename)
}

/// read settings file and get json key-value pairs
fn json_load(filepath: &PathBuf) -> Result<Value, Box<dyn std::error::Error>> {
    let mut file =
        File::open(&filepath).map_err(|e| format!("Failed to open settings file: {}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    let json: Value =
        from_str(&contents).map_err(|e| format!("Failed to deserialize settings: {}", e))?;
    Ok(json)
}

#[cfg(target_os = "linux")]
fn config_root_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut home_dir = std::env::var("HOME").expect("HOME not set");
            home_dir.push_str("/.config");
            PathBuf::from(home_dir)
        })
}

#[cfg(target_os = "windows")]
fn config_root_dir() -> PathBuf {
    std::env::var("APPDATA")
        .map(PathBuf::from)
        .expect("APPDATA not set")
}

#[cfg(target_os = "macos")]
fn config_root_dir() -> PathBuf {
    let mut home_dir = std::env::var("HOME").expect("HOME not set");
    home_dir.push_str("/Library/Application Support");
    PathBuf::from(home_dir)
}

#[cfg(target_os = "android")]
fn config_root_dir() -> PathBuf {
    let internal_storage =
        std::env::var("ANDROID_INTERNAL_STORAGE").expect("ANDROID_INTERNAL_STORAGE not set");
    PathBuf::from(internal_storage).join("config")
}

#[cfg(target_os = "ios")]
fn config_root_dir() -> PathBuf {
    let home_dir = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home_dir).join("Documents").join("config")
}
