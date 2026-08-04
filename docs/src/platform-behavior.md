# Platform behavior

The Rust snippets on this page are illustrative fragments, not compiled or
run by CI — see [Testing guide](testing.md#verification-boundary).

**Supported targets.** The crate builds for Unix (including macOS) and
Windows targets. Storage-root resolution has no fallback branch for any
other target family, so the crate does not compile there at all — mentions
of "other targets" elsewhere on this page and in
[Save behavior](save-behavior.md) describe `SaveMode`'s replacement-strategy
fallback for a hypothetical future target, not a configuration that exists
today.

The default desktop storage root is selected by platform.

| Platform | Base directory |
|---|---|
| Windows desktop | `%APPDATA%` |
| macOS | `~/Library/Application Support` |
| Linux / Unix | `$XDG_CONFIG_HOME` or `~/.config` |

`ConfigManager::for_app("my-app")` appends the app name to this base directory.

`ConfigManager::new()` appends the current executable stem instead. This is easy
for examples, but less stable than an explicit app name.

### The executable-name fallback can collide

If the current executable's name cannot be determined, or is not a safe
path component, `ConfigManager::new()` falls back to the literal name
`"app"` rather than failing. This fallback is a fixed constant, so **any
two executables that both hit it resolve to the same settings file** —
one can silently read and overwrite the other's settings. This is a
correctness and data-integrity hazard, not a privilege-boundary one: both
applications still run as the same user, in that user's own configuration
directory, with no escalation involved.

Since 2.6.0, `ConfigManager::try_new()` reports this failure instead of
falling back — see the [API guide](api-guide.md#choosing-a-constructor)
for the full comparison across constructors.

### Resolution failure

Since 2.5.0, resolving the base directory can fail: on Unix (excluding
macOS) when neither `XDG_CONFIG_HOME` nor `HOME` is set, on macOS when `HOME`
is not set, and on Windows when `%APPDATA%` is not set. This is uncommon on
desktop systems but can happen in services or containers run without a user
environment.

`ConfigManager::for_app()` and, since 2.6.0, `ConfigManager::try_new()`
both report this as `ConfigError::Platform` with a message naming the
missing variable. `ConfigManager::new()` cannot report it without breaking
its signature, so it falls back to the current directory instead — see the
[API guide](api-guide.md) for the constructor comparison.
`ConfigManager::at_current_dir()` has its own, unrelated fallback to `"."`,
which is not surprising there because the caller explicitly asked for
working-directory storage.

Applications that hit resolution failure in practice should supply a path
explicitly:

```rust
let manager = ConfigManager::<Settings>::new().with_root_dir(chosen_path);
```

## Sandboxed hosts

Sandboxed hosts should usually resolve their own app-local data directory and
pass it with `with_root_dir()`.

This keeps the crate simple and prevents normal desktop users from paying for
platform-specific dependencies they do not need.


## Save replacement behavior

`SaveMode::Atomic` uses platform-specific replacement primitives.

* Unix-like platforms use same-directory `rename` replacement.
* Windows uses an internal `MoveFileExW` wrapper with replace-existing and
  write-through flags.
* Other targets do not claim replacement of an existing file as atomic. Use
  `SaveMode::Direct` unless a target-specific replacement implementation is added.

## File permissions

Since 2.5.0, atomic save preserves the existing settings file's permission
bits on Unix and creates new files owner-only (`0600`). See
[Save behavior](save-behavior.md#permissions-on-unix) for the details.

**Windows is unchanged.** Windows access control uses security descriptors
rather than mode bits, and per-user `%APPDATA%` is already restricted to that
user by directory ACL inheritance; the temporary file is created in that same
directory and inherits the same protection. This reasoning follows from the
documented Windows inheritance model — **it has not been verified empirically
against a real Windows security descriptor**, and should not be read as a
measured claim.
