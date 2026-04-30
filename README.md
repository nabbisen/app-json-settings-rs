# App JSON Settings

[![crates.io](https://img.shields.io/crates/v/app-json-settings?label=rust)](https://crates.io/crates/app-json-settings)
[![Documentation](https://docs.rs/app-json-settings/badge.svg?version=latest)](https://docs.rs/app-json-settings/latest)
[![Dependency Status](https://deps.rs/crate/app-json-settings/latest/status.svg)](https://deps.rs/crate/app-json-settings)
[![License](https://img.shields.io/github/license/nabbisen/app-json-settings-rs)](https://github.com/nabbisen/app-json-settings-rs/blob/main/LICENSE)

**Typed application settings storage for Rust.**

`app-json-settings` persists a Rust struct as your application configuration.
You do not manipulate JSON manually and you do not manage config file paths yourself.

The library focuses on safe, minimal, cross-platform configuration handling.

## Quick start

```ini
# Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
app-json-settings = "2"
```

```rust
use app_json_settings::ConfigManager;

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Settings {
    volume: u32,
}

fn save() -> app_json_settings::Result<()> {
    ConfigManager::new().save(&Settings { volume: 100 })?;
    Ok(())
}

fn load() -> app_json_settings::Result<()> {
    let settings = ConfigManager::<Settings>::new().load_or_default()?;
    println!("{}", settings.volume);
    Ok(())
}
```

---

## Why this library

Writing config handling code repeatedly leads to:

* fragile first-run initialization
* manual path handling per OS
* read → modify → write mistakes
* string-key based settings

This crate removes those concerns and lets the struct be the configuration.

## Design goals

* Minimal API surface
* Predictable behavior
* No runtime dependencies beyond Serde
* Works for CLI, GUI, mobile, and server applications

This crate is not a general database or dynamic settings system.
It is a typed persistent configuration layer.

---

## Features

### Safe configuration I/O

* Load and save settings with minimal code
* `load_or_default()` prevents first-run crashes
* `update()` provides safe read-modify-write

### Typed serialization

* Uses your own struct as the configuration model
* No string keys
* Compile-time refactor safety
* Serde `Serialize` / `Deserialize` supported

### JSON focused

* JSON only (intentionally)
* Selectable output:

  * compact
  * pretty
* Suitable for mobile I/O constraints while still human-readable

### Cross-platform support

The library internally resolves platform-specific config directories:

* Windows → `%APPDATA%`
* macOS → `~/Library/Application Support`
* Linux / Unix → `$XDG_CONFIG_HOME` or `~/.config`

No platform conditional code is required in your application.

---

## Customization

```rust
let config = ConfigManager::<Settings>::new()
    .with_filename("user.json")
    .disable_pretty_json();
```

Options:

* custom directory
* custom file name
* compact or pretty JSON output

## Partial update

You usually do not need to manually load and save.

```rust
config.update(|s| {
    s.volume = 200;
})?;
```

`update()` guarantees a safe read-modify-write cycle.

---

## Choosing the right API

| Function            | When to use                             |
| ------------------- | --------------------------------------- |
| `load()`            | Config file must already exist          |
| `load_or_default()` | Normal application startup              |
| `save()`            | Replace entire configuration            |
| `update()`          | Modify part of the configuration safely |

---

## Open-source, with care

This project is lovingly built and maintained by volunteers. We hope it helps streamline your work. Please understand that the project has its own direction — while we welcome feedback, it might not fit every edge case 🌱

## Acknowledgements

Depends on the crates of [serde](https://serde.rs/), [serde_json](https://github.com/serde-rs/json).
