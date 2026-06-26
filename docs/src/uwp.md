# Pure UWP support

Pure UWP apps should not rely on classic desktop `%APPDATA%` path resolution.
They should store app data under the app-local data location.

The recommended pattern is host-resolved storage:

```rust
let manager = ConfigManager::<Settings>::new()
    .with_root_dir(uwp_local_folder_path)
    .try_with_filename("settings.json")?;
```

The host app resolves the UWP local folder, then passes the directory to
`app-json-settings`.

## Optional crate resolver

When a Windows build enables the optional `uwp` feature, the crate exposes:

```rust
let manager = ConfigManager::<Settings>::new()
    .at_uwp_local_folder()?
    .try_with_filename("settings.json")?;
```

Feature declaration:

```toml
[dependencies]
app-json-settings = { version = "2", features = ["uwp"] }
```

The default build does not depend on the `windows` crate.

## Scope

The UWP support is intentionally narrow. It resolves the app-local storage root
and then reuses the normal JSON file logic. The crate does not request broad
filesystem capabilities and does not implement roaming settings.
