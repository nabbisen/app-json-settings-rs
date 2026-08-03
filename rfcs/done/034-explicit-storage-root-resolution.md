# RFC 034 — Explicit storage root resolution failure

**Status.** Implemented (v2.5.0)
**Tracks.** Honest failure reporting from the production constructor.
**Touches.** `src/core/dir.rs`, `src/core.rs`, `src/core/tests.rs`, `docs/src/platform-behavior.md`, `docs/src/api-guide.md`, `docs/src/migration-v2.md`, `CHANGELOG.md`.
**Handoff.** [implementation handoff](../handoffs/034-explicit-storage-root-resolution/implementation-handoff.md)

## Summary

`ConfigManager::for_app()` returns `Result`, but that `Result` reports only an
invalid application name. When the platform configuration directory cannot be
resolved, the crate silently substitutes a relative path instead of reporting it.
Make resolution failure an error that `for_app()` propagates.

## Motivation

`default_config_dir()` cannot fail. When `HOME` is unset on Unix, `home_dir()`
returns `PathBuf::from(".")`; when `%APPDATA%` is unset on Windows, the same
substitution happens. So `for_app("my-app")` resolves to
`$CWD/.config/my-app/settings.json` and reports success.

The precise problem is not that a developer might overlook something. It is
**false coverage**:

* `for_app()` returns `Result`.
* A conscientious developer handles that error and reasonably concludes they have
  covered "the library could not determine where to store settings."
* They have not. The `Result` carries only `InvalidPathComponent` — a statement
  about *their input*, never about the environment.

So careful error handling looks complete while the failure it appears to cover
passes through silently. That is worse than an unhandled error, because it
defeats the developer's own diligence.

The observable consequence: a service writes settings relative to its working
directory. If that directory is writable, the settings persist somewhere
transient and are lost on restart or relocation. The user reports that settings
do not save, and nothing ever errored.

**Severity is modest.** It requires an environment without `HOME` or `%APPDATA%` —
systemd units without `User=`, some container configurations. Desktop
applications, the crate's stated audience, essentially never hit it. No user has
reported it; it was found during architectural review.

## Goals

* `for_app()` reports storage-root resolution failure instead of substituting a
  path.
* Make the existing `Result` mean what developers already assume it means.
* Add no new public API.
* Give affected users a documented, correct alternative.

## Non-goals

* No new constructor, builder option, or policy enum.
* No change to `with_root_dir()`, which is explicitly caller-supplied and correct
  as-is.
* No change to path validation.
* No attempt to guess a better fallback directory.

## Design

Make the internal resolver fallible and propagate:

* `default_config_dir()` returns `Result<PathBuf>`, failing when the platform
  base directory cannot be determined.
* `for_app()` propagates that as `ConfigError::Platform`, alongside the existing
  `InvalidPathComponent`.
* The error message names the missing variable — `HOME`/`XDG_CONFIG_HOME` on
  Unix, `%APPDATA%` on Windows — and points at `with_root_dir()`.

### Why this option

Four approaches were weighed:

| | Approach | Weight | Problem |
|---|---|---|---|
| **A** | Fallible resolver; `for_app()` propagates | No new API | Behavior change |
| B | `for_app_strict()` / `require_platform_dir()` | New API | Safe only if opted into; the misleading path stays the default |
| C | Inspection method, e.g. `root_source()` | Small | Still silent unless the caller knows to ask |
| D | `MissingRootPolicy` parameter | Heaviest | Enum and constructor variant for an edge case |

**A is chosen** because it is simultaneously the lightest and the safest, which is
unusual and worth stating. It adds nothing to the API surface — it makes the
`Result` that already exists finally carry the information callers assume it
carries. B and C both leave the unsafe behavior as the default; a safety property
that must be opted into does not protect the people who most need it.

### What does not change

