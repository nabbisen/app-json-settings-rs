use serde::{Serialize, de::DeserializeOwned};

use std::fs;
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::ConfigError;
use crate::Result;
use crate::core::constant::DEFAULT_FILE_NAME;
#[cfg(all(windows, feature = "uwp"))]
use crate::core::dir::uwp_local_folder_dir;
use crate::core::dir::{app_name_from, default_config_dir, default_runtime_app_name};
use crate::core::json::{JsonFormat, deserialize, serialize};
use crate::core::save::save_to_path;
use crate::core::validation::{validate_path_component, validate_plain_file_name};

pub mod constant;
mod dir;
pub mod error;
mod json;
mod save;
pub mod validation;

pub use save::SaveMode;

#[cfg(test)]
mod tests;

/// Manages one typed JSON settings file.
///
/// `ConfigManager<T>` stores and loads a complete configuration value of type
/// `T`. The type must implement Serde `Serialize` and `DeserializeOwned`.
///
/// The manager is intentionally small. It owns only:
///
/// * the directory containing the settings file,
/// * the settings file name,
/// * the JSON output format, and
/// * the save strategy.
#[derive(Debug, Clone)]
pub struct ConfigManager<T> {
    folder_path: PathBuf,
    file_name: String,
    json_format: JsonFormat,
    save_mode: SaveMode,
    _marker: PhantomData<T>,
}

