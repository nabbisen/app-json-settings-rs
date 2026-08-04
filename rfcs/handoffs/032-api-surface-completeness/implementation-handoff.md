# Implementation handoff — RFC 032 API surface completeness

**Governing RFC.** [RFC 032](../../proposed/032-api-surface-completeness.md)
**Status.** Inherited from RFC 032 (Proposed).
**Milestone.** M3 — slice 3.

## Purpose

Document the seven public methods missing from the API guide, make the
Windows-only API visible on docs.rs, and stop validation accepting names that
cannot work on Windows.

## Background

Read RFC 032. The decision inside it is settled: **option B — reject Windows
reserved device names.** The project owner chose it because option A would leave
`try_with_filename("NUL")` silently discarding settings on Windows, and
documenting a silent-data-loss path is not a resolution.

This slice contains the first code change since RFC 038.

## Change scope

| File | Change |
|---|---|
| `src/core/validation.rs` | Reject reserved device names |
| `src/core/validation/tests.rs` | **New file** — validation unit tests |
| `src/core/tests.rs` | Constructor-level rejection tests |
| `docs/src/api-guide.md` | The seven undocumented methods |
| `docs/src/platform-behavior.md` | The reserved-name rule |
| `Cargo.toml` | docs.rs metadata |
| `CHANGELOG.md` | Entry for the next minor |

## Non-change scope

* **Full OS-specific filename legality.** RFC 025's non-goal stands: trailing
  dots and spaces, path length limits, per-filesystem character sets, and
  case-collision rules stay out. Reserved device names are separable *because*
  they are a closed set.
* **`ConfigError` — do not add a variant.** Reserved names report through the
  existing `InvalidPathComponent`. Same constraint as RFC 038, same reason.
* `folder_path()` — documented by RFC 039. Do not duplicate it.
* The `uwp` feature's disposition — RFC 040, still open. F3 here is only about
  docs.rs visibility and applies regardless of that outcome.
* Any constructor's behavior beyond what the new validation rejects.
* CI configuration.

## Required implementation

### 1. Reject reserved device names in `src/core/validation.rs`

Extend `is_safe_path_component()` to reject the Windows reserved device names:

```
CON  PRN  AUX  NUL
COM1 … COM9
LPT1 … LPT9
```

Twenty-two names. Three requirements, each of which is easy to miss:

**Case-insensitive.** Windows treats `con`, `Con`, and `CON` identically. Compare
case-insensitively rather than listing variants.

**On every platform, not just Windows.** `is_plain_file_name()`'s own
documentation says validation is deliberately OS-independent *"so tests behave
consistently on Windows, macOS, and Unix."* A Windows-only rule would break that
and make the same code path behave differently per platform. The cost is that a
Linux application cannot call itself `con`, which is acceptable.

**Match the stem, not the whole string.** Windows treats `NUL.txt` as the null
device too — the reserved name is the portion before the first `.`. Checking only
the whole string would let `try_with_filename("nul.json")` through, which is
exactly the silent-data-loss case this slice exists to close. **This is the
requirement most likely to be missed.**

Both `for_app()` and `try_with_filename()` inherit the fix, since both route
through this function.

### 2. docs.rs metadata in `Cargo.toml`

Make the Windows-only API visible. Intent:

```toml
[package.metadata.docs.rs]
all-features = true
targets = ["x86_64-pc-windows-msvc"]
```

**Verify the key names and semantics against docs.rs's current metadata
documentation before writing it.** The snippet above states intent, not confirmed
syntax — I have not validated it against their contract.

Keep the default target unchanged, so a failed Windows doc build costs the
platform switcher rather than the whole documentation build.

### 3. Document the seven in `docs/src/api-guide.md`

| Method | Treatment |
|---|---|
| `at_current_dir()` | Own entry |
| `disable_pretty_json()` | Own entry |
| `with_direct_save()` | Own entry |
| `save_mode()` | Own entry |
| `path()` | Group with `folder_path()` and `file_name()` in the inspection section RFC 039 established |
| `at_custom_dir()` | **One line**, naming `with_root_dir()` as preferred |
| `with_filename()` | **One line**, naming `try_with_filename()` as preferred |

The last two are compatibility surface. Giving them full sections would suggest
equal standing with the checked forms; omitting them entirely leaves a reader who
meets them in existing code with nothing. One line each is the point.

