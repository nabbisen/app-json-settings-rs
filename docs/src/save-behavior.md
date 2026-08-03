# Save behavior

The Rust snippets on this page are illustrative fragments, not compiled or
run by CI — see [Testing guide](testing.md#verification-boundary).

`save()` serializes the complete settings value to JSON in memory and then
writes the resulting JSON to the configured settings path.

## Default: atomic save

Since 2.3.0, the default save mode is `SaveMode::Atomic`.

```rust
use app_json_settings::{ConfigManager, SaveMode};

let manager = ConfigManager::<Settings>::for_app("my-app")?;
assert_eq!(manager.save_mode(), SaveMode::Atomic);
```

Atomic save uses this sequence:

1. Serialize the settings value fully in memory.
2. Create the settings directory if needed.
3. Write JSON to a unique temporary file in the same directory.
4. Flush the temporary file.
5. Replace the destination path with the temporary file.
6. Best-effort sync the parent directory where supported.
7. Best-effort remove the temporary file if an error occurs before replacement.

This avoids exposing a partially written final settings file if the process stops
while writing the new JSON content.

## Permissions on Unix

Since 2.5.0, atomic save preserves the existing settings file's permission
bits on Unix, and the temporary file is created owner-only (`0600`) so its
content is never briefly readable by other local users while it is being
written.

* If the target file already exists, its mode is applied to the temporary
  file before the replace step.
* If no target file exists yet, the new file keeps the `0600` it was created
  with.
* Applying the target's mode is best-effort and fail-secure: if reading or
  applying the mode fails (for example, on a filesystem that does not model
  permission bits, such as FAT or some network mounts), the file stays at
  `0600` rather than falling back to a more permissive default. The result is
  never more permissive than intended, only possibly more restrictive.
* **Group-shared caveat.** A settings file under a shared `with_root_dir()`
  path is created owner-only on first save. This only bites once — running
  `chmod g+r` on the file after creation fixes it permanently, because every
  later save preserves whatever mode the file already has.

This does not make the crate a secret store. Applications with real
secret-handling requirements should use a platform keychain.

## Direct save

Direct save is still available:

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?
    .with_direct_save();
```

or:

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?
    .with_save_mode(SaveMode::Direct);
```

Direct save writes directly to the final file path. This matches the 2.2.0
behavior and may be useful for debugging, unusual filesystems, or applications
that intentionally want simple overwrite semantics.

## Platform notes

On Unix-like platforms, atomic replacement uses `rename` with the temporary file
in the same directory as the final file.

On Windows, atomic replacement uses a small internal `MoveFileExW` wrapper with
replace-existing and write-through flags. This avoids adding the `windows` crate
to the default dependency set.

On other targets, replacement of an existing file is not claimed as atomic. Such
targets should use `SaveMode::Direct` or receive a target-specific replacement
implementation before claiming strong atomic replacement behavior.

## Limits

Atomic save is not journaling. It does not guarantee recovery from every storage
failure, filesystem bug, or hardware failure. It is a pragmatic reliability
improvement for the common settings-file case.
