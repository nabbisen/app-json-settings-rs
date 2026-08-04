use std::ffi::OsString;
use std::path::PathBuf;

use crate::{ConfigError, Result};

/// Resolves the platform configuration base directory from an environment
/// lookup function.
///
/// This is a pure, testable seam: the public-facing [`default_config_dir`]
/// supplies the real environment via `std::env::var_os`. Tests supply a stub
/// instead of mutating process environment variables, which is `unsafe` in
/// this edition and would race the parallel test harness across the CI
/// matrix.
fn config_dir_from(getenv: impl Fn(&str) -> Option<OsString>) -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        getenv("APPDATA").map(PathBuf::from).ok_or_else(|| {
            ConfigError::Platform(
                "could not resolve the platform configuration directory: %APPDATA% is not \
                 set; use with_root_dir() to supply a path explicitly"
                    .to_string(),
            )
        })
    }

    #[cfg(target_os = "macos")]
    {
        getenv("HOME")
            .map(|home| {
                let mut p = PathBuf::from(home);
                p.push("Library");
                p.push("Application Support");
                p
            })
            .ok_or_else(|| {
                ConfigError::Platform(
                    "could not resolve the platform configuration directory: HOME is not \
                     set; use with_root_dir() to supply a path explicitly"
                        .to_string(),
                )
            })
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(xdg) = getenv("XDG_CONFIG_HOME") {
            Ok(PathBuf::from(xdg))
        } else if let Some(home) = getenv("HOME") {
            let mut p = PathBuf::from(home);
            p.push(".config");
            Ok(p)
        } else {
            Err(ConfigError::Platform(
                "could not resolve the platform configuration directory: neither \
                 XDG_CONFIG_HOME nor HOME is set; use with_root_dir() to supply a path \
                 explicitly"
                    .to_string(),
            ))
        }
    }
}

pub fn default_config_dir() -> Result<PathBuf> {
    config_dir_from(|name| std::env::var_os(name))
}

/// Derives an application name from an executable path.
///
/// This is a pure, testable seam, following [`config_dir_from`]'s pattern:
/// [`try_new`](crate::ConfigManager::try_new) and
/// [`default_runtime_app_name`] both call this with the real
/// `std::env::current_exe()` result, but a test can drive it directly with
/// a stub path or `None` -- `current_exe()` cannot be made to fail from
/// inside a test otherwise, so without this seam the failure path would be
/// untestable.
///
/// Fails if `exe` is `None` (the executable path could not be determined)
/// or if its file stem is not a safe path component (missing, or rejected
/// by [`is_safe_path_component`](crate::core::validation::is_safe_path_component)).
/// The two failures produce distinguishable messages.
pub fn app_name_from(exe: Option<PathBuf>) -> Result<String> {
    let exe = exe.ok_or_else(|| {
        ConfigError::Platform(
            "could not determine the current executable path, so no application name \
             could be derived; use for_app() to supply an explicit application name, \
             or try_new() to fail instead of falling back to \"app\""
                .to_string(),
        )
    })?;

    let stem = exe
        .file_stem()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();

    if crate::core::validation::is_safe_path_component(&stem) {
        Ok(stem)
    } else {
        Err(ConfigError::Platform(format!(
            "the current executable's name is not a safe application name ({stem:?}); \
             use for_app() to supply an explicit application name, or try_new() to fail \
             instead of falling back to \"app\""
        )))
    }
}

/// Derives an application name from the current executable, falling back to
/// the literal name `"app"` if it cannot be determined or is unsafe.
///
/// **This fallback is a fixed constant.** Any two executables that both hit
/// it resolve to the same name, and therefore the same settings file under
/// [`ConfigManager::new`](crate::ConfigManager::new) -- one can silently
/// read and overwrite the other's settings. Prefer
/// [`try_new`](crate::ConfigManager::try_new) to fail instead of falling
/// back, or [`for_app`](crate::ConfigManager::for_app) to supply an
/// explicit name and avoid derivation entirely.
pub fn default_runtime_app_name() -> String {
    app_name_from(std::env::current_exe().ok()).unwrap_or_else(|_| "app".to_string())
}

#[cfg(all(windows, feature = "uwp"))]
pub fn uwp_local_folder_dir() -> Result<PathBuf> {
    use windows::Storage::ApplicationData;

    let application_data = ApplicationData::Current().map_err(platform_error)?;
    let local_folder = application_data.LocalFolder().map_err(platform_error)?;
    let path = local_folder.Path().map_err(platform_error)?;

    Ok(PathBuf::from(path.to_string_lossy()))
}

#[cfg(all(windows, feature = "uwp"))]
fn platform_error(error: windows::core::Error) -> ConfigError {
    ConfigError::Platform(error.message())
}

#[cfg(test)]
mod tests;
