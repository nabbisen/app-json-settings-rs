# Changelog

## 2.1.0

### Added

* Added `ConfigManager::with_root_dir()` as the preferred caller-supplied root directory API.
* Added `ConfigManager::folder_path()` for inspection and tests.
* Added optional Windows UWP support behind the `uwp` feature.
* Added `ConfigManager::at_uwp_local_folder()` on Windows with `uwp`, resolving `ApplicationData.Current.LocalFolder`.
* Added unit tests for custom root handling, compatibility alias behavior, save/load, compact JSON, first-run default creation, and update persistence.

### Changed

* Kept the default dependency footprint unchanged except for existing `serde` and `serde_json`.
* Changed `ConfigManager::new()` to fall back to `app` instead of panicking if the executable name cannot be resolved.
* Added `Display` and `std::error::Error` implementations for `ConfigError`.
* Added `ConfigError::Platform` for optional platform-specific resolver failures.

### Compatibility

* `ConfigManager::at_custom_dir()` remains available and now delegates to `with_root_dir()`.
* UWP path resolution is optional. The default build does not depend on the `windows` crate.
