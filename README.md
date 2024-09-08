# App JSON Settings

App settings as JSON format stored in file and available via read-by-key and write-by-key.

Aims a tiny settings manager with reasonably few dependencies.

[![crates.io](https://img.shields.io/crates/v/app-json-settings?label=latest)](https://crates.io/crates/app-json-settings)
[![Documentation](https://docs.rs/app-json-settings/badge.svg?version=latest)](https://docs.rs/app-json-settings/latest)
[![License](https://img.shields.io/github/license/nabbisen/app-json-settings-rs)](https://github.com/nabbisen/app-json-settings-rs/blob/main/LICENSE)
[![Dependency Status](https://deps.rs/crate/app-json-settings/latest/status.svg)](https://deps.rs/crate/app-json-settings)

## Examples

### Rust - as Tauri backend

```rust
use app_json_settings::JsonSettigs;

#[tauri::command]
fn settings_read_by_key(key: &str) -> Result<KeyValue, String> {
    JsonSettigs::exe_dir().read_by_key(key).map_err(|err| err.to_string())
}

#[tauri::command]
fn settings_write_by_key(key: &str, value: Value) -> Result<(), String> {
    JsonSettigs::exe_dir().write_by_key(key, &value).map_err(|err| err.to_string())
}
```

Instead of `JsonSettigs::exe_dir()` above, where to store the settings file has options.

| fn | where to store |
| -- | -------------- |
| `exe_dir()` | the same to where the executable is |
| `config_dir()` | points to app dir in user config dir. the app dir name is automatically defined due to the executable name |
| `new(filepath)` | custom path and file name |

### TypeScript - as Tauri frontend

```ts
import { invoke } from '@tauri-apps/api/core'

interface ReadByKeyResponse {
  key: string
  value: unknown
  file_exists: boolean
  key_exists: boolean
}

const read = (key: string): Promise<unknown> => {
  return invoke('settings_read_by_key', { key: key }).then((res) => {
    const _res = res as ReadByKeyResponse
    if (!_res.file_exists || !_res.key_exists) return undefined
    return _res.value
  })
}

const write = <T>(key: string, value: any) => {
  invoke('settings_write_by_key', { key: key, value: value })
}
```

### settings.json

```json
{
  "keyBoolean": true,
  "keyNumber": 1000,
  "keyString": "Hello world."
}
```

## Acknowledgements

Depends on: [serde](https://serde.rs/) / [serde_json](https://github.com/serde-rs/json) .
