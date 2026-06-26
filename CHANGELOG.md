# Changelog

## 2.2.0

### Added

* Added `ConfigManager::for_app()` for explicit stable application identity.
* Added `ConfigManager::try_with_filename()` for checked plain-file-name configuration.
* Added `ConfigManager::file_name()` for public API inspection.
* Added public validation helpers: `is_plain_file_name()` and `is_safe_path_component()`.
* Added `ConfigError::InvalidPathComponent` for caller-supplied unsafe names.
* Added integration tests under `tests/`.
* Added mdBook-compatible documentation under `docs/src`.
* Added RFC lifecycle structure using `rfcs/done`, `rfcs/proposed`, and `rfcs/archive`.
* Added `rfcs/README.md`, `ROADMAP.md`, `NOTICE`, CI workflow, and RFC integrity script.

### Changed

* Bumped crate version to `2.2.0`.
* Shortened `README.md` and moved detail-oriented guidance to `docs/src`.
* Made JSON serialization and deserialization error mapping explicit at the call site.
* Moved RFC 021 to `rfcs/done/` and marked it implemented in v2.1.0.
* Replaced the old commented integration test file with real public API tests.

### Compatibility

* `ConfigManager::new()`, `with_filename()`, and `at_custom_dir()` remain available.
* Default features still do not pull in the `windows` crate.
* Atomic replacement writes are not implemented in v2.2.0; they are planned for v2.3.0.

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
