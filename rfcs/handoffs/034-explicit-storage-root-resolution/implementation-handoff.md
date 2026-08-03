# Implementation handoff — RFC 034 Explicit storage root resolution failure

**Governing RFC.** [RFC 034](../../done/034-explicit-storage-root-resolution.md)
**Status.** Inherited from RFC 034 (Implemented, 2.5.0).
**Milestone.** M2 — release 2.5.0.
**Implementation order.** Second of three, after RFC 029.

## Purpose

Make `for_app()` report that the platform configuration directory could not be
resolved, instead of silently substituting a relative path.

## Background

Read RFC 034. The problem is **false coverage**: `for_app()` returns `Result`,
but that `Result` today reports only an invalid application name. A developer who
handles the error reasonably believes they have covered "the library could not
work out where to store settings." They have not — that case never surfaces.

## Sequencing

**After RFC 029.** Both touch `CHANGELOG.md` and `docs/src/platform-behavior.md`;
running them concurrently conflicts in the files that carry the upgrade story.

## Change scope

| File | Change |
|---|---|
| `src/core/dir.rs` | Fallible resolution with a testable seam |
| `src/core.rs` | `for_app()` propagates; `new()` keeps falling back |
| `src/core/dir/tests.rs` | **New file.** Resolver tests |
| `src/core/tests.rs` | Behavior tests through the public API |
| `docs/src/platform-behavior.md` | Per-platform resolution and its failure |
| `docs/src/api-guide.md` | `new()` versus `for_app()` distinction |
| `docs/src/migration-v2.md` | Upgrade section |
| `CHANGELOG.md` | 2.5.0 entry |

## Non-change scope

* `ConfigManager::new()`'s **signature and fallback behavior**. It returns `Self`
  and cannot propagate without an API break. It keeps falling back.
* `at_current_dir()`'s behavior. Document it; do not change it.
* `with_root_dir()` — caller-supplied and correct as-is. It must keep working
  with the environment unresolvable; that is the documented escape hatch.
* Path-component validation and `InvalidPathComponent`.
* The `uwp` resolver.
* Any public API signature. No new constructor, builder option, or policy enum.
* The save path — that is RFC 029's territory.

## Required implementation

### 1. A testable seam, not environment mutation in tests

**Do not write tests that mutate process environment variables.** In edition
2024 `std::env::set_var` is `unsafe` — verified:

```
error[E0133]: call to unsafe function `set_var` is unsafe and requires unsafe block
```

Writing `unsafe` in tests to work around that is not acceptable here, and
environment mutation is process-global so it would race the parallel test
harness across the CI matrix.

Instead, split resolution into a pure inner function that takes an environment
lookup, with the public-facing wrapper supplying the real one:

```rust
fn config_dir_from(getenv: impl Fn(&str) -> Option<OsString>) -> Result<PathBuf>
fn default_config_dir() -> Result<PathBuf>   // delegates, using std::env::var_os
```

Both are private. This is an internal seam, not a public API change, and it makes
every case testable with no `unsafe`, no environment mutation, and no flakiness.

### 2. Fallible resolution

`config_dir_from` returns `Err(ConfigError::Platform(..))` when the platform base
directory cannot be determined:

* Unix (non-macOS): neither `XDG_CONFIG_HOME` nor `HOME` is set.
* macOS: `HOME` is not set.
* Windows: `APPDATA` is not set.

The message must name the missing variable and point the caller at
`with_root_dir()`. Someone reading this error in a log should be able to act on
it without reading our source.

### 3. Propagate from `for_app()` only

`for_app()` returns `ConfigError::Platform` on resolution failure, alongside the
existing `InvalidPathComponent`.

`new()` keeps today's fallback — it cannot do otherwise. Implement that
explicitly (fall back when resolution fails) rather than leaving it to chance, so
the asymmetry is visible in the code and not an accident.

## Prohibited shortcuts

* **Do not use `unsafe { std::env::set_var(..) }` in tests.** Use the seam.
* Do not change `new()`'s signature to return `Result`. That is an API break and
  an owner decision, explicitly out of scope.
