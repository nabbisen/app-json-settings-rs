# Implementation handoff — RFC 038 Fail-closed constructor

**Governing RFC.** [RFC 038](../../done/038-fail-closed-constructor.md)
**Status.** Inherited from RFC 038 (Implemented, 2.6.0).
**Milestone.** M3 — slice 6. Ships in 2.6.0.

## Purpose

Give a caller who wants executable-derived naming a fail-closed way to get it,
and document the two silent fallbacks `new()` performs — including the one that
can put two applications on the same settings file.

## Background

Read RFC 038. In short: `new()` substitutes `.` for an unresolvable config
directory and the literal `app` for an underivable executable name. The second is
undocumented and collides — two different binaries in that state resolve to the
same file.

This is the first code change in M3. Slices 1, 2, and 5 were documentation.

## Sequencing

After RFC 037 (slice 5), which discloses the `ConfigError` break this RFC's design
is constrained by. Ahead of slices 3 and 4.

## Change scope

| File | Change |
|---|---|
| `src/core/dir.rs` | `app_name_from()` seam; fallible derivation |
| `src/core.rs` | `try_new()`; `new()` re-expressed; rustdoc |
| `src/core/dir/tests.rs` | Seam tests |
| `src/core/tests.rs` | Constructor behavior tests |
| `docs/src/api-guide.md` | Constructor comparison |
| `docs/src/platform-behavior.md` | The collision |
| `CHANGELOG.md` | 2.6.0 entry |

## Non-change scope

* **`new()`'s observable behavior.** It must still fall back on both paths and
  must not error or panic. This is the regression guard, not a nicety.
* **`ConfigError` — do not add a variant.** See the prohibited shortcuts.
* `for_app()`, `with_root_dir()`, `try_with_filename()`, `at_current_dir()`.
* The `Default` implementation.
* The save path, the `uwp` resolver, CI.
* No new dependency — no `log`, no `tracing`, no `libc`.

## Required implementation

### 1. The seam, in `src/core/dir.rs`

```rust
fn app_name_from(exe: Option<PathBuf>) -> Result<String>
```

Returns the file stem when it exists and passes `is_safe_path_component()`;
otherwise an error. `current_exe()` cannot be made to fail from inside a test, so
without this seam the failure path is untestable — which is how the defect
survived in the first place. Same pattern as RFC 034's `config_dir_from`.

Keep the existing public-facing derivation working by delegating to it.

### 2. `try_new()` in `src/core.rs`

```rust
pub fn try_new() -> Result<Self>
```

Fails if **either** the platform configuration directory is unresolvable **or**
the executable name is underivable or unsafe. Both doors, or it is not
fail-closed.

### 3. Error reporting — read this before writing it

Report **both** failures as `ConfigError::Platform(String)`, with messages that
distinguish them.

**Do not add a `ConfigError` variant**, however much better it would fit. The
enum is not `#[non_exhaustive]`, so a new variant breaks every downstream
exhaustive `match` — the exact mistake RFC 037 discloses. Repeating it in the
same cycle is not an option. The loose semantic fit is accepted deliberately and
is recorded in the RFC.

Make the messages actually distinguishable — someone reading one in a log should
know which of the two failed and what to do.

### 4. `new()` re-expressed

Implement `new()` in terms of `try_new()`, applying the fallbacks on the error
path: `.` for the directory, `app` for the name.

**Behavior must be byte-for-byte what it is today.** The point is that derivation
logic cannot drift between the two constructors, not that anything changes.

### 5. Documentation

`new()`'s rustdoc must state:

* both fallbacks — the existing directory one and the currently undocumented
  `app` one;
* that the `app` fallback is a fixed constant, so two applications that both hit
  it share a settings file;
* pointers to `try_new()` and `for_app()`.

`try_new()`'s rustdoc: what it fails on, and why a caller would choose it over
`new()` or `for_app()`.

`docs/src/api-guide.md`: a short constructor comparison — `new()` convenience,
`try_new()` fail-closed with derived identity, `for_app()` fail-closed with
explicit identity, `with_root_dir()` caller-supplied.

`docs/src/platform-behavior.md`: the collision, under default-root resolution.

**On tone:** describe the collision as a correctness and data-integrity hazard.
It crosses no privilege boundary — same user, that user's own config directory,
no escalation. Do not write it as a security vulnerability; overstating it is its
own inaccuracy.

## Prohibited shortcuts

* **Do not add a `ConfigError` variant.** The single most tempting move here.
* Do not change `new()`'s behavior, deprecate it, or make it panic.
* Do not add a logging dependency to warn on fallback.
* Do not add a derived-name accessor — `folder_path()` already covers it and RFC
  038 excludes it.
* Do not mutate process environment or `current_exe()` in tests. Use the seam.
* Do not write `unsafe`.
* Do not touch `at_current_dir()`.

## Required tests

In `src/core/dir/tests.rs`, driving the seam directly:

* Normal path returns the stem.
* `None` — `current_exe()` unavailable — is an error.
* A stem failing `is_safe_path_component()` is an error.
* The two error messages are distinguishable from each other.

In `src/core/tests.rs`:

* `try_new()` succeeds in the normal test environment.
* **`new()` still falls back rather than erroring.** This is the regression guard
  for "no behavior change" — do not skip it because it looks trivial.

No test is needed for the collision itself: it follows from the constant, not
from a code path that could regress independently.

## Compatibility constraints

Additive only. One new method. No signature change, no behavior change, no new
error variant, no new dependency. Minor release, 2.6.0.

## Security constraints

None introduced. The collision this documents crosses no privilege boundary; see
the tone note above.

## Known risks

* **Scope pull toward a new error variant.** It is the better design and it is
  not available. If reporting through `Platform` feels wrong while implementing,
  that reaction is correct — and the answer is still no.
* **`new()` stays hazardous.** Documentation is the only mitigation available;
  do not try to engineer around it inside this slice.

## Required evidence

* Test output for all six tests above.
* `grep -rn "set_var\|unsafe" src/` showing nothing new.
* The two error message strings, quoted, so the reviewer can judge whether they
  are genuinely distinguishable.
* `git diff` of `new()` alongside a statement that its behavior is unchanged, and
  the passing regression test that shows it.
* CI green on all three platforms.

## Acceptance criteria

* `try_new()` exists and fails on both sources.
* Both failures are `ConfigError::Platform` with distinguishable messages.
* No new `ConfigError` variant, no new dependency, no `unsafe`.
* `new()`'s behavior is unchanged and it is implemented in terms of `try_new()`.
* `app_name_from()` is private and both failure modes are tested without
  environment mutation.
* `new()`'s rustdoc states both fallbacks and the collision.
* `api-guide.md` has the constructor comparison; `platform-behavior.md`
  documents the collision.
* `CHANGELOG.md` records an additive 2.6.0 change.
* CI green.

## Required review-request content

Per §9.2, as a file package at
`.git-exclude/review-request/011-rfc-038-fail-closed-constructor/README.md`.

Lead with the `new()`-unchanged evidence — that is the claim the additive-only
compatibility statement rests on.

## Escalate rather than decide

* Reporting through `ConfigError::Platform` turns out to be genuinely unworkable
  rather than merely inelegant.
* `new()` cannot be expressed in terms of `try_new()` without changing behavior.
* The seam requires a public API change.
* Another silent substitution surfaces while you are in `dir.rs`.
