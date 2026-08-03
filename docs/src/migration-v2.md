# Migration to v2

The Rust snippets on this page are illustrative, not compiled or run by CI
— see [Testing guide](testing.md#verification-boundary). Some deliberately
show pre-2.5.0 code and would not compile against the current crate even if
verified.

## v2.0.x to 2.1.0

`with_root_dir()` was added as the preferred name for caller-provided storage
roots. Existing `at_custom_dir()` code still works.

Pure UWP support is available through either host-resolved roots or the optional
`uwp` feature.

## 2.1.0 to 2.2.0

`for_app()` was added as the recommended production constructor:

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?;
```

`try_with_filename()` was added for checked file names:

```rust
let manager = manager.try_with_filename("settings.json")?;
```

`with_filename()` remains available for v2.x compatibility.


## 2.2.0 to 2.3.0

`SaveMode` was added and `save()` now uses `SaveMode::Atomic` by default.

Applications that intentionally want the previous direct overwrite behavior can
select it explicitly:

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?
    .with_direct_save();
```

The public loading, saving, and update methods remain source-compatible.


## v2.4.x to 2.5.0

`ConfigManager::for_app()` now reports storage-root resolution failure
instead of silently substituting a relative path.

**Who is affected.** Only applications running where the platform
configuration directory cannot be resolved: on Unix (excluding macOS) when
neither `XDG_CONFIG_HOME` nor `HOME` is set, on macOS when `HOME` is not set,
and on Windows when `%APPDATA%` is not set. This essentially never happens on
a normal desktop environment. It can happen for services or containers run
without a user environment — for example, a systemd unit without `User=`, or
some minimal container configurations.

**How it shows up.** Previously, `for_app()` would succeed and silently
resolve to a path under the current working directory. Now it returns
`Err(ConfigError::Platform(_))`, with a message naming the missing
environment variable.

**Before:**

```rust
// Succeeded even without HOME/%APPDATA%, silently writing under $CWD.
let manager = ConfigManager::<Settings>::for_app("my-app")?;
```

**After:**

```rust
let manager = match ConfigManager::<Settings>::for_app("my-app") {
    Ok(manager) => manager,
    Err(ConfigError::Platform(message)) => {
        eprintln!("could not resolve a config directory: {message}");
        // Supply a path explicitly instead of relying on platform resolution.
        ConfigManager::<Settings>::new().with_root_dir(chosen_path)
    }
    Err(error) => return Err(error),
};
```

`ConfigManager::new()` is unaffected — it keeps falling back to the current
directory, since it cannot report an error without breaking its signature.
Code that only ever calls `new()` sees no behavior change from this release.