impl<T> ConfigManager<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Creates a config manager using the OS-standard config directory and the
    /// current executable name.
    ///
    /// On Windows desktop apps this resolves under `%APPDATA%`. Pure UWP apps
    /// should use [`with_root_dir`](Self::with_root_dir) or the optional
    /// `uwp` feature instead.
    ///
    /// This constructor cannot report failure without an API break, so it
    /// falls back silently in two places instead of erroring:
    ///
    /// * if the platform configuration directory cannot be determined (for
    ///   example, no `HOME` or `%APPDATA%` in the environment), it falls back
    ///   to the current directory (`.`);
    /// * if the current executable's name cannot be determined, or is not a
    ///   safe path component, it falls back to the literal name `"app"`.
    ///
    /// **The second fallback is a fixed constant.** Any two executables that
    /// both hit it resolve to the same settings file and can silently read
    /// and overwrite each other's settings. If that is not acceptable, use
    /// [`try_new`](Self::try_new) to fail instead of falling back, or
    /// [`for_app`](Self::for_app) to supply an explicit identity so nothing
    /// is derived in the first place.
    pub fn new() -> Self {
        let folder_path = default_config_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(default_runtime_app_name());
        Self::from_parts(folder_path, DEFAULT_FILE_NAME)
    }

    /// Creates a config manager using the OS-standard config directory and
    /// the current executable name, failing rather than silently
    /// substituting either.
    ///
    /// Fail-closed counterpart to [`new`](Self::new): returns
    /// [`ConfigError::Platform`] if the platform configuration directory
    /// cannot be resolved, or if the current executable's name cannot be
    /// determined or is not a safe path component -- the two cases `new()`
    /// papers over with `.` and `"app"` respectively. Both failures report
    /// through the same variant with distinguishable messages, since adding
    /// a dedicated variant would itself be a breaking change: `ConfigError`
    /// is not `#[non_exhaustive]`, so a new variant breaks any exhaustive
    /// `match` on it.
    ///
    /// Prefer this over `new()` when you genuinely want the executable's
    /// derived name but sharing a settings file with another executable
    /// that hits the same fallback is not acceptable. If you have a stable
    /// application identity to supply instead, prefer
    /// [`for_app`](Self::for_app), which needs no derivation at all.
    pub fn try_new() -> Result<Self> {
        let folder_path = default_config_dir()?.join(app_name_from(std::env::current_exe().ok())?);
        Ok(Self::from_parts(folder_path, DEFAULT_FILE_NAME))
    }

    /// Creates a config manager for an explicit application name.
    ///
    /// This is the recommended desktop constructor for production apps because
    /// the storage directory is stable even if the executable file name changes.
    /// The `app_name` must be a single safe path component, not a path.
    ///
    /// Returns [`ConfigError::Platform`] if the platform configuration
    /// directory cannot be resolved (for example, no `HOME` or `%APPDATA%` in
    /// the environment). Applications that hit this in practice — typically
    /// services or containers without a user environment — should supply a
    /// path explicitly with [`with_root_dir`](Self::with_root_dir) instead.
    pub fn for_app(app_name: &str) -> Result<Self> {
        let app_name = validate_path_component(app_name)?;
        let folder_path = default_config_dir()?.join(app_name);
        Ok(Self::from_parts(folder_path, DEFAULT_FILE_NAME))
    }

    fn from_parts<P>(folder_path: P, file_name: &str) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            folder_path: folder_path.into(),
            file_name: file_name.to_string(),
            json_format: JsonFormat::Pretty,
            save_mode: SaveMode::Atomic,
            _marker: PhantomData,
        }
    }

    /// Stores the settings file in the current working directory.
    ///
    /// If the current working directory cannot be determined, falls back to
    /// `"."`. Unlike [`for_app`](Self::for_app), this fallback is not
    /// surprising here: the caller explicitly asked for working-directory
    /// storage, and `"."` already means "the working directory" to the
    /// filesystem.
    pub fn at_current_dir(mut self) -> Self {
        self.folder_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        self
    }

    /// Stores the settings file in a caller-provided directory.
    ///
    /// This is the primary compatibility seam for sandboxed hosts, including
    /// Pure UWP. The host application may resolve its application data directory
    /// and pass it here.
    pub fn with_root_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.folder_path = path.into();
        self
    }

    /// Stores the settings file in a caller-provided directory.
    ///
    /// This method is kept for compatibility with v2.0.x. Prefer
    /// [`with_root_dir`](Self::with_root_dir) in new code.
    pub fn at_custom_dir<P: Into<PathBuf>>(self, path: P) -> Self {
        self.with_root_dir(path)
    }

    /// Stores the settings file under `ApplicationData.Current.LocalFolder`.
    ///
    /// This method is available only on Windows when the optional `uwp` feature
    /// is enabled.
    #[cfg(all(windows, feature = "uwp"))]
    pub fn at_uwp_local_folder(mut self) -> Result<Self> {
        self.folder_path = uwp_local_folder_dir()?;
        Ok(self)
    }

    /// Changes the settings file name without validation.
    ///
    /// This method is retained for v2.x compatibility. New code should prefer
    /// [`try_with_filename`](Self::try_with_filename), which rejects path-like
    /// names such as `../settings.json`.
    pub fn with_filename(mut self, name: &str) -> Self {
        self.file_name = name.to_string();
        self
    }

    /// Changes the settings file name after validating it as a plain file name.
    ///
    /// The accepted value must be a single file name, not an absolute path and
    /// not a relative path containing directory traversal.
    pub fn try_with_filename(mut self, name: &str) -> Result<Self> {
        self.file_name = validate_plain_file_name(name)?.to_string();
        Ok(self)
    }

    /// Stores JSON in compact form instead of pretty-printed form.
    pub fn disable_pretty_json(mut self) -> Self {
        self.json_format = JsonFormat::Compact;
        self
    }

    /// Selects the save strategy.
    ///
    /// The default is [`SaveMode::Atomic`]. Use [`SaveMode::Direct`] only when
    /// an application intentionally wants v2.2-style direct overwrite behavior
    /// or needs to work around unusual filesystem semantics.
    pub fn with_save_mode(mut self, mode: SaveMode) -> Self {
        self.save_mode = mode;
        self
    }

    /// Selects direct overwrite saves.
    ///
    /// This is a convenience alias for `with_save_mode(SaveMode::Direct)`.
    pub fn with_direct_save(self) -> Self {
        self.with_save_mode(SaveMode::Direct)
    }

    /// Returns the configured save strategy.
    pub fn save_mode(&self) -> SaveMode {
        self.save_mode
    }

    /// Returns the settings folder path.
    pub fn folder_path(&self) -> &Path {
        &self.folder_path
    }

    /// Returns the settings file name.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Returns the settings file path.
    pub fn path(&self) -> PathBuf {
        self.folder_path.join(&self.file_name)
    }

    /// Saves the complete configuration, replacing the existing file content.
    pub fn save(&self, config: &T) -> Result<()> {
        let content = serialize(config, self.json_format)?;
        save_to_path(&self.path(), &content, self.save_mode)
    }

    /// Loads a configuration file that is expected to already exist.
    pub fn load(&self) -> Result<T> {
        let content = fs::read_to_string(self.path())?;
        deserialize(&content)
    }
}

impl<T> Default for ConfigManager<T>
where
    T: Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ConfigManager<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    /// Loads the configuration, or creates and saves `T::default()` on first run.
    pub fn load_or_default(&self) -> Result<T> {
        let path = self.path();

        match fs::read_to_string(&path) {
            Ok(content) => deserialize(&content),

            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let default_config = T::default();
                self.save(&default_config)?;
                Ok(default_config)
            }

            Err(e) => Err(ConfigError::Io(e)),
        }
    }

    /// Applies a read-modify-write update and saves the result.
    pub fn update<F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut T),
    {
        let mut cfg = self.load_or_default()?;
        f(&mut cfg);
        self.save(&cfg)?;
        Ok(cfg)
    }
}
