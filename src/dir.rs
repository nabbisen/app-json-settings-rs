use serde_json::{from_str, Value};
use std::fs::{create_dir_all, File};
use std::io::Read;
use std::path::PathBuf;

/// user config dir
pub fn config_dir() -> PathBuf {
    let current_exe = std::env::current_exe().unwrap();
    let filename = current_exe.file_name().unwrap().to_str().unwrap();
    config_root_dir().join(filename)
}

/// settings file path in executable dir
pub fn exe_dir_filepath(filename: &str) -> PathBuf {
    let exec_filepath = std::env::current_exe().expect("Failed to get exec path");
    let dirpath = exec_filepath
        .parent()
        .expect("Failed to get exec parent dir path");
    dirpath.join(filename)
}

/// settings file path in config dir
pub fn config_dir_filepath(filename: &str) -> PathBuf {
    let dirpath = config_dir();
    if !dirpath.exists() {
        create_dir_all(&dirpath).expect("Failed to create app dir in user config");
    }
    dirpath.join(filename)
}

/// read settings file and get json key-value pairs
pub fn json_load(filepath: &PathBuf) -> Result<Value, Box<dyn std::error::Error>> {
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
pub fn config_root_dir() -> PathBuf {
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
