# RFC 025 — Path safety and explicit app identity

**Status.** Implemented (2.2.0)
**Tracks.** API safety and long-term maintainability.
**Touches.** `src/core.rs`, `src/core/validation.rs`, `src/core/error.rs`, docs, tests.

## Summary

Add checked APIs for explicit app identity and plain settings file names while
preserving v2.x compatibility for existing constructors.

## Motivation

`ConfigManager::new()` derives an app directory from the current executable name.
That is convenient, but production apps often need a stable identity independent
of executable layout. Also, unchecked file-name customization can accidentally
look like a path, which is not ideal for a settings crate.

## Goals

* Add `ConfigManager::for_app(app_name)`.
* Add `ConfigManager::try_with_filename(name)`.
* Add `ConfigError::InvalidPathComponent`.
* Add public validation helpers for users who want preflight checks.
* Keep `ConfigManager::new()` and `with_filename()` available.

## Non-goals

* Do not break existing v2.x callers.
* Do not remove `with_filename()` yet.
* Do not sanitize names silently.
* Do not define full OS-specific filename legality.

## External design

Recommended production constructor:

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?;
```

Checked file-name customization:

```rust
let manager = manager.try_with_filename("preferences.json")?;
```

The checked APIs reject empty values, `.` / `..`, path separators, Windows drive
separators, and control characters.

## Acceptance checklist

* `for_app()` is public and returns `Result<Self>`.
* `try_with_filename()` is public and returns `Result<Self>`.
* Invalid checked names return `ConfigError::InvalidPathComponent`.
* Existing `new()`, `with_filename()`, and `at_custom_dir()` remain available.
* Tests cover valid and invalid path components.
