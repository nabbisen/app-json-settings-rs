# Quick start

The Rust snippets on this page are illustrative fragments, not compiled or
run by CI — see [Testing guide](testing.md#verification-boundary). For
complete runnable versions, see [Executable examples](#executable-examples)
below.

Add the crate and Serde derive support:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
app-json-settings = "2"
```

Define a settings type:

```rust
#[derive(serde::Deserialize, serde::Serialize, Default)]
struct Settings {
    volume: u32,
    dark_mode: bool,
}
```

Load defaults on first run:

```rust
use app_json_settings::ConfigManager;

fn load_settings() -> app_json_settings::Result<Settings> {
    ConfigManager::<Settings>::for_app("my-app")?.load_or_default()
}
```

Update part of the settings value:

```rust
fn enable_dark_mode() -> app_json_settings::Result<()> {
    ConfigManager::<Settings>::for_app("my-app")?.update(|settings| {
        settings.dark_mode = true;
    })?;

    Ok(())
}
```

For tests, portable mode, and sandboxed hosts, pass an explicit root directory:

```rust
let manager = ConfigManager::<Settings>::new()
    .with_root_dir("./portable-config")
    .try_with_filename("settings.json")?;
```


## Executable examples

For complete runnable files, see the Cargo examples:

```sh
cargo run --example basic
cargo run --example custom_root
cargo run --example update
```
