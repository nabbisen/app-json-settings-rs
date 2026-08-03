use super::*;

/// Builds a stub environment lookup from fixed key/value pairs. No real
/// environment variables are read or mutated, so these tests are safe to run
/// in parallel with the rest of the suite.
fn getenv_from(
    pairs: &'static [(&'static str, &'static str)],
) -> impl Fn(&str) -> Option<OsString> {
    move |name| {
        pairs
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| OsString::from(*value))
    }
}

fn getenv_empty(_name: &str) -> Option<OsString> {
    None
}

fn assert_platform_error_mentions(error: ConfigError, needles: &[&str]) {
    match error {
        ConfigError::Platform(message) => {
            for needle in needles {
                assert!(
                    message.contains(needle),
                    "error message {message:?} should mention {needle:?}"
                );
            }
        }
        other => panic!("expected ConfigError::Platform, got {other:?}"),
    }
}

#[cfg(target_os = "windows")]
mod windows_resolution {
    use super::*;

    #[test]
    fn appdata_set_is_used() {
        let dir = config_dir_from(getenv_from(&[(
            "APPDATA",
            r"C:\Users\demo\AppData\Roaming",
        )]))
        .expect("resolution should succeed");
        assert_eq!(dir, PathBuf::from(r"C:\Users\demo\AppData\Roaming"));
    }

    #[test]
    fn appdata_unset_is_an_error() {
        let error = config_dir_from(getenv_empty).expect_err("resolution should fail");
        assert_platform_error_mentions(error, &["APPDATA", "with_root_dir"]);
    }
}

#[cfg(target_os = "macos")]
mod macos_resolution {
    use super::*;

    #[test]
    fn home_set_is_used() {
        let dir = config_dir_from(getenv_from(&[("HOME", "/Users/demo")]))
            .expect("resolution should succeed");
        assert_eq!(
            dir,
            PathBuf::from("/Users/demo/Library/Application Support")
        );
    }

    #[test]
    fn home_unset_is_an_error() {
        let error = config_dir_from(getenv_empty).expect_err("resolution should fail");
        assert_platform_error_mentions(error, &["HOME", "with_root_dir"]);
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod unix_resolution {
    use super::*;

    #[test]
    fn xdg_config_home_takes_priority() {
        let dir = config_dir_from(getenv_from(&[
            ("XDG_CONFIG_HOME", "/custom/config"),
            ("HOME", "/home/demo"),
        ]))
        .expect("resolution should succeed");
        assert_eq!(dir, PathBuf::from("/custom/config"));
    }

    #[test]
    fn falls_back_to_home_dot_config_when_xdg_unset() {
        let dir = config_dir_from(getenv_from(&[("HOME", "/home/demo")]))
            .expect("resolution should succeed");
        assert_eq!(dir, PathBuf::from("/home/demo/.config"));
    }

    #[test]
    fn both_unset_is_an_error() {
        let error = config_dir_from(getenv_empty).expect_err("resolution should fail");
        assert_platform_error_mentions(error, &["XDG_CONFIG_HOME", "HOME", "with_root_dir"]);
    }
}
