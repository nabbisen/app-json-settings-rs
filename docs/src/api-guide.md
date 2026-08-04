# API guide

The Rust snippets on this page are illustrative fragments, not compiled or
run by CI — see [Testing guide](testing.md#verification-boundary).

## Constructors

### `ConfigManager::for_app(app_name)`

Recommended for production desktop apps.

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?;
```

The app name must be a safe single path component.

Since 2.5.0, this constructor also reports when the platform configuration
directory itself cannot be resolved (for example, no `HOME` or `%APPDATA%` in
the environment), returning `ConfigError::Platform`. This is the constructor
to prefer specifically because it does not hide that failure.

### `ConfigManager::new()`

Convenience constructor that derives the app directory from the current
executable name.

```rust
let manager = ConfigManager::<Settings>::new();
```

This is convenient for examples and small tools, but it cannot report
failure without an API break, so it falls back silently in two places
instead:

* if the platform configuration directory cannot be resolved, it falls back
  to the current directory;
* if the current executable's name cannot be determined or is not a safe
  path component, it falls back to the literal name `"app"`.

**The second fallback is a fixed constant.** Any two executables that both
hit it resolve to the same settings file and can silently read and
overwrite each other's settings. If that is not acceptable, prefer
`try_new()` or `for_app()` below.

### `ConfigManager::try_new()`

Fail-closed counterpart to `new()`: the same executable-derived identity,
but returns `ConfigError::Platform` instead of substituting either
fallback.

```rust
let manager = ConfigManager::<Settings>::try_new()?;
```

Prefer this over `new()` when you genuinely want the executable's derived
name but sharing a settings file with another executable that hits the
same fallback is not acceptable. If you have a stable application identity
to supply instead, `for_app()` needs no derivation at all.

### `with_root_dir(path)`

Overrides the settings root directory.

```rust
let manager = ConfigManager::<Settings>::new().with_root_dir("./config");
```

Use this for tests, portable mode, app-managed storage locations, and sandboxed
hosts.

### Choosing a constructor

| Constructor | Identity | On failure |
|---|---|---|
| `new()` | Derived from the executable name | Falls back silently (`.` and `"app"`) |
| `try_new()` | Derived from the executable name | Returns `ConfigError::Platform` |
| `for_app(name)` | Explicit, caller-supplied | Returns `ConfigError::Platform` (directory only — there is nothing to derive) |
| `with_root_dir(path)` | Caller-supplied directory | N/A — the caller already resolved it |

`new()` is the convenient default. `try_new()` and `for_app()` are both
fail-closed; choose `try_new()` when the executable-derived name is what
you actually want, and `for_app()` when you have a stable name to supply
directly.

### Inspecting what was resolved

A third remedy for the collision hazard above needs no constructor change
at all: `folder_path()` returns the directory `ConfigManager` resolved, so
a caller that wants to keep using `new()` can assert the result matches
expectation instead of switching to `try_new()` or `for_app()`.

```rust
let manager = ConfigManager::<Settings>::new();
assert_eq!(
    manager.folder_path().file_name(),
    Some(std::ffi::OsStr::new("my-app")),
);
```

If the assertion fails, the executable-name fallback fired and produced
something other than `"my-app"` — most likely the literal `"app"`.

## File names

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?
    .try_with_filename("preferences.json")?;
```

`try_with_filename()` validates that the value is a plain file name.

## Loading and saving

```rust
manager.save(&settings)?;
let settings = manager.load()?;
```

`save()` uses atomic replacement by default since 2.3.0. `load()` expects the
file to already exist.

For normal app startup:

```rust
let settings = manager.load_or_default()?;
```

## Save mode

```rust
use app_json_settings::SaveMode;

let manager = ConfigManager::<Settings>::for_app("my-app")?
    .with_save_mode(SaveMode::Direct);
```

`SaveMode::Atomic` is the default. `SaveMode::Direct` is available when an
application intentionally wants direct overwrite behavior.

## Updates

```rust
manager.update(|settings| {
    settings.launch_count += 1;
})?;
```

`update()` performs a load-or-default, applies the closure, and saves the result
using the configured save mode.
