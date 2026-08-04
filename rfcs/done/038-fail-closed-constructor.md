# RFC 038 — Fail-closed constructor for executable-derived identity

**Status.** Implemented (2.6.0)
**Tracks.** Data integrity of the default storage root.
**Touches.** `src/core.rs`, `src/core/dir.rs`, `src/core/dir/tests.rs`, `src/core/tests.rs`, `docs/src/api-guide.md`, `docs/src/platform-behavior.md`, `CHANGELOG.md`.
**Handoff.** [implementation handoff](../handoffs/038-fail-closed-constructor/implementation-handoff.md)
**Relates to.** [RFC 034](../done/034-explicit-storage-root-resolution.md) — same
false-coverage shape, other constructor.

## Summary

Add `ConfigManager::try_new() -> Result<Self>`, a fail-closed counterpart to
`new()`, and document `new()`'s two silent fallbacks — including the fact that one
of them can put two different applications on the same settings file.

Additive. No behavior change to any existing method.

## Motivation

`ConfigManager::new()` performs two silent substitutions:

| When | Substitution | Documented? |
|---|---|---|
| Platform config directory unresolvable | `.` (working directory) | Yes — RFC 034 added it |
| Executable name underivable or unsafe | the literal `app` | **No** |

The second is undocumented and is the more serious of the two.

### The fallback collides

`default_runtime_app_name()` falls back to a fixed constant. Any two applications
that hit it therefore resolve to the **same** settings file. Measured against
2.5.1, with two different binaries whose executable stems fail
`is_safe_path_component()`:

```
orbok:editor           -> ~/.config/app/settings.json
totally:different-app  -> ~/.config/app/settings.json
```

One application can silently read and overwrite another's settings. This is a
data-integrity hazard, not merely an observability one.

### Why this surfaced

A downstream consumer — a local-first desktop application on 2.0.3 — reported
that `new()`'s fallback defeats their fail-closed profile design: they freeze one
settings path for the process lifetime, and a silent substitution splits their
settings away from their other directories while every other path stays correct.
They noted, correctly, that 2.0.3's panic was *better* for them because it was
loud.

They did not report the collision. It was found while investigating their report.

### Why `for_app()` is not the whole answer

For most consumers it is. `for_app("name")` takes an explicit identity, so no
derivation happens and there is nothing to fall back from, and it reports
storage-root resolution failure. The reporting consumer's case is a one-line
change to `for_app()`.

But it does not cover a caller who genuinely *wants* the executable's name — a
tool distributed under more than one binary name, for instance — and needs that
derivation to be fail-closed. Today there is no such option.

## Goals

* A caller who wants executable-derived naming can obtain it fail-closed.
* `new()`'s fallbacks are documented, including the collision.
* No breaking change.
* No new `ConfigError` variant.

## Non-goals

* **Not changing `new()`'s behavior.** It stays exactly as it is. Changing it
  would be breaking, and the reporting consumer explicitly did not ask for the
  panic back.
* **Not adding a derived-name accessor.** `folder_path()` already exists and its
  final component is the derived name, so an accessor would be convenience over
  existing API. The crate's non-goals keep the surface small.
* **Not adding a logging dependency** to warn when the fallback fires. That would
  cost every consumer a dependency to serve a rare case.
* Not touching `at_current_dir()`. Its `.` fallback is defensible: the caller
  explicitly asked for the working directory, so `.` is that directory rather
  than a substitution for something else.
* **Not adding `#[non_exhaustive]` to `ConfigError`.** Still a major-version
  decision reserved for the project owner.

## Design

### 1. `try_new()`

```rust
pub fn try_new() -> Result<Self>
```

Fails when **either** source of identity is unavailable:

* the platform configuration directory cannot be resolved, or
* the executable name cannot be derived, or does not pass
  `is_safe_path_component()`.

A constructor that closes only one of the two doors is not fail-closed.

### 2. The error variant is constrained by our own past mistake

The natural design is a new `ConfigError` variant for "executable name
underivable". **That is exactly the breaking change [RFC 037](./037-migration-guidance-for-2-0-x-upgraders.md)
exists to disclose** — `ConfigError` is not `#[non_exhaustive]`, so adding a
variant breaks every downstream exhaustive `match`. Doing it in the same cycle as
disclosing that mistake would be indefensible.

So `try_new()` reports both failures as `ConfigError::Platform(String)`, with
messages that distinguish them. `Platform` is a loose semantic fit for "the
executable name is unusable" — it is accepted deliberately, as the only
non-breaking option, and the RFC records that rather than pretending it is ideal.

This is worth noting as a live cost of the un-`#[non_exhaustive]` enum: it is
already constraining new design, not just historical compatibility.

### 3. `new()` is re-expressed, not re-specified

