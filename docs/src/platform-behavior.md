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
