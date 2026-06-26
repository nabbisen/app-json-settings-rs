use std::path::PathBuf;

#[cfg(all(windows, feature = "uwp"))]
use crate::{ConfigError, Result};

pub fn default_config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    }

    #[cfg(target_os = "macos")]
    {
        let mut p = home_dir();
        p.push("Library");
        p.push("Application Support");
        p
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            PathBuf::from(xdg)
        } else {
            let mut p = home_dir();
            p.push(".config");
            p
        }
    }
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
    ConfigError::Platform(error.message().to_string_lossy())
}

#[cfg(any(target_os = "macos", all(unix, not(target_os = "macos"))))]
pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