`new()` keeps identical behavior, implemented in terms of `try_new()` with the
fallbacks applied on the error path. The two cannot then drift: any future change
to derivation logic affects both, and the fallback stays visibly a fallback
rather than being spread across two functions.

### 4. The collision cannot be fixed for `new()`

Recorded so it is not re-opened later. The fallback fires precisely when nothing
identifying is available, so there is no unique value to substitute. Any fixed
constant collides; any derived value requires the input that is missing. `new()`
is therefore hazardous by construction, and the remedies available are
`try_new()`, `for_app()`, and documentation.

### 5. A testable seam

Following RFC 034's precedent, extract a private function taking the executable
path rather than reading process state:

```rust
fn app_name_from(exe: Option<PathBuf>) -> Result<String>
```

`current_exe()` cannot be made to fail from inside a test, so without this seam
the fallback path is untestable — and an untestable failure path is how the
original defect survived.

### 6. Documentation

* `new()`'s rustdoc states **both** fallbacks and the collision, and points at
  `try_new()` and `for_app()`.
* `try_new()`'s rustdoc states what it fails on and why a caller would choose it.
* `docs/src/api-guide.md` gains a short constructor comparison: `new()`
  convenience, `try_new()` fail-closed with derived identity, `for_app()`
  fail-closed with explicit identity, `with_root_dir()` caller-supplied.
* `docs/src/platform-behavior.md` documents the collision under default-root
  resolution.

## Compatibility

* **Additive.** One new method; no signature or behavior change to anything
  existing.
* No new error variant, no new dependency.
* Ships in **2.6.0** — minor, per the project owner's decision.
* Consumers on `new()` are unaffected unless they choose to migrate.

## Security considerations

The collision is a **correctness and data-integrity hazard, not a vulnerability**.
It crosses no privilege boundary: both applications run as the same user, in that
user's own configuration directory, with no escalation and no exposure to another
account. Stating this precisely matters — overstating it would be its own kind of
inaccuracy.

The practical risk is that one application silently overwrites another's settings
and the user loses configuration with no diagnostic.

## Testing and verification

* `app_name_from()` returns the stem for a normal path.
* `app_name_from(None)` — `current_exe()` unavailable — is an error.
* `app_name_from()` with a stem failing `is_safe_path_component()` is an error.
* The error messages distinguish the two failure sources.
* `try_new()` succeeds in the normal test environment.
* `new()` still falls back rather than erroring, on both paths — this is the
  regression guard for "no behavior change".
* No `unsafe`, no environment mutation, no new dependency.

The collision itself is demonstrated in this RFC's Motivation and does not need a
test: it is a consequence of the fixed constant, not a code path that could
regress independently.

## Risks and unresolved questions

* **`ConfigError::Platform` is a loose fit.** Accepted above. If a major version
  ever adds `#[non_exhaustive]`, a dedicated variant becomes available and this
  should be revisited.
* **`new()` remains hazardous.** Documentation is the only available mitigation,
  and documentation is not a control. Anyone who never reads the rustdoc is
  exactly as exposed as before.
* **Adding API to a crate that prizes smallness.** One method, justified by a
  data-integrity hazard with an external report attached. Not a precedent for
  further additions.

## Alternatives considered

* **A dedicated `ConfigError` variant.** Rejected: breaking, and it would repeat
  the exact mistake RFC 037 discloses, in the same cycle.
* **Restore the panic in `new()`.** Rejected: a regression, and panicking in a
  library constructor is reasonably disliked. The reporting consumer did not ask
  for it.
* **Emit a `log`/`tracing` warning on fallback.** Rejected: a dependency for
  every consumer to serve a rare case, and the crate has no logging dependency
  today.
* **Deprecate `new()`.** Rejected: it is the `Default` implementation and is
  legitimately convenient for examples and tests. Deprecation would warn every
  existing user for a hazard most will never hit.
* **Documentation only.** Rejected by the project owner in favour of this RFC:
  it leaves the hazard live and undetectable by any consumer who wants to guard
  against it.

## Acceptance criteria

* `try_new()` exists, returns `Result<Self>`, and fails on both an unresolvable
  configuration directory and an underivable or unsafe executable name.
* Both failures report `ConfigError::Platform` with distinguishable messages.
* No new `ConfigError` variant.
* `new()`'s behavior is unchanged and is implemented in terms of `try_new()`.
* `app_name_from()` seam exists, is private, and both failure modes are unit
  tested without environment mutation or `unsafe`.
* `new()`'s rustdoc states both fallbacks and the collision.
* `api-guide.md` carries the constructor comparison;
  `platform-behavior.md` documents the collision.
* `CHANGELOG.md` records an additive change for 2.6.0.
* No breaking change, no new dependency.
