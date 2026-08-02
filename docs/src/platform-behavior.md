# Platform behavior

The default desktop storage root is selected by platform.

| Platform | Base directory |
|---|---|
| Windows desktop | `%APPDATA%` |
| macOS | `~/Library/Application Support` |
| Linux / Unix | `$XDG_CONFIG_HOME` or `~/.config` |

`ConfigManager::for_app("my-app")` appends the app name to this base directory.

`ConfigManager::new()` appends the current executable stem instead. This is easy
for examples, but less stable than an explicit app name.

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

Since v2.5.0, atomic save preserves the existing settings file's permission
bits on Unix and creates new files owner-only (`0600`). See
[Save behavior](save-behavior.md#permissions-on-unix) for the details.

**Windows is unchanged.** Windows access control uses security descriptors
rather than mode bits, and per-user `%APPDATA%` is already restricted to that
user by directory ACL inheritance; the temporary file is created in that same
directory and inherits the same protection. This reasoning follows from the
documented Windows inheritance model — **it has not been verified empirically
against a real Windows security descriptor**, and should not be read as a
measured claim.
