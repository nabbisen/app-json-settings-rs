use super::ConfigManager;
use super::validation::{is_plain_file_name, is_safe_path_component};

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
        .try_with_filename("user.json")
        .expect("file name should be valid");

    assert_eq!(manager.folder_path(), dir.as_path());
    assert_eq!(manager.file_name(), "user.json");
    assert_eq!(manager.path(), dir.join("user.json"));
}

#[test]
fn at_custom_dir_remains_backward_compatible_alias() {
    let dir = temp_dir("custom-dir");
    let manager = ConfigManager::<TestSettings>::new().at_custom_dir(&dir);

    assert_eq!(manager.folder_path(), dir.as_path());
}

#[test]
fn for_app_uses_explicit_app_identity() {
    let manager = ConfigManager::<TestSettings>::for_app("app-json-settings-test")
        .expect("app name should be valid");

    assert!(manager.folder_path().ends_with("app-json-settings-test"));
}

#[test]
fn new_does_not_panic() {
    // new() cannot report platform-resolution failure without an API break,
    // so it must fall back rather than panic even where resolution would
    // fail. Resolution failure itself is exercised directly against the
    // private seam in `core::dir::tests`, without mutating the real
    // environment; this test only proves the public constructor stays
    // infallible.
    let _ = ConfigManager::<TestSettings>::new();
}

#[test]
fn try_new_succeeds_in_normal_test_environment() {
    let manager = ConfigManager::<TestSettings>::try_new()
        .expect("try_new should succeed when both resolution steps succeed");
    assert!(!manager.folder_path().as_os_str().is_empty());
}

#[test]
fn new_still_falls_back_rather_than_erroring() {
    // new() cannot be forced through its failure branches without mutating
    // the real environment (unsafe in this edition, and would race the
    // parallel test harness) -- the failure branches themselves are
    // exercised directly against the private seam in `core::dir::tests`.
    // This is the regression guard that is actually achievable here: it
    // proves new()'s folder_path is still composed from the exact same
    // fallback-applying expression, computed independently, so a future
    // drift between new() and its underlying derivation would show up as a
    // mismatch rather than passing silently.
    let expected_folder = crate::core::dir::default_config_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(crate::core::dir::default_runtime_app_name());

    let manager = ConfigManager::<TestSettings>::new();
    assert_eq!(manager.folder_path(), expected_folder);
}

#[test]
fn with_root_dir_works_independently_of_platform_resolution() {
    // with_root_dir() overwrites the folder path unconditionally, so it must
    // keep working as the documented escape hatch even in an environment
    // where platform resolution (HOME / %APPDATA%) would fail. This proves
    // the public escape hatch end to end; resolution failure itself is
    // proven directly against the private seam in `core::dir::tests`.
    let dir = temp_dir("root-dir-independent-of-resolution");
    let manager = ConfigManager::<TestSettings>::new()
        .with_root_dir(&dir)
        .try_with_filename("settings.json")
        .expect("file name should be valid");

    manager
        .save(&TestSettings {
            volume: 1,
            enabled: true,
        })
        .expect("save should succeed regardless of platform resolution");

    let loaded = manager.load().expect("load should succeed");
    assert_eq!(
        loaded,
        TestSettings {
            volume: 1,
            enabled: true,
        }
    );
}

#[test]
fn safe_file_name_validation_rejects_paths() {
    assert!(is_plain_file_name("settings.json"));
    assert!(is_safe_path_component("app-json-settings"));

    for value in [
        "",
        ".",
        "..",
        "../settings.json",
        "a/b.json",
        r"a\b.json",
        "C:settings.json",
    ] {
        assert!(!is_plain_file_name(value), "{value:?} should be invalid");
        assert!(
            ConfigManager::<TestSettings>::new()
                .try_with_filename(value)
                .is_err(),
            "{value:?} should be rejected by try_with_filename"
        );
    }
}

