# Changelog

## 2.4.1

### Fixed

* Fixed a default-build compile failure on every Windows target: a
  function-local `use std::os::windows::ffi::OsStrExt` did not extend to the
  sibling function that needed it. The import is now hoisted to module scope
  in `src/core/save.rs`. This affected the published 2.3.0 and 2.4.0 crates.
* Fixed two compile failures in the optional `uwp` feature on Windows: added
  the `Storage_Search` cargo feature required by `ApplicationData::LocalFolder()`,
  and adjusted `src/core/dir.rs` for `windows-result` 0.2's `Error::message()`
  returning `String` directly. The `uwp` feature now **compiles**; its runtime
  behavior against a real UWP application container remains **untested**, since
  CI cannot host one.

### Changed

* CI now runs the real check set (clippy, default/no-default-feature/example/doc
  tests) as a matrix across Linux, macOS, and Windows, instead of Linux only.
* `scripts/check-rfcs.sh` now treats an absent `rfcs/<state>` directory as
  empty instead of failing. Every CI run since the workflow was introduced in
  2.2.0 had failed on this check in a fresh checkout, because git does not
  track the empty `rfcs/archive/` directory.
* Added a CI job that verifies the crate builds under the declared MSRV
  (Rust 1.85.0).
* Added a written completion rule to `docs/src/maintainer-notes.md`: an RFC
  does not move to `rfcs/done/` while the release gate for its change is red.
* Added RFC 027 and RFC 028 to track this release.

### Compatibility

* No public API change.
* No dependency-set change. `Storage_Search` is a feature of the already-optional
  `windows` dependency.
* No persistence-format change.
* Applications on non-Windows platforms see no behavior difference. Applications
  on Windows go from "does not build" to "builds."

## 2.4.0

### Added

* Added executable Cargo examples: `basic`, `custom_root`, and `update`.
* Added `examples/README.md`.
* Added `docs/src/examples.md`.
* Added `cargo test --examples` to CI.
* Added RFC 026 for the minimal executable example set.

### Changed

* Bumped crate version to `2.4.0`.
* Updated README, quick-start, testing, and roadmap documentation to reference the runnable examples.

### Compatibility

* No public API changes.
* No dependency changes.
* Default build still does not depend on the `windows` crate.

## 2.3.0

### Added

* Added `SaveMode` with `Atomic` and `Direct` strategies.
* Added `ConfigManager::with_save_mode()`.
* Added `ConfigManager::with_direct_save()`.
* Added `ConfigManager::save_mode()`.
* Added atomic-save implementation using same-directory temporary files.
* Added Windows replacement support through a small internal `MoveFileExW` wrapper without adding a default `windows` dependency.
* Added `docs/src/save-behavior.md`.
* Added tests for save-mode selection, default atomic mode, atomic temporary-file cleanup, and serialization-failure preservation.

### Changed

* Bumped crate version to `2.3.0`.
* Changed the default save behavior from direct overwrite to atomic replacement.
* Moved RFC 024 to `rfcs/done/` and marked it implemented in v2.3.0.
* Updated storage, API, platform, testing, migration, and roadmap documentation for atomic save.

### Compatibility

* Public load/save/update APIs remain source-compatible.
* Applications that want v2.2-style direct overwrite behavior can call `with_direct_save()` or `with_save_mode(SaveMode::Direct)`.
* The default build still does not depend on the `windows` crate.

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
