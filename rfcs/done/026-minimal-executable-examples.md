# RFC 026 — Minimal executable examples

**Status.** Implemented (2.4.0)
**Tracks.** Documentation, examples, user onboarding.
**Touches.** `examples/`, `docs/src/examples.md`, `README.md`, `.github/workflows/ci.yml`.

## Summary

Add a small set of executable Cargo examples that show common application
workflows without expanding the project into a broad examples gallery.

The examples are:

* `basic` — first-run defaults and simple typed GUI-style settings.
* `custom_root` — caller-provided storage root for portable, test, sandboxed,
  and UWP-style hosts.
* `update` — read-modify-write settings persistence.

Each example is runnable with `cargo run --example <name>`.

## Motivation

The crate is intentionally small. Documentation snippets are useful, but users
also benefit from complete files that can be run and edited. The risk is that
examples become a second documentation system and raise maintenance cost.

This RFC chooses a narrow example set that covers the main workflows while
leaving advanced and platform-specific topics in the documentation.

## Design

Examples live directly under `examples/` as normal Cargo example targets:

```text
examples/
  README.md
  basic.rs
  custom_root.rs
  update.rs
```

The examples use temporary directories instead of real OS application data
locations. This lets users run examples safely without polluting their real app
settings directory. The `basic` example still comments the production desktop
constructor, `ConfigManager::for_app("my-gui-app")`, so users see the intended
production shape.

`docs/src/examples.md` explains when to run each example and states the
maintenance rule: examples demonstrate workflows, not every API surface.

CI runs `cargo test --examples` so the examples remain compile-checked.

## Non-goals

* No GUI-framework-specific examples in this RFC.
* No UWP WinRT example in this RFC. UWP guidance remains in `docs/src/uwp.md`.
* No atomic-save example. Atomic save is default behavior and is documented in
  `docs/src/save-behavior.md`.
* No workspace split or example subprojects. Standard Cargo example targets are
  sufficient at the current project size.

## Compatibility

This RFC adds files and CI coverage only. It does not change the public API,
default features, dependency set, or runtime behavior.

## Implementation notes

The examples should remain short and dependency-free beyond the crate's existing
public dependencies. Future examples should be added only when they cover a
distinct common task that cannot be explained cleanly by one of the existing
examples or the mdBook documentation.
