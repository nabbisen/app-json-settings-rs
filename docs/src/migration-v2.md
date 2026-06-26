# Migration to v2

## v2.0.x to v2.1.0

`with_root_dir()` was added as the preferred name for caller-provided storage
roots. Existing `at_custom_dir()` code still works.

Pure UWP support is available through either host-resolved roots or the optional
`uwp` feature.

## v2.1.0 to v2.2.0

`for_app()` was added as the recommended production constructor:

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?;
```

`try_with_filename()` was added for checked file names:

```rust
let manager = manager.try_with_filename("settings.json")?;
```

`with_filename()` remains available for v2.x compatibility.
