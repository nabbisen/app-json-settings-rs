# Error handling

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
