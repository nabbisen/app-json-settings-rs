# App JSON Settings

[![License](https://img.shields.io/github/license/nabbisen/app-json-settings-rs)](LICENSE)
[![Documentation](https://docs.rs/app-json-settings/badge.svg?version=latest)](https://docs.rs/app-json-settings/latest)
[![crates.io](https://img.shields.io/crates/v/app-json-settings?label=rust)](https://crates.io/crates/app-json-settings)
[![Dependency Status](https://deps.rs/crate/app-json-settings/latest/status.svg)](https://deps.rs/crate/app-json-settings)

**Tiny typed JSON settings persistence for Rust applications.**

`app-json-settings` stores one Serde-serializable Rust value as a JSON settings
file. It is designed for small GUI, CLI, and local-first apps that want typed
settings without hand-written path and JSON boilerplate.

## Overview

The crate focuses on a small API:

* choose a settings storage root
* optionally choose a file name
* load, save, or update one typed settings value

It intentionally stays JSON-focused and does not try to become a database,
preference service, or general configuration framework.

## Why / when

Use this crate when your application has a small settings struct and you want:

* first-run defaults with `load_or_default()`
* read-modify-write updates with `update()`
* atomic save by default, with direct save still available
* OS-default config locations for desktop apps — with a clear error, not a
  silent fallback, if that location can't be resolved (`with_root_dir()`
  covers the exception)
* caller-provided storage roots for tests, portable mode, or sandboxed hosts
* optional Pure UWP local-folder resolution without a default `windows` dependency

## Quick start

```toml
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

fn main() -> app_json_settings::Result<()> {
    let manager = ConfigManager::<Settings>::for_app("my-app")?;

    let settings = manager.load_or_default()?;
    println!("volume = {}", settings.volume);

    manager.update(|settings| {
        settings.volume = 100;
    })?;

    Ok(())
}
```

## Features / design notes

* Default build depends only on `serde` and `serde_json`.
* `ConfigManager::for_app()` is the recommended desktop constructor, and
  reports storage-root resolution failure instead of silently falling back
  to the working directory.
* `ConfigManager::with_root_dir()` is the sandbox-friendly storage seam.
* `ConfigManager::try_with_filename()` validates plain file names.
* `SaveMode::Atomic` is the default save strategy. It preserves the previous
  file's Unix permission bits across replacement, and creates new files
  owner-only (`0600`).
* `with_filename()` remains available for v2.x compatibility.
* The optional `uwp` feature enables `at_uwp_local_folder()` on Windows.
* Requires Rust 1.85.0 or newer, verified in CI.

## Examples

A small executable example set is available under `examples/`:

```sh
cargo run --example basic
cargo run --example custom_root
cargo run --example update
cargo run --example recovery
```

## More detail

Full documentation is maintained under `docs/src` and can be read with mdBook.

* **Getting started** — [introduction](docs/src/introduction.md),
  [quick start](docs/src/quick-start.md),
  [executable examples](docs/src/examples.md)
* **Using the crate** — [storage model](docs/src/storage-model.md),
  [save behavior](docs/src/save-behavior.md),
  [API guide](docs/src/api-guide.md),
  [error handling](docs/src/error-handling.md),
  [operational contract](docs/src/operational-contract.md),
  [platform behavior](docs/src/platform-behavior.md),
  [Pure UWP support](docs/src/uwp.md)
* **Contributing and maintaining** — [testing guide](docs/src/testing.md),
  [migration to v2](docs/src/migration-v2.md),
  [maintainer notes](docs/src/maintainer-notes.md)

## Acknowledgements

Depends on [serde](https://serde.rs/) and
[serde_json](https://github.com/serde-rs/json).
