# RFC 024 — Save reliability and atomic write policy

**Status.** Proposed
**Tracks.** Persistence durability and crash-safety behavior.
**Touches.** `src/core.rs`, `src/core/save.rs` or equivalent, tests, docs.

## Summary

Add an atomic-save path so settings replacement is less likely to leave a
partially written JSON file if the process or machine stops during a write.

## Motivation

In v2.2.0, `save()` writes directly to the final file path. This is simple and
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

## Candidate external design

Add a save mode:

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

The preferred default for v2.3.0 is `Atomic` if the implementation is robust on
all supported platforms. If not, keep `Direct` as default and make atomic save an
opt-in feature until confidence improves.

## Implementation notes

A likely algorithm:

1. Ensure the settings directory exists.
2. Serialize JSON fully in memory.
3. Write to a temporary file under the same directory.
4. Flush the temporary file.
5. Replace or rename it over the destination.
6. Best-effort clean up the temporary file if an error occurs.

Windows replacement behavior needs explicit review before this RFC is accepted.

## Acceptance checklist

* Save mode API is documented.
* Tests cover direct save and atomic save.
* Temporary files do not normally remain after successful save.
* Error paths do not delete the last known good file.
* Platform notes are documented.
