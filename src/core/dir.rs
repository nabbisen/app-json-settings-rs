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

pub fn default_runtime_app_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_stem()
                .map(|name| name.to_string_lossy().to_string())
        })
        .filter(|name| crate::core::validation::is_safe_path_component(name))
        .unwrap_or_else(|| "app".to_string())
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
