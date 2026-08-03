# Operational contract

This page states two behaviors the crate already has and has always had, but
never wrote down: what happens when more than one process touches the same
settings file, and what happens when the file exists but does not parse.
Nothing on this page changes crate behavior — it documents the current
behavior precisely enough to be a commitment.

## Concurrency

**Reads are safe against concurrent writes.** Since 2.3.0 the default save
mode (`SaveMode::Atomic`) replaces the settings file by an atomic
platform-level replace (`rename` on Unix, an internal `MoveFileExW` wrapper
on Windows). A concurrent reader therefore always sees either the complete
previous content or the complete new content — never a partial write. This is
the failure most people fear from concurrent file access, and the crate
genuinely prevents it under the default save mode.

`SaveMode::Direct` does **not** provide this guarantee: it writes to the
final path in place, so a reader can observe a partially written file while a
direct save is in progress.

**Writes are not coordinated.** `update()` is an unlocked read-modify-write:
it loads the current value (or the default), applies the given closure, and
saves the result. If two processes call `update()` at close to the same time,
each reads, modifies, and writes independently — the second write simply
replaces the first. **Last writer wins.**

This is a deliberate contract, not an oversight:

* There is no cross-process lock anywhere in the crate, and it will not
  acquire one. Advisory locking means hand-written platform FFI (`flock` on
  Unix, `LockFileEx` on Windows) in a project that has just spent a release
  recovering from exactly that kind of FFI going wrong. Advisory locks are
  also not honored on every filesystem — some network mounts silently ignore
  them — so the guarantee would be conditional in ways that are hard to state
  honestly.
* The crate's stated audience — small desktop, CLI, and local-first
  applications — is overwhelmingly single-writer.

An application that genuinely needs coordinated writes across processes
should serialize them itself: a single designated writer process, an
in-process mutex where one process owns the file, or application-level
locking built for the application's actual requirements.

## Corrupted or externally modified files

* `load()` on a file that exists but is not valid JSON returns
  `ConfigError::Deserialize`.
* `load_or_default()` creates and saves `T::default()` **only when the file
  is absent**. A file that exists but does not parse returns
  `ConfigError::Deserialize` — it does **not** silently fall back to
  defaults.
* A file that parses as JSON but does not match the shape of `T` is also a
  `Deserialize` error.

That last point is worth prominence: the most common real-world
"corruption" is not a mangled file. It is the application's own settings
struct gaining a new required field with no `#[serde(default)]`, so a
previously valid settings file now fails to deserialize against the new
type. Adding `#[serde(default)]` to new fields (or giving the whole struct a
`Default` impl and deriving field-level defaults) is the mitigation that
actually applies to this case, not a recovery routine.

**Why `load_or_default()` does not reset on invalid content.** Silently
replacing unreadable data with defaults would destroy user state with no
signal that anything happened. A user who hand-edits their settings file and
introduces a typo would lose the rest of their configuration without being
told. The crate surfaces the error instead and lets the application decide
— the application is the only party that knows whether the existing data is
precious.

## Symlinked settings paths

Atomic replacement's building block, an OS-level `rename` over the
destination path, has a consequence worth stating explicitly if the settings
path is itself a symlink: `rename` replaces the symlink with the temporary
file, not the file the symlink points to. Permission preservation (see
[Save behavior](save-behavior.md#permissions-on-unix)) reads the *link
target's* mode before that replacement, so the mode that gets applied is the
target's, not the symlink's own. After the save, the path that was a symlink
is a regular file.

This is inherent to how atomic file replacement works in general — it is not
something this crate adds or could reasonably opt out of without abandoning
atomicity — but it is easy to miss if the settings path was expected to stay
a symlink across saves.

## Recovery pattern

A worked pattern for handling a settings file that fails to deserialize:
move it aside, continue with defaults, and tell the user so they know their
previous settings did not simply vanish.

```rust
use app_json_settings::{ConfigError, ConfigManager};

fn load_settings(
    manager: &ConfigManager<Settings>,
) -> app_json_settings::Result<Settings> {
    match manager.load_or_default() {
        Ok(settings) => Ok(settings),
        Err(ConfigError::Deserialize(error)) => {
            eprintln!("settings file is invalid, moving it aside: {error}");
            let backup = manager.path().with_extension("json.bak");
            std::fs::rename(manager.path(), &backup)?;
            manager.load_or_default()
        }
        Err(error) => Err(error),
    }
}
```

The moved-aside `.bak` file carries the same sensitivity as the original
settings file — if the application's settings can hold anything sensitive,
apply the same handling (and, on Unix, the same permission expectations
described in [Save behavior](save-behavior.md#permissions-on-unix)) to the
backup copy as well.
