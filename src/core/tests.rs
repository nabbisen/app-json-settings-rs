use super::ConfigManager;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
struct TestSettings {
    volume: u32,
    enabled: bool,
}

fn temp_dir(name: &str) -> PathBuf {
    let unique = format!(
        "app-json-settings-{name}-{}-{}-{}",
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
fn with_root_dir_controls_settings_location() {
    let dir = temp_dir("root-dir");
    let manager = ConfigManager::<TestSettings>::new()
        .with_root_dir(&dir)
        .with_filename("user.json");

    assert_eq!(manager.folder_path(), dir.as_path());
    assert_eq!(manager.path(), dir.join("user.json"));
}

#[test]
fn at_custom_dir_remains_backward_compatible_alias() {
    let dir = temp_dir("custom-dir");
    let manager = ConfigManager::<TestSettings>::new().at_custom_dir(&dir);

    assert_eq!(manager.folder_path(), dir.as_path());
}

#[test]
fn save_and_load_round_trip_pretty_json() {
    let dir = temp_dir("round-trip");
    let manager = ConfigManager::<TestSettings>::new().with_root_dir(&dir);

    let expected = TestSettings {
        volume: 42,
        enabled: true,
    };

    manager.save(&expected).expect("save should succeed");

    let raw = fs::read_to_string(manager.path()).expect("settings file should be readable");
    assert!(raw.contains('\n'));

    let actual = manager.load().expect("load should succeed");
    assert_eq!(actual, expected);
}

#[test]
fn compact_json_can_be_selected() {
    let dir = temp_dir("compact");
    let manager = ConfigManager::<TestSettings>::new()
        .with_root_dir(&dir)
        .disable_pretty_json();

    manager
        .save(&TestSettings {
            volume: 7,
            enabled: true,
        })
        .expect("save should succeed");

    let raw = fs::read_to_string(manager.path()).expect("settings file should be readable");
    assert_eq!(raw, r#"{"volume":7,"enabled":true}"#);
}

#[test]
fn load_or_default_creates_missing_file() {
    let dir = temp_dir("default");
    let manager = ConfigManager::<TestSettings>::new().with_root_dir(&dir);

    let settings = manager
        .load_or_default()
        .expect("load_or_default should create default settings");

    assert_eq!(settings, TestSettings::default());
    assert!(manager.path().exists());
}

#[test]
fn update_modifies_and_persists_configuration() {
    let dir = temp_dir("update");
    let manager = ConfigManager::<TestSettings>::new().with_root_dir(&dir);

    let updated = manager
        .update(|settings| {
            settings.volume = 100;
            settings.enabled = true;
        })
        .expect("update should succeed");

    assert_eq!(
        updated,
        TestSettings {
            volume: 100,
            enabled: true,
        }
    );

    let loaded = manager.load().expect("saved settings should load");
    assert_eq!(loaded, updated);
}
