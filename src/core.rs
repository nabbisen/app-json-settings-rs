use serde::{de::DeserializeOwned, Serialize};

use std::fs;
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::core::constant::DEFAULT_FILE_NAME;
use crate::core::dir::default_config_dir;
#[cfg(all(windows, feature = "uwp"))]
use crate::core::dir::uwp_local_folder_dir;
use crate::core::json::JsonFormat;
use crate::ConfigError;
use crate::Result;

pub mod constant;
mod dir;
pub mod error;
mod json;

#[cfg(test)]
mod tests;

pub struct ConfigManager<T> {
    folder_path: PathBuf,
    file_name: String,
    json_format: JsonFormat,
    _marker: PhantomData<T>,
}

impl<T> ConfigManager<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Creates a config manager using the OS standard config directory and current executable name.
    ///
    /// On Windows desktop apps this resolves under `%APPDATA%`.
    /// Pure UWP apps should use `with_root_dir` or the optional `uwp` feature instead.
    pub fn new() -> Self {
        let app_name = std::env::current_exe()
            .ok()
            .and_then(|path| path.file_stem().map(|name| name.to_string_lossy().to_string()))
            .unwrap_or_else(|| "app".to_string());

        let folder_path = default_config_dir().join(&app_name);

        Self {
            folder_path,
            file_name: DEFAULT_FILE_NAME.to_string(),
            json_format: JsonFormat::Pretty,
            _marker: PhantomData,
        }
    }

    /// Stores the settings file in the current working directory.
    pub fn at_current_dir(mut self) -> Self {
        self.folder_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        self
    }

    /// Stores the settings file in a caller-provided directory.
    ///
    /// This is the primary compatibility seam for sandboxed hosts, including Pure UWP.
    /// The host application may resolve its application data directory and pass it here.
    pub fn with_root_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.folder_path = path.into();
        self
    }

    /// Stores the settings file in a caller-provided directory.
    ///
    /// This method is kept for compatibility with v2.0.x. Prefer `with_root_dir` in new code.
    pub fn at_custom_dir<P: Into<PathBuf>>(self, path: P) -> Self {
        self.with_root_dir(path)
    }

    /// Stores the settings file under `ApplicationData.Current.LocalFolder`.
    ///
    /// This method is available only on Windows when the optional `uwp` feature is enabled.
    #[cfg(all(windows, feature = "uwp"))]
    pub fn at_uwp_local_folder(mut self) -> Result<Self> {
        self.folder_path = uwp_local_folder_dir()?;
        Ok(self)
    }

    /// Changes the settings file name.
    pub fn with_filename(mut self, name: &str) -> Self {
        self.file_name = name.to_string();
        self
    }

    /// Stores JSON in compact form instead of pretty-printed form.
    pub fn disable_pretty_json(mut self) -> Self {
        self.json_format = JsonFormat::Compact;
        self
    }

    /// Returns the settings folder path.
    pub fn folder_path(&self) -> &Path {
        &self.folder_path
    }

    /// Returns the settings file path.
    pub fn path(&self) -> PathBuf {
        self.folder_path.join(&self.file_name)
    }

    /// Saves the complete configuration, replacing the existing file content.
    pub fn save(&self, config: &T) -> Result<()> {
        if !self.folder_path.exists() {
            fs::create_dir_all(&self.folder_path)?;
        }

        let content = match self.json_format {
            JsonFormat::Compact => serde_json::to_string(config)?,
            JsonFormat::Pretty => serde_json::to_string_pretty(config)?,
        };

        fs::write(self.path(), content)?;
        Ok(())
    }

    /// Loads a configuration file that is expected to already exist.
    pub fn load(&self) -> Result<T> {
        let content = fs::read_to_string(self.path())?;
        Ok(serde_json::from_str(&content)?)
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
            Ok(content) => Ok(serde_json::from_str(&content)?),

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
