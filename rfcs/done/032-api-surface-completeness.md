# RFC 032 — API surface completeness

**Status.** Implemented (2.7.0)
**Tracks.** Public API that is documented, and validation that means what callers assume.
**Touches.** `docs/src/api-guide.md`, `Cargo.toml` (docs.rs metadata), and — depending on one decision — `src/core/validation.rs`.
**Handoff.** [implementation handoff](../handoffs/032-api-surface-completeness/implementation-handoff.md)
**Relates to.** [RFC 025](../done/025-path-safety-and-explicit-app-identity.md), which declared OS-specific filename legality a non-goal.

## Summary

Document the public methods that are missing from the API guide, make the
Windows-only API visible on docs.rs, and settle whether `for_app()` should reject
Windows reserved device names.

The first two are mechanical. The third is a scope decision reserved for the
project owner, and this RFC recommends but does not assume it.

## Motivation

### F1 — Seven public methods are absent from the API guide

The roadmap recorded this as one method. Audited against `src/core.rs`, it is
seven:

| Method | In `api-guide.md`? |
|---|---|
| `at_current_dir()` | Absent |
| `at_custom_dir()` | Absent |
| `disable_pretty_json()` | Absent |
| `with_direct_save()` | Absent |
| `path()` | Absent |
| `with_filename()` | Only as a substring of `try_with_filename` |
| `save_mode()` | Only as a substring of `with_save_mode` |

**`path()` is the notable one.** It returns the full settings file path — the
thing a caller reaches for to find where their settings actually live — and it
appears nowhere in the guide. `disable_pretty_json()` is a feature no reader
would discover.

Two of these are deliberate compatibility aliases (`at_custom_dir()`,
`with_filename()`). They should be documented *as* aliases, with the preferred
form named, rather than omitted — a reader encountering them in existing code
currently finds nothing.

### F2 — `for_app()` accepts Windows reserved device names

`is_safe_path_component()` rejects empty strings, `.`, `..`, path separators,
drive separators, and control characters. It does not reject `CON`, `PRN`, `AUX`,
`NUL`, `COM1`–`COM9`, or `LPT1`–`LPT9`, which Windows treats as device names in
any directory and at any case.

Measured — every reserved name passes every validator:

```
name   safe_comp    plain_file   for_app    try_with_filename
CON    true         true         true       true
NUL    true         true         true       true
con    true         true         true       true
```

**The filename case is the serious one.** `try_with_filename("NUL")` returns
`Ok`, and on Windows `NUL` refers to the null device *in any directory*. So
`save()` writes to it and **succeeds while discarding the data**, and `load()`
reads EOF and finds an empty file. That is silent data loss, with no diagnostic —
the same shape as the collision hazard RFC 038 documented.

The directory case is milder: `for_app("CON")` returns `Ok` and then fails at
`create_dir_all` with an `io::Error` that does not mention the real cause. Loud,
but late and confusing.

Both are the false-coverage shape RFC 034 and RFC 038 corrected elsewhere: a
`Result` that appears to cover a class of failure it does not carry.

**This is not a security issue, and should not be written as one.** No privilege
boundary is crossed — same user, that user's own configuration directory, no
escalation and no escape. It is a data-integrity problem.

*Reasoned from the documented Win32 behavior; not verified empirically on
Windows, and the implementation should say so if it documents the mechanism.*

### F3 — The Windows-only API is invisible on docs.rs

docs.rs builds on Linux with default features, so `at_uwp_local_folder()` — gated
`cfg(all(windows, feature = "uwp"))` — does not appear in the published
documentation at all. A reader cannot discover it exists.

## Goals

* Every public method reachable from `docs/src/api-guide.md`.
* Compatibility aliases documented as aliases.
* The Windows-only surface discoverable on docs.rs.
* A decision on reserved device names, recorded either way.

## Non-goals

* **Not full OS-specific filename legality.** RFC 025 declared this a non-goal
  and it stands: trailing dots and spaces, path length limits, per-filesystem
  character sets, and case-collision rules remain out of scope. Reserved device
  names are a closed set of 22, which is why they are separable.
* Not changing any method's behavior beyond the F2 decision.
* Not `folder_path()` — handled by [RFC 039](../done/039-homes-for-consumer-answers.md).
* Not the `uwp` feature's disposition — that is
  [RFC 040](./040-uwp-feature-disposition.md), and F3 stands regardless of its
  outcome, since even a deprecated feature should be visible.

## Design

### 1. Document the seven