### 4. Document the rule in `docs/src/platform-behavior.md`

State that reserved device names are rejected on all platforms, and why — they
cannot work on Windows, where the name refers to a device rather than a file.

**Describe it as correctness, not security.** No privilege boundary is involved.
If you document the Windows mechanism, carry the same caveat RFC 029 used for its
ACL reasoning: reasoned from documented behavior, not verified here.

## Prohibited shortcuts

* **Do not add a `ConfigError` variant** for reserved names, however much better
  it would fit. Use `InvalidPathComponent`.
* Do not make the check `#[cfg(windows)]`.
* Do not extend into trailing dots, trailing spaces, length limits, or character
  sets. That is RFC 025's non-goal and it stands.
* Do not check only the whole string — see the stem requirement above.
* Do not touch `folder_path()`'s documentation.
* Do not describe the change as a security fix.

## Required tests

New `src/core/validation/tests.rs`, following the project convention:

* Every one of the 22 names rejected, upper and lower case.
* `nul.json`, `CON.txt` — reserved stem with an extension — rejected.
* **Names that merely start with a reserved string are accepted**: `console`,
  `nullable`, `communications`, `printer`. This is the regression guard against
  a prefix match, and it matters more than the rejection tests.
* Existing valid names still accepted — `settings.json`, `my-app`.

In `src/core/tests.rs`:

* `for_app("CON")` returns `Err(InvalidPathComponent)`.
* `try_with_filename("NUL")` returns `Err(InvalidPathComponent)`.
* Existing constructor tests pass unchanged.

## Required documentation updates

Covered above, plus a `CHANGELOG.md` entry for the next minor release: validation
now rejects reserved device names, which is a behavior change, alongside the
documentation and docs.rs additions. State the behavior change plainly and do not
present it as a security fix.

## Compatibility constraints

* Additive for documentation and metadata.
* **Behavior change** for validation: `for_app()` and `try_with_filename()` now
  reject 22 names plus their extension forms. **Minor release, not a patch.**
* No API signature change, no new error variant, no new dependency.

## Security constraints

None. This is correctness and error quality. The failure it prevents is silent
data loss on Windows, within the user's own directory — no boundary is crossed.

## Known risks

* **The stem requirement.** Whole-string matching would pass every rejection test
  above and still leave `nul.json` reachable. The prefix-acceptance tests
  (`console`, `nullable`) are the other half of the guard.
* **A hypothetical non-Windows application named `con`** now fails to construct.
  Accepted by the owner and recorded in the RFC.
* **docs.rs metadata could fail to build.** Mitigated by leaving the default
  target alone.

## Required evidence

* Test output for all validation cases, rejections and acceptances.
* A run showing `for_app("CON")` and `try_with_filename("nul.json")` both
  rejected, and `for_app("console")` accepted.
* An enumeration of every `pub fn` in `src/core.rs` checked against
  `api-guide.md` — the audit that produced this slice found seven where the
  roadmap recorded one, so do it by listing rather than by eye.
* The docs.rs metadata keys, with the source you verified them against.
* `git diff --stat` confirming the change scope.
* CI green on all three platforms.

## Acceptance criteria

* All 22 reserved names rejected, case-insensitively, on every platform.
* Reserved stems with extensions rejected; names merely starting with a reserved
  string accepted.
* Both `for_app()` and `try_with_filename()` reject them, via
  `InvalidPathComponent`.
* All seven methods documented; aliases as one-liners; `path()` grouped with the
  other inspection methods.
* docs.rs metadata added, keys verified against docs.rs's documentation.
* `platform-behavior.md` documents the rule as correctness, not security.
* `CHANGELOG.md` records the behavior change for a minor release.
* No new `ConfigError` variant, no `cfg(windows)` on the check, no new dependency.
* CI green.

## Required review-request content

Per §9.2, as a file package at
`.git-exclude/review-request/014-rfc-032-api-surface-completeness/README.md`.

Lead with the stem cases — `nul.json` rejected, `console` accepted. That pair is
what distinguishes a correct implementation from one that passes the obvious
tests.

## Escalate rather than decide

* The reserved-name set turns out to need entries beyond the 22 named here.
* docs.rs's metadata contract does not support the intent in item 2.
* Rejecting reserved names breaks an existing test, which would mean the crate
  depends on one somewhere.
* The stem rule appears to conflict with a legitimate filename in the project's
  own examples or tests.
