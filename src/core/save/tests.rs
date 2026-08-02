#![cfg(unix)]

use super::*;

use std::os::unix::fs::PermissionsExt;
use std::sync::atomic::AtomicUsize;

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_dir(name: &str) -> PathBuf {
    let unique = format!(
        "app-json-settings-save-tests-{name}-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX epoch")
            .as_nanos(),
        TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
    );

    std::env::temp_dir().join(unique)
}

/// `create_temp_file()` is private to this module, so its mode must be
/// asserted directly here rather than inferred from a fully saved file:
/// the final file also has `apply_target_mode` applied to it, which would
/// mask a wrong temp-file mode.
#[test]
fn create_temp_file_is_owner_only() {
    let dir = temp_dir("temp-file-mode");
    fs::create_dir_all(&dir).expect("test directory should be created");
    let target = dir.join("settings.json");

    let (temp_path, temp_file) = create_temp_file(&target).expect("temp file should be created");

    let mode = temp_file
        .metadata()
        .expect("temp file metadata should be readable")
        .permissions()
        .mode()
        & 0o777;

    assert_eq!(mode, 0o600, "temp file mode was {mode:o}, expected 0o600");

    drop(temp_file);
    let _ = fs::remove_file(&temp_path);
    let _ = fs::remove_dir_all(&dir);
}
