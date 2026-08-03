# RFC 024 — Save reliability and atomic write policy

**Status.** Implemented (2.3.0)
**Tracks.** Persistence durability and crash-safety behavior.
**Touches.** `src/core.rs`, `src/core/save.rs` or equivalent, tests, docs.

## Summary

Add an atomic-save path so settings replacement is less likely to leave a
partially written JSON file if the process or machine stops during a write.

## Motivation

In 2.2.0, `save()` writes directly to the final file path. This is simple and
reasonable for a tiny crate, but settings files are user-facing state. A crash or
power loss during direct write can corrupt the file.

## Goals

* Add a robust default save strategy where practical.
* Write to a temporary file in the same directory before replacing the target.
* Keep behavior small and understandable.
* Preserve compatibility for applications that prefer direct writes.
* Document Windows, macOS, and Unix behavior clearly.

## Non-goals

* Do not implement journaling.
* Do not implement schema migrations.
* Do not guarantee recovery from all storage-device failures.
* Do not add heavy dependencies.

## External design

Added a save mode:

```rust
pub enum SaveMode {
    Atomic,
    Direct,
}
```

and builder methods:

```rust
manager.with_save_mode(SaveMode::Atomic);
manager.with_direct_save();
```

`Atomic` is the default in 2.3.0. `Direct` remains available for applications
that intentionally want v2.2-style direct overwrite behavior or need to work
around unusual filesystem behavior.

## Implementation

The implemented algorithm:

1. Serialize JSON fully in memory.
2. Ensure the settings directory exists.
3. Write to a temporary file under the same directory.
4. Flush the temporary file.
5. Replace or rename it over the destination.
6. Best-effort sync the parent directory where supported.
7. Best-effort clean up the temporary file if an error occurs before replacement.

Unix-like platforms use same-directory `rename`. Windows uses a small internal
`MoveFileExW` wrapper with replace-existing and write-through flags, avoiding a
new default dependency. Other targets do not claim replacement of an existing file as atomic and should
use `SaveMode::Direct` or receive a target-specific implementation before
claiming strong atomic replacement behavior.

## Acceptance checklist

* Save mode API is documented.
* Tests cover direct save and atomic save.
* Temporary files do not normally remain after successful save.
* Error paths do not delete the last known good file.
* Platform notes are documented.


## Release notes

Implemented in 2.3.0.