Add each to `docs/src/api-guide.md` in the structure the page already uses.
`at_custom_dir()` and `with_filename()` get one line each naming their preferred
replacement, not their own sections — they are compatibility surface, and giving
them equal weight would suggest equal standing.

`path()` belongs with `folder_path()` and `file_name()` in the inspection
grouping RFC 039 established.

### 2. docs.rs metadata

Add to `Cargo.toml`:

```toml
[package.metadata.docs.rs]
all-features = true
targets = ["x86_64-pc-windows-msvc"]
```

Keep the default target as-is so a Windows doc-build failure costs the platform
switcher rather than the whole documentation build. Confirm the shape against
docs.rs's current metadata contract before writing it — this RFC states intent,
not verified syntax.

### 3. Reserved device names — the decision

**Option A — document the limitation.** Add to the API guide and
`platform-behavior.md` that `for_app()` does not reject Windows reserved device
names, and that using one produces a failure at save time on Windows. Zero
behavior change.

**Option B — reject them.** Extend `is_safe_path_component()` to reject the 22
reserved names, case-insensitively, **on every platform**.

The cross-platform part matters. `is_plain_file_name()`'s existing documentation
says validation is deliberately OS-independent *"so tests behave consistently on
Windows, macOS, and Unix."* Rejecting on Windows only would break that principle;
rejecting everywhere preserves it, at the cost of a Linux application being
unable to call itself `con`.

**Decided by the project owner: option B.**

It makes validation mean what callers already assume, which is the failure this
project has corrected twice elsewhere (RFC 034, RFC 038). The set is closed and
small, so it does not reopen RFC 025's non-goal. Decisive factor: option A would
leave `try_with_filename("NUL")` silently discarding settings on Windows, and
documenting a silent-data-loss path is not a resolution.

**The cost, recorded honestly:** it is a behavior change for an application
legitimately named one of those strings on a non-Windows platform. That
application is hypothetical, but the change would be real for it, and it makes
this a minor release rather than a patch.

### 4. Optional — `pub` that is not public

`validate_plain_file_name()`, `validate_path_component()` (in
`src/core/validation.rs`) and `save_to_path()` (in `src/core/save.rs`) are
declared `pub` but live inside the private `core` module and are not re-exported,
so they are unreachable externally. `pub(crate)` would state the actual intent.

Cosmetic, no behavior change, and separable — drop it if the slice is getting
long.

## Compatibility

* F1 and F3: none. Documentation and metadata.
* F2 under option A: none.
* F2 under option B: `for_app()` and `try_with_filename()` reject 22 additional
  names. Minor release, not a patch.
* Item 4: none — the items are already unreachable.

## Security considerations

None. Reserved device names on Windows are a correctness and error-quality
problem, not a security boundary — the failure is a refused directory creation,
not an escape from one.

## Testing and verification

* Every public method in `src/core.rs` cross-checked against `api-guide.md`, by
  enumeration rather than by eye — the audit that produced F1 found seven where
  the roadmap recorded one.
* Under option B: unit tests for the reserved set, case-insensitively, and a test
  that `for_app()` rejects them. Existing validation tests unchanged.
* docs.rs metadata verified against docs.rs's documented contract before it
  ships, not assumed.
* No change to existing behavior beyond the F2 decision.

## Risks and unresolved questions

* **Option B is a behavior change** for a hypothetical application. Stated above
  rather than dismissed.
* **The Windows mechanism is reasoned, not measured.** Nobody on this project can
  run Windows, so any documentation of *why* the failure occurs should carry the
  same epistemic caveat RFC 029 used for its ACL reasoning.
* **docs.rs metadata could fail to build.** Mitigated by keeping the default
  target unchanged.

## Alternatives considered

* **Reject reserved names on Windows only.** Rejected: breaks the deliberate
  cross-platform consistency of the validation helpers, and would make the same
  code path behave differently per platform for no gain.
* **Document only the methods a reader is likely to need.** Rejected — that
  judgement is how the seven accumulated.
* **Fold F3 into RFC 040.** Rejected: docs.rs visibility is worth having whether
  the feature is supported, experimental, or deprecated.

## Acceptance criteria

* All seven methods documented in `api-guide.md`; aliases marked as aliases.
* `path()` grouped with the other inspection methods.
* docs.rs metadata added and its shape verified against docs.rs's contract.
* The F2 decision implemented as chosen, and recorded either way.
* Under option B: reserved names rejected case-insensitively on all platforms,
  with tests.
* No behavior change beyond the F2 decision.
