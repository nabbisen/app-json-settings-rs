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

## A restrictive mode does not protect the file from replacement

The same `rename` has a second consequence, and this one can be mistaken for
a safety mechanism. Restricting the settings file's own permissions does
**not** stop `save()` from replacing it. `rename` is gated by write
permission on the *directory*, not on the file being replaced. Measured on
Unix against a settings file at mode `000`:

```
target set to 000 -> readable? false
save over the 000 target -> true
content now: {   "v": 1 }
```

The file could not be read, and was replaced anyway. Making the *directory*
read-only is what actually prevents the save:

```
save with the DIRECTORY read-only -> false
content after that attempt: {   "v": 42 }
```

The mode *does* survive that replacement — but that is this crate's doing,
not `rename`'s. `rename` repoints the directory entry at the temporary
file's inode, so the replaced file's mode departs with the replaced inode and
the result would otherwise carry the temporary file's `0600`. `save_atomic()`
calls `apply_target_mode()` to copy the target's mode onto the temporary file
first, which is the only reason the mode is preserved. Measured with that
call disabled:

```
restrictive target (0400): target before = 400 -> after replacement = 600
loose target      (0644): target before = 644 -> after replacement = 600
```

So the contents do not survive, and the mode survives only because
preservation is implemented. A mode that blocks reading is not a lock against
overwriting.

Anyone reimplementing atomic replacement gets facts 1 and 2 from `rename`
whether or not they intend them; they get mode preservation only by writing
it.

This matters mainly for the assumption behind it: `chmod` on the settings
file is not a way to pin settings against modification by an application
using this crate. If a settings file must not be replaced, the directory is
the level to control, and the resulting `save()` failure surfaces as
`ConfigError::Io`.

## Preservation can leave the file looser than the crate would create it

Mode preservation copies the target's mode, but not wholesale. New files are
created owner-only at `0600`. An existing file's *read* widening is
preserved — `0644`, set by a user, or created by a version of this crate
predating owner-only creation, keeps that mode through every subsequent
save. Its group-write and other-write bits, if any, are not:

```
loose target        (0644): target before = 644 -> after replacement = 644
group/other-writable (0666): target before = 666 -> after replacement = 644
```

The crate never tightens a mode it did not create, for read access. That is
deliberate: preserving a read-widened mode respects a user who set one on
purpose, and silently overriding it would be its own surprise. Group-write
and other-write are different: they let another local account alter what
the application reads back, and unlike a widened read mode, they cannot
represent a configuration decision anyone plausibly made — they are the
residue of a bad umask, a careless `chmod`, or an archive extracted with
permissive modes. The crate refuses to propagate them, so a file that
picked up `0666` or `0664` from something else returns to `0644` on its
next save rather than staying loose for the life of the file.

This refusal is silent — repaired, not reported. The caller gets no
indication that a target's mode was narrowed; `save()` still just succeeds.

The practical consequence: **do not infer `0600` from the creation default.**
If the settings file predates owner-only creation, or had its *read* access
widened at any point, it stays that way for the life of the file. An
application whose settings hold anything sensitive and that needs
owner-only guaranteed should assert or set the mode itself rather than
relying on the default it was created with — the crate does not enforce
one.

## Recovery pattern

The shape of handling a settings file that fails to deserialize: move it
aside, continue with defaults, and tell the user so they know their
previous settings did not simply vanish. The worked, runnable version lives
in `examples/recovery.rs`.

```rust
match manager.load_or_default() {
    Ok(settings) => settings,
    Err(ConfigError::Deserialize(error)) => {
        eprintln!("settings file is invalid, moving it aside: {error}");
        // Rename to a `.bak` path, then retry `load_or_default()` so the
        // caller gets defaults instead of an error. Full pattern, runnable:
        // examples/recovery.rs (`cargo run --example recovery`).
        todo!()
    }
    Err(error) => return Err(error),
}
```

This is an excerpt — illustrative, not compiled or run by CI (see
[Testing guide](testing.md#verification-boundary)). The complete, runnable
version, including the actual `.bak` rename, lives in
[`examples/recovery.rs`](https://github.com/nabbisen/app-json-settings-rs/blob/main/examples/recovery.rs)
and is compile-checked and run in CI: `cargo run --example recovery`.

The moved-aside `.bak` file carries the same sensitivity as the original
settings file — if the application's settings can hold anything sensitive,
apply the same handling (and, on Unix, the same permission expectations
described in [Save behavior](save-behavior.md#permissions-on-unix)) to the
backup copy as well.