#[test]
fn for_app_rejects_reserved_device_names() {
    let error = ConfigManager::<TestSettings>::for_app("CON")
        .expect_err("CON should be rejected as an app name");
    assert!(matches!(error, crate::ConfigError::InvalidPathComponent(_)));
}

#[test]
fn try_with_filename_rejects_reserved_device_names() {
    let error = ConfigManager::<TestSettings>::new()
        .try_with_filename("NUL")
        .expect_err("NUL should be rejected as a file name");
    assert!(matches!(error, crate::ConfigError::InvalidPathComponent(_)));
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
fn load_reports_invalid_json_as_deserialization_error() {
    let dir = temp_dir("invalid-json");
    let manager = ConfigManager::<TestSettings>::new().with_root_dir(&dir);

    fs::create_dir_all(&dir).expect("test directory should be created");
    fs::write(manager.path(), "not-json").expect("invalid settings file should be written");

    let error = manager.load().expect_err("invalid JSON should fail");
    assert!(matches!(error, crate::ConfigError::Deserialize(_)));
}

#[test]
fn load_or_default_does_not_reset_an_existing_invalid_file() {
    // load_or_default() creates defaults only when the file is absent
    // (NotFound). An existing file that fails to parse must return
    // Deserialize, not silently reset to defaults -- and the file on disk
    // must be untouched, which is what actually proves no reset happened.
    let dir = temp_dir("invalid-json-no-reset");
    let manager = ConfigManager::<TestSettings>::new().with_root_dir(&dir);

    fs::create_dir_all(&dir).expect("test directory should be created");
    let invalid_content = "not-json";
    fs::write(manager.path(), invalid_content).expect("invalid settings file should be written");

    let error = manager
        .load_or_default()
        .expect_err("load_or_default should not silently reset an existing invalid file");
    assert!(matches!(error, crate::ConfigError::Deserialize(_)));

    let content_after =
        fs::read_to_string(manager.path()).expect("settings file should still be readable");
    assert_eq!(
        content_after, invalid_content,
        "the invalid file must be left exactly as it was, not reset"
    );
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

#[test]
fn default_save_mode_is_atomic() {
    let manager = ConfigManager::<TestSettings>::new();
    assert_eq!(manager.save_mode(), super::SaveMode::Atomic);
}

#[test]
fn direct_save_mode_can_be_selected() {
    let manager = ConfigManager::<TestSettings>::new().with_direct_save();
    assert_eq!(manager.save_mode(), super::SaveMode::Direct);
}

#[test]
fn explicit_save_mode_can_be_selected() {
    let manager = ConfigManager::<TestSettings>::new().with_save_mode(super::SaveMode::Direct);
    assert_eq!(manager.save_mode(), super::SaveMode::Direct);
}

#[test]
fn direct_save_writes_settings_file() {
    let dir = temp_dir("direct-save");
    let manager = ConfigManager::<TestSettings>::new()
        .with_root_dir(&dir)
        .with_direct_save();

    manager
        .save(&TestSettings {
            volume: 9,
            enabled: false,
        })
        .expect("direct save should succeed");

    let loaded = manager.load().expect("direct-saved settings should load");
    assert_eq!(
        loaded,
        TestSettings {
            volume: 9,
            enabled: false,
        }
    );
}

#[test]
fn atomic_save_does_not_leave_temp_file_after_success() {
    let dir = temp_dir("atomic-temp-cleanup");
    let manager = ConfigManager::<TestSettings>::new().with_root_dir(&dir);

    manager
        .save(&TestSettings {
            volume: 11,
            enabled: true,
        })
        .expect("atomic save should succeed");

    let entries = fs::read_dir(&dir).expect("settings directory should exist");
    let temp_files: Vec<_> = entries
        .map(|entry| entry.expect("directory entry should be readable"))
        .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp."))
        .collect();

    assert!(
        temp_files.is_empty(),
        "temporary files should be cleaned up"
    );
}

#[cfg(unix)]
#[test]
fn atomic_save_preserves_existing_mode_0600() {
    use std::os::unix::fs::PermissionsExt;

    let dir = temp_dir("mode-preserve-0600");
    let manager = ConfigManager::<TestSettings>::new().with_root_dir(&dir);

    manager
        .save(&TestSettings {
            volume: 1,
            enabled: false,
        })
        .expect("initial save should succeed");
    fs::set_permissions(manager.path(), fs::Permissions::from_mode(0o600))
        .expect("mode should be settable");

    manager
        .save(&TestSettings {
            volume: 2,
            enabled: true,
        })
        .expect("second save should succeed");

    let mode = fs::metadata(manager.path())
        .expect("settings file metadata should be readable")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600, "mode was {mode:o}, expected preserved 0o600");
}

#[cfg(unix)]
#[test]
fn atomic_save_preserves_existing_mode_0644() {
    use std::os::unix::fs::PermissionsExt;

    let dir = temp_dir("mode-preserve-0644");
    let manager = ConfigManager::<TestSettings>::new().with_root_dir(&dir);

    manager
        .save(&TestSettings {
            volume: 1,
            enabled: false,
        })
        .expect("initial save should succeed");
    fs::set_permissions(manager.path(), fs::Permissions::from_mode(0o644))
        .expect("mode should be settable");

    manager
        .save(&TestSettings {
            volume: 2,
            enabled: true,
        })
        .expect("second save should succeed");

    let mode = fs::metadata(manager.path())
        .expect("settings file metadata should be readable")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o644, "mode was {mode:o}, expected preserved 0o644");
}

