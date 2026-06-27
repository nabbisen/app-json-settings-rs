# Executable examples

The crate keeps a deliberately small example set. The goal is to show common
application workflows without turning examples into a second documentation tree.

Run examples from the project root:

```sh
cargo run --example basic
cargo run --example custom_root
cargo run --example update
```

## `basic`

Shows a small GUI-style settings type and first-run default creation. The
production desktop constructor is `ConfigManager::for_app("my-gui-app")`; the
example uses a temporary root so running it does not write to your real app
settings directory.

## `custom_root`

Shows `with_root_dir()` for portable apps, tests, sandboxed hosts, and UWP-style
apps where the host application resolves its own storage root.

## `update`

Shows the read-modify-write path with `update()`. This is the usual flow for GUI
preferences such as launch counters, theme changes, recent-file lists, and window
state updates.

## Maintenance rule

Examples should demonstrate user workflows, not every API surface. New examples
should be added only when they cover a distinct common task that cannot be made
clearer in the existing examples.
