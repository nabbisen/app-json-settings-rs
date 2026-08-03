# Error handling

The Rust snippets on this page are illustrative fragments, not compiled or
run by CI — see [Testing guide](testing.md#verification-boundary).

Most APIs return `app_json_settings::Result<T>`.

The error type is `ConfigError`:

| Variant | Meaning |
|---|---|
| `Io` | File-system or stream I/O failed. |
| `Serialize` | JSON serialization failed while saving. |
| `Deserialize` | JSON deserialization failed while loading. |
| `InvalidPathComponent` | A checked app name or file name was unsafe. |
| `Platform` | A platform-specific resolver failed. |

JSON serialization and deserialization are mapped explicitly at the call site so
callers can distinguish save failures from load failures.

Example:

```rust
match manager.load() {
    Ok(settings) => settings,
    Err(app_json_settings::ConfigError::Deserialize(error)) => {
        eprintln!("settings file is invalid JSON: {error}");
        Settings::default()
    }
    Err(error) => return Err(error),
}
```

See [Operational contract](operational-contract.md) for what `load()` and
`load_or_default()` guarantee under concurrent access and when the settings
file exists but does not parse, including a worked recovery pattern.

## `ConfigError::Platform`

Two things produce this variant: `ConfigManager::for_app()`, when the
platform configuration directory cannot be resolved (no `HOME` on Unix or
macOS, no `%APPDATA%` on Windows), and the optional UWP local-folder
resolver.

The remedy is the same in both cases: supply a path explicitly with
`with_root_dir()` instead of relying on platform resolution. See
[Migration to v2](migration-v2.md) for the upgrade guidance this produced in
2.5.0.