#[cfg(unix)]
#[test]
fn atomic_save_creates_new_file_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let dir = temp_dir("mode-new-file");
    let manager = ConfigManager::<TestSettings>::new().with_root_dir(&dir);

    manager
        .save(&TestSettings {
            volume: 3,
            enabled: true,
        })
        .expect("save should succeed");

    let mode = fs::metadata(manager.path())
        .expect("settings file metadata should be readable")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600, "new file mode was {mode:o}, expected 0o600");
}

#[cfg(unix)]
#[test]
fn direct_save_mode_behavior_is_unchanged() {
    use std::os::unix::fs::PermissionsExt;

    let dir = temp_dir("mode-direct-save");
    let manager = ConfigManager::<TestSettings>::new()
        .with_root_dir(&dir)
        .with_direct_save();

    manager
        .save(&TestSettings {
            volume: 1,
            enabled: false,
        })
        .expect("initial save should succeed");
    fs::set_permissions(manager.path(), fs::Permissions::from_mode(0o600))
        .expect("mode should be settable");

    manager
        .save(&TestSettings {
            volume: 2,
            enabled: true,
        })
        .expect("second save should succeed");

    let mode = fs::metadata(manager.path())
        .expect("settings file metadata should be readable")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        mode, 0o600,
        "direct save should still preserve existing mode; was {mode:o}"
    );
}

#[derive(Debug, Deserialize)]
struct FailingSerializeSettings;

impl Serialize for FailingSerializeSettings {
    fn serialize<S>(&self, _serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom(
            "intentional serialization failure",
        ))
    }
}

#[test]
fn failed_serialization_preserves_existing_file() {
    let dir = temp_dir("serialization-failure-preserves-existing");
    let good_manager = ConfigManager::<TestSettings>::new().with_root_dir(&dir);

    good_manager
        .save(&TestSettings {
            volume: 5,
            enabled: true,
        })
        .expect("initial save should succeed");

    let before = fs::read_to_string(good_manager.path()).expect("existing file should be readable");

    let failing_manager = ConfigManager::<FailingSerializeSettings>::new().with_root_dir(&dir);
    let error = failing_manager
        .save(&FailingSerializeSettings)
        .expect_err("serialization should fail before touching storage");

    assert!(matches!(error, crate::ConfigError::Serialize(_)));
    let after =
        fs::read_to_string(good_manager.path()).expect("existing file should remain readable");
    assert_eq!(after, before);
}
