//! Tiny typed JSON settings persistence for Rust applications.
//!
//! `app-json-settings` stores one Serde-serializable Rust value as a JSON
//! settings file. The crate intentionally keeps the API small: choose a storage
//! root, optionally choose a file name, and then load, save, or update the value.
//!
//! ```rust,no_run
//! use app_json_settings::ConfigManager;
//!
//! #[derive(serde::Deserialize, serde::Serialize, Default)]
//! struct Settings {
//!     volume: u32,
//! }
//!
//! # fn main() -> app_json_settings::Result<()> {
//! let manager = ConfigManager::<Settings>::for_app("demo-app")?;
//! let settings = manager.load_or_default()?;
//! manager.update(|settings| settings.volume = settings.volume.saturating_add(1))?;
//! # let _ = settings;
//! # Ok(())
//! # }
//! ```

mod core;

pub use core::constant::DEFAULT_FILE_NAME;
pub use core::error::{ConfigError, Result};
pub use core::validation::{is_plain_file_name, is_safe_path_component};
pub use core::{ConfigManager, SaveMode};
