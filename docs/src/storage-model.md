# Storage model

The Rust snippets on this page are illustrative fragments, not compiled or
run by CI — see [Testing guide](testing.md#verification-boundary).

The crate stores one settings value in one JSON file.

```text
settings root directory/
  settings.json
```

The root directory and the file name are separate concepts.

## Root directory

A root directory can come from three places:

| API | Purpose |
|---|---|
| `ConfigManager::for_app("my-app")` | Recommended desktop app constructor. |
| `ConfigManager::new()` | Convenience constructor based on the current executable name. |
| `with_root_dir(path)` | Caller-provided root for tests, portable mode, and sandboxed hosts. |

`with_root_dir()` is deliberately unopinionated. It trusts the host application
to pass the correct directory. This is the key compatibility seam for Pure UWP
and other sandboxed hosts.

## File name

The default file name is `settings.json`.

New code should use `try_with_filename()` when changing it:

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?
    .try_with_filename("user-settings.json")?;
```

`try_with_filename()` accepts only a plain file name. It rejects path-like input
such as:

```text
../settings.json
a/b.json
a\b.json
C:settings.json
```

`with_filename()` remains available for v2.x compatibility, but new code should
prefer the checked API.

## Save behavior

Since 2.3.0, `save()` uses `SaveMode::Atomic` by default. It writes JSON to a
unique temporary file in the same directory, flushes it, and then replaces the
final settings file.

Applications that need the previous direct-write behavior can opt in:

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?
    .with_direct_save();
```

See [Save behavior](save-behavior.md) for platform notes and limits.
