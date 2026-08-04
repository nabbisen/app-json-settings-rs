# Changelog

## 2.6.0

### Added

* Added `ConfigManager::try_new()`, a fail-closed counterpart to `new()`.
  `new()` derives an application name from the current executable and falls
  back silently to the literal name `"app"` if that name cannot be
  determined or is not a safe path component — a fallback that is a fixed
  constant, so any two executables that both hit it resolve to the same
  settings file and can silently read and overwrite each other's settings.
  `try_new()` returns `ConfigError::Platform` instead of substituting
  either of `new()`'s two fallbacks (the executable-name one described
  above, and the platform-configuration-directory one `for_app()` already
  reported since 2.5.0).

### Changed

* `new()`'s rustdoc now states both of its silent fallbacks, including the
  previously undocumented executable-name one and the collision it can
  cause.
* `docs/src/api-guide.md` gains a constructor comparison across `new()`,
  `try_new()`, `for_app()`, and `with_root_dir()`.
* `docs/src/platform-behavior.md` documents the collision under
  default-root resolution.

`new()`'s behavior is unchanged — this closes a documentation and API gap
without touching what `new()` itself does. No new `ConfigError` variant:
both of `try_new()`'s failure sources report through the existing
`ConfigError::Platform`, since `ConfigError` is not `#[non_exhaustive]` and
adding a variant would itself be a breaking change (see
`docs/src/migration-v2.md` for the disclosure of that pre-existing issue).

### Compatibility

* Additive only. One new method; no signature or behavior change to
  anything existing.
* No new `ConfigError` variant, no new dependency.
* Minor release, not a patch.

## 2.5.1

### Changed

* `README.md` brought current with 2.5.0: states the declared MSRV, notes
  that `for_app()` reports storage-root resolution failure instead of
  falling back silently, and mentions the 2.5.0 permission-preservation and
  error-reporting behavior changes. Documentation links reorganized from a
  flat list into three reader paths, with every `docs/src/` page now linked
  from exactly one of them.
* Added `examples/recovery.rs`, demonstrating recovery from a settings file
  that fails to deserialize: catch the error, move the file aside, and
  continue with defaults.
* `docs/src/testing.md` now states the verification boundary explicitly:
  `examples/` and the `src/lib.rs` doctest are compiled and run by CI on
  three platforms; the illustrative Rust fragments elsewhere in `docs/src/`
  are not, and are now marked as such.

This is a documentation and example release. No API, behavior, dependency,
or MSRV change.

### Compatibility

* No public API change, no new dependency.
* No behavior change. `git diff --name-only 2.5.0..2.5.1` touches nothing
  under `src/`.
* Patch release, not a minor.

## 2.5.0

### Changed

* Atomic save now preserves an existing settings file's Unix permission bits
  across replacement, instead of the replaced file silently taking on a
  umask-derived mode. A file the application or user had restricted to
  `0600` no longer becomes `0644`-typical on the next save.
* Newly created settings files are created `0600` on Unix. Existing files are
  unaffected — preservation keeps whatever mode they already have; this only
  changes the default for files that do not exist yet.
* The temporary file used during atomic save is now created owner-only
  (`0600`) from the start, so its content is never briefly readable by other
  local users while it is being written.
* Applying an existing file's mode to the temporary file is best-effort: on
  filesystems that do not model permission bits (FAT, some network mounts),
  the file stays at the safer `0600` rather than falling back to a more
  permissive default.
* Windows and `SaveMode::Direct` are unchanged. See
  `docs/src/platform-behavior.md` for why Windows needs no equivalent change,
  and note that reasoning has not been verified empirically against a real
  Windows security descriptor.
* `ConfigManager::for_app()` now reports storage-root resolution failure
  instead of silently substituting a relative path. Previously, if the
  platform configuration directory could not be determined (no `HOME` on
  Unix/macOS, no `%APPDATA%` on Windows), `for_app()` would succeed anyway and
  quietly write settings under the current working directory. It now returns
  `Err(ConfigError::Platform(_))`, naming the missing variable. See
  `docs/src/migration-v2.md` for who is affected and how to adapt.
* `ConfigManager::new()` is unaffected by the above — it keeps falling back to
  the current directory, since it cannot report an error without an API
  break. Its fallback is now implemented explicitly rather than incidentally.
* Added a new documentation page, `docs/src/operational-contract.md`,
  stating the crate's concurrency contract (atomic reads are safe, writes
  are unlocked and last-writer-wins, and why the crate does not take a
  cross-process lock) and its behavior on corrupted or externally modified
  settings files (`load_or_default()` never silently resets). No behavior
  changed; this documents behavior the crate already had. See RFC 030.

This closes a permission-preservation regression introduced when
`SaveMode::Atomic` became the default in 2.3.0 (RFC 024); see RFC 029. It
does not make the crate a secret store — applications with real
secret-handling requirements should continue to use a platform keychain.

`for_app()`'s new error reporting closes a "false coverage" gap: applications
that already handle `for_app()`'s `Result` were not actually covering
storage-root resolution failure, because that `Result` never carried it. See
RFC 034.

### Compatibility

* No public API change, no new dependency, no `unsafe` added.
* Behavior change on Unix only. It affects newly created files, which are now
  `0600`, and existing files whose mode differs from the umask default, which
  now keep that mode instead of losing it on the next save. Files already
  sitting at the umask default are unaffected. Minor release, not a patch.
* `for_app()`'s signature is unchanged; only environments where platform
  resolution was already effectively broken (silently writing under `$CWD`)
  see a behavior difference, and that difference is surfacing an error where
  none was reported before. `with_root_dir()` remains the documented escape
  hatch and is unaffected. Minor release, not a patch.

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
* Moved RFC 024 to `rfcs/done/` and marked it implemented in 2.3.0.
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
* Moved RFC 021 to `rfcs/done/` and marked it implemented in 2.1.0.
* Replaced the old commented integration test file with real public API tests.

### Compatibility

* `ConfigManager::new()`, `with_filename()`, and `at_custom_dir()` remain available.
* Default features still do not pull in the `windows` crate.
* Atomic replacement writes are not implemented in 2.2.0; they are planned for 2.3.0.

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
