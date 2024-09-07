# App JSON Settings

App settings as JSON format stored in file and available via read-by-key and write-by-key.

Aims a tiny settings manager with reasonably few dependencies.

## Examples

### Rust - as Tauri backend

```rust
use app_json_settings::JsonSettigs;

#[tauri::command]
fn settings_read_by_key(key: &str) -> Result<KeyValue, String> {
    JsonSettigs::default().read_by_key(key).map_err(|err| err.to_string())
}

#[tauri::command]
fn settings_write_by_key(key: &str, value: Value) -> Result<(), String> {
    JsonSettigs::default().write_by_key(key, &value).map_err(|err| err.to_string())
}
```

### TypeScript - as Tauri frontend

```ts
import { invoke } from '@tauri-apps/api/core'

const read = (key: string) => {
  invoke('settings_read_by_key', { key: key })
}

const write = (key: string, value: any) => {
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