`ConfigManager::new()` returns `Self` and cannot propagate an error without an
API break. It keeps the existing fallback. This asymmetry becomes a documented
distinction rather than a hidden trap: `new()` is the convenience constructor,
`for_app()` is the production one — which the documentation already says.

`at_current_dir()` likewise returns `Self` and keeps its
`unwrap_or_else(|_| ".")` behavior. Its intent is explicitly the working
directory, so the fallback is far less surprising there, but it should be
documented.

## Compatibility

**This is a behavior change, not a signature change.** Existing code compiles
unchanged. Code running where the config directory resolves normally — every
desktop environment — sees no difference.

Code running without `HOME`/`%APPDATA%` moves from "silently writes to the
working directory" to "returns `ConfigError::Platform`."

**The risk, stated plainly:** an application deliberately or accidentally relying
on the working-directory fallback will start failing. The escape hatch already
exists and is the correct API for that intent:

```rust
let manager = ConfigManager::<Settings>::new().with_root_dir(chosen_path);
```

An application that wants working-directory storage should say so explicitly
rather than obtain it as a side effect of a failed lookup.

Minor release. Not a patch.

## Migration and release notes

Required, because a downstream application is already pinned to this crate and
upgrading across this change must not be a surprise.

* `CHANGELOG.md`: state plainly that `for_app()` now returns
  `ConfigError::Platform` where it previously fell back, name the affected
  environments, and show the `with_root_dir()` alternative.
* `docs/src/migration-v2.md`: a section for this version covering who is affected
  (services and containers without `HOME`/`%APPDATA%`), how to detect it (the new
  error), and what to do.
* `docs/src/platform-behavior.md`: document resolution per platform and what
  happens when it fails.
* `docs/src/api-guide.md`: state the `new()` versus `for_app()` distinction
  explicitly, including that only `for_app()` reports resolution failure.

## Security considerations

Minor improvement. Settings written to an unexpected working directory may land
somewhere with weaker permissions than the intended per-user config directory, or
somewhere world-readable such as a shared temporary directory. Failing instead of
guessing removes that path. Interacts with
[RFC 029](./029-permission-preservation-on-atomic-save.md), which governs the
mode of the file once written.

## Testing and verification

Environment-variable manipulation is process-global, so these tests must not run
concurrently with others that read the same variables. Prefer a single serialised
test, or a subprocess, over `set_var` scattered across parallel tests.

* With `HOME` and `XDG_CONFIG_HOME` unset on Unix, `for_app()` returns
  `ConfigError::Platform`.
* With the environment resolvable, `for_app()` behaves exactly as before.
* `with_root_dir()` continues to work with the environment unset — this is the
  documented escape hatch and must be proven.
* `new()` still falls back rather than panicking.
* Existing tests pass unchanged.

## Risks and unresolved questions

* **Silent-fallback users break.** Accepted, mitigated by release notes and
  `with_root_dir()`.
* **`new()` remains inconsistent with `for_app()`.** Accepted for this RFC:
  fixing it requires an API break, which is an owner decision and does not belong
  in a minor release. Worth revisiting if `new()` is ever reconsidered.
* **Test isolation.** Environment mutation is global; poorly isolated tests here
  would be flaky across the CI matrix. Called out above.

## Alternatives considered

Options B, C, and D above. Also considered and rejected: keeping the fallback but
logging a warning — the crate has no logging dependency and should not acquire
one for this.

## Acceptance criteria

* `default_config_dir()` is fallible.
* `for_app()` returns `ConfigError::Platform` when resolution fails, with a
  message naming the missing variable and pointing at `with_root_dir()`.
* `new()` and `at_current_dir()` are unchanged in behavior and documented.
* Tests cover resolution failure, the normal path, and the `with_root_dir()`
  escape hatch, without cross-test environment interference.
* `CHANGELOG.md` and `docs/src/migration-v2.md` cover the change for upgraders.
* `docs/src/platform-behavior.md` and `docs/src/api-guide.md` updated.
* No public API signature change.
