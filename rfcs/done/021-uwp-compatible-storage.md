# RFC 021 — Optional UWP-Compatible Storage Root

**Status.** Implemented (v2.1.0)
**Tracks.** Windows sandbox compatibility and dependency policy.
**Touches.** `Cargo.toml`, `src/core.rs`, `src/core/dir.rs`, `README.md`.

## Summary

Add Pure UWP-compatible settings storage support without increasing the default
dependency footprint for ordinary desktop users.

## Motivation

`app-json-settings` is commonly useful for Rust GUI applications. On Windows,
classic desktop applications can use `%APPDATA%`, but Pure UWP apps should store
application data under their app-local storage area instead of assuming classic
Win32 application data paths.

The crate should support this model while keeping the default build small.

## Goals

* Keep the default build free from a `windows` dependency.
* Add a clear caller-supplied root directory API for sandboxed hosts.
* Add an optional Windows-only UWP convenience resolver.
* Preserve v2.0.x compatibility for `at_custom_dir()`.
* Keep JSON read/write logic shared across desktop and sandboxed hosts.

## Non-goals

* Do not turn the crate into a general Windows storage abstraction.
* Do not support arbitrary UWP file-system capabilities.
* Do not use roaming settings or roaming folders as a primary design.
* Do not require UWP support for normal desktop users.

## External design

### Default path behavior

`ConfigManager::new()` continues to resolve a desktop-oriented config directory
and append the executable stem as the application directory.

### Caller-supplied root directory

`ConfigManager::with_root_dir(path)` sets the directory that contains the JSON
settings file. This is the primary compatibility seam for Pure UWP, tests,
portable apps, and other sandboxed hosts.

### Optional UWP resolver

When built on Windows with the `uwp` feature, the crate exposes:

```rust
ConfigManager::<Settings>::new().at_uwp_local_folder()?;
```

This resolves `Windows.Storage.ApplicationData.Current.LocalFolder`, converts it
to a `PathBuf`, and then reuses the existing JSON file I/O path.

## Dependency policy

```toml
[features]
default = []
uwp = ["dep:windows"]
```

The `windows` crate is target-specific, optional, and enabled only by the `uwp`
feature.

## Compatibility

`at_custom_dir()` remains available as a v2.0.x-compatible alias. New code should
prefer `with_root_dir()` because it better communicates the storage-root concept.

## Acceptance checklist

* `Cargo.toml` version is `2.1.0`.
* Default features do not pull in `windows`.
* `with_root_dir()` is available without feature flags.
* `at_uwp_local_folder()` is gated by `cfg(all(windows, feature = "uwp"))`.
* Existing save, load, load-or-default, and update behavior remains intact.
* Tests cover custom-root behavior and normal JSON persistence behavior.