* Do not make `new()` panic when resolution fails.
* Do not guess an alternative directory — not `/tmp`, not the executable's
  directory, not a hard-coded path. Failing is the point.
* Do not add a logging dependency to warn instead of failing. RFC 034 considered
  and rejected that.
* Do not "improve" `at_current_dir()` while nearby.
* Do not touch `src/core/save.rs`.

## Required tests

### In `src/core/dir/tests.rs` (new file)

Add `#[cfg(test)] mod tests;` to `src/core/dir.rs`, following the project
convention. Drive `config_dir_from` with a stub lookup — no real environment
involved:

* Unix: `XDG_CONFIG_HOME` set → that path is used.
* Unix: `XDG_CONFIG_HOME` unset, `HOME` set → `$HOME/.config`.
* Unix: both unset → `Err(ConfigError::Platform)`.
* The error message names the missing variable.

Gate per-platform cases with `#[cfg(...)]` so the suite is meaningful on all
three runners.

### In `src/core/tests.rs`

* `for_app()` succeeds normally in the test environment, exactly as before.
* `with_root_dir()` produces a working manager regardless of resolution — this is
  the documented escape hatch and must be proven, not assumed.
* `new()` does not panic.

Existing tests must pass unchanged.

## Required documentation updates

`docs/src/platform-behavior.md`: resolution per platform, what happens when it
fails, and that only `for_app()` reports it.

`docs/src/api-guide.md`: state the `new()` versus `for_app()` distinction
explicitly — `new()` is the convenience constructor and falls back; `for_app()`
is the production constructor and reports failure.

`docs/src/migration-v2.md`: a **2.4.x → 2.5.0** section covering who is affected
(services and containers without `HOME`/`%APPDATA%`), how it shows up (the new
`ConfigError::Platform`), and the fix (`with_root_dir()` with an explicit path).

`CHANGELOG.md`: state plainly that `for_app()` now returns an error where it
previously fell back, name the affected environments, show the alternative.

**The migration section is mandatory, not optional.** A downstream application is
already pinned to this crate. Upgrading across a silent-to-loud behavior change
must not be a surprise.

## Compatibility constraints

* No signature change; existing code compiles unchanged.
* Behavior change: code in an unresolvable environment moves from "silently
  writes to the working directory" to "returns `ConfigError::Platform`".
* Minor release, not a patch.
* Everything in a normal desktop environment is unaffected.

## Security constraints

Minor improvement — settings no longer land in an unexpected working directory
that may be world-readable. Do not extend this into validating or hardening
caller-supplied roots; `with_root_dir()` is explicitly trusted by design.

## Known risks

* **Applications relying on the working-directory fallback will start failing.**
  Accepted by the project owner. Mitigated by the migration guide and by
  `with_root_dir()`, which is the correct API for that intent.
* **Test isolation** is the main implementation hazard, addressed by the seam. If
  you find yourself reaching for environment mutation, stop and re-read step 1.

## Required evidence

* Test output for every resolver case, including the failure message text.
* Proof that no test mutates process environment — `grep -rn "set_var" src/`
  returning nothing.
* Full CI matrix green, including macOS and Windows.
* `git diff --stat` confirming the change scope.
* The rendered migration section, quoted in the review request.

## Acceptance criteria

* `config_dir_from` (or equivalent seam) exists, is private, and is unit-tested
  without environment mutation.
* `for_app()` returns `ConfigError::Platform` on resolution failure, with a
  message naming the missing variable and pointing at `with_root_dir()`.
* `new()` and `at_current_dir()` are behaviorally unchanged and documented.
* No `unsafe` anywhere in the change or its tests.
* `with_root_dir()` proven to work when resolution would fail.
* Migration section and changelog entry present.
* CI green on all three platforms.

## Required review-request content

Per §9.2, as a file package under `.git-exclude/review-request/NNN-slug/README.md`.

## Escalate rather than decide

* The seam cannot be built without changing a public signature.
* A platform's resolution rules turn out to differ from the RFC's description.
* Making `for_app()` fallible breaks an existing test in a way that suggests the
  behavior change is wider than described.
* `new()`'s fallback appears untenable and an API break looks necessary — that is
  an owner decision, not an implementation one.
