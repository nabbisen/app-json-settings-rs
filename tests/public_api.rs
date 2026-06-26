use app_json_settings::{ConfigError, ConfigManager, DEFAULT_FILE_NAME, SaveMode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
struct Settings {
    theme: String,
    launch_count: u32,
}

fn temp_dir(name: &str) -> PathBuf {
    let unique = format!(
        "app-json-settings-public-api-{name}-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX epoch")
            .as_nanos(),
        TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
    );

    std::env::temp_dir().join(unique)
}

#[test]
fn default_file_name_is_public_and_stable() {
    assert_eq!(DEFAULT_FILE_NAME, "settings.json");
}

#[test]
fn explicit_root_and_checked_file_name_are_public_api() {
    let dir = temp_dir("checked-file-name");
    let manager = ConfigManager::<Settings>::new()
        .with_root_dir(&dir)
        .try_with_filename("user-settings.json")
        .expect("file name should be valid");

    assert_eq!(manager.folder_path(), dir.as_path());
    assert_eq!(manager.file_name(), "user-settings.json");
    assert_eq!(manager.path(), dir.join("user-settings.json"));
}

#[test]
fn invalid_checked_file_name_returns_error() {
    let error = ConfigManager::<Settings>::new()
        .try_with_filename("../settings.json")
        .expect_err("path-like file name should be rejected");

    assert!(matches!(error, ConfigError::InvalidPathComponent(_)));
}

#[test]
fn public_round_trip_and_update_work() {
    let dir = temp_dir("round-trip");
    let manager = ConfigManager::<Settings>::new().with_root_dir(&dir);

    manager
        .save(&Settings {
            theme: "light".to_string(),
            launch_count: 1,
        })
        .expect("save should succeed");

    let loaded = manager.load().expect("load should succeed");
    assert_eq!(loaded.theme, "light");
    assert_eq!(loaded.launch_count, 1);

    let updated = manager
        .update(|settings| {
            settings.theme = "dark".to_string();
            settings.launch_count += 1;
        })
        .expect("update should succeed");

    assert_eq!(updated.theme, "dark");
    assert_eq!(updated.launch_count, 2);
}

#[test]
fn compact_json_is_public_api() {
    let dir = temp_dir("compact");
    let manager = ConfigManager::<Settings>::new()
        .with_root_dir(&dir)
        .disable_pretty_json();

    manager
        .save(&Settings {
            theme: "dark".to_string(),
            launch_count: 3,
        })
        .expect("save should succeed");

    let raw = fs::read_to_string(manager.path()).expect("settings file should be readable");
    assert_eq!(raw, r#"{"theme":"dark","launch_count":3}"#);
}

#[test]
fn save_mode_is_public_api() {
    let manager = ConfigManager::<Settings>::new().with_save_mode(SaveMode::Direct);

    assert_eq!(manager.save_mode(), SaveMode::Direct);
    assert_eq!(
        ConfigManager::<Settings>::new().save_mode(),
        SaveMode::Atomic
    );
}
