# Implementation handoff — RFC 041 slice 1, refuse to propagate write bits

**Governing RFC.** [RFC 041](../../done/041-permission-policy-for-pre-existing-modes.md)
**Status.** Inherited from RFC 041 (Implemented, 2.8.0).
**Scope.** Slice 1 only. Slice 2 (`enforce_owner_only()`) was declined by the owner and never implemented.

## Purpose

Stop `apply_target_mode()` carrying group-write and other-write bits forward
from a pre-existing settings file, so a file that something else left at `0666`
returns to `0644` on the next save instead of keeping `0666` for the life of the
file.

## Background

Read RFC 041, and RFC 029 which it amends. In short:

RFC 029 made atomic save preserve the target's mode, **deliberately in both
directions** — its acceptance criteria required a test proving `0644` survives,
"this is not 'always restrict'." That test exists and is
`mode-preserve-0644` in `src/core/tests.rs`.

What RFC 029 did not weigh is that preservation also makes a dangerous mode
permanent. `rename` would otherwise hand the file the temporary file's `0600`,
so without `apply_target_mode` the next save repairs it. Measured with the call
disabled:

```
restrictive target (0400): target before = 400 -> after replacement = 600
loose target      (0644): target before = 644 -> after replacement = 600
world-writable    (0666): target before = 666 -> after replacement = 600
```

So the crate does not merely decline to improve an insecure mode — it prevents
the repair that would otherwise happen. That is the part that is ours, and the
part this slice fixes.

**This is a refinement of RFC 029, not a reversal.** If you find yourself
changing `mode-preserve-0644`, stop — see Prohibited shortcuts.

## Change scope

| File | Change |
|---|---|
| `src/core/save.rs` | Mask constant; `apply_target_mode()` applies it |
| `src/core/tests.rs` | Mode tests |
| `docs/src/save-behavior.md` | Document the refusal |
| `docs/src/operational-contract.md` | Amend the existing preservation section |
| `CHANGELOG.md` | Entry under the existing `## Unreleased` |

## Non-change scope

* **`mode-preserve-0644`.** Must stay green and byte-identical. It is the
  regression guard proving this refines RFC 029 rather than reversing it.
* **Read bits and all owner bits.** Untouched. `0640` stays `0640`, `0644` stays
  `0644`, `0400` stays `0400`.
* **`ConfigError`.** No new variant — see prohibited shortcuts.
* **No public API.** `enforce_owner_only()` is slice 2 and is not approved.
* `create_temp_file()`'s `0600` creation mode, the temp-naming scheme, the retry
  loop, `replace_file()`, `sync_parent_dir()`.
* `SaveMode::Direct`, Windows, and non-Unix targets.

## Required implementation

In `src/core/save.rs`, a named constant carrying its reasoning — **not an inline
literal**:

```rust
/// Permission bits never propagated from a pre-existing settings file.
///
/// Group-write and other-write let another local account alter what the
/// application reads back. Unlike a deliberate widening of *read* access
/// (`0644`, `0640`), which RFC 029 preserves on purpose, these bits cannot
/// represent a configuration decision anyone plausibly made: they are the
/// residue of a bad umask, a careless `chmod`, or an archive extracted with
/// permissive modes. Preserving them would make that state permanent, because
/// `rename` would otherwise return the file to the temporary file's `0600`.
#[cfg(unix)]
const NON_PROPAGATED_MODE_BITS: u32 = 0o022;
```

`apply_target_mode()` masks the target's mode before applying it. Keep it
best-effort and silent on failure, exactly as today.

Resulting behavior:

| Target | Today | Required |
|---|---|---|
| `0600` | `0600` | `0600` |
| `0400` | `0400` | `0400` |
| `0640` | `0640` | `0640` |
| `0644` | `0644` | `0644` |
| `0664` | `0664` | `0644` |
| `0666` | `0666` | `0644` |
| `0620` | `0620` | `0600` |

## Prohibited shortcuts

* **Do not modify `mode-preserve-0644`.** If it fails, your mask is wrong —
  `0644` contains no bits in `0o022`. The test is correct.
* **Do not mask read bits.** Narrowing `0644` to `0600` is unconditional
  enforcement, which RFC 041 rejects as a default and the owner did not approve.
* **Do not add a `ConfigError` variant** to report the condition. It is a
  breaking change (`ConfigError` is not `#[non_exhaustive]`), and reporting is
  deferred to [RFC 042](../../proposed/042-config-error-variant-stability.md) by
  owner decision. Slice 1 repairs silently and that is intended for now.
* **Do not make the mask configurable.** No builder, no parameter, no feature
  flag.
* Do not hard-fail when `chmod` fails. RFC 029's fail-secure reasoning is
  unchanged: the temp file is already `0600`.

## Required tests

In `src/core/tests.rs`, `#[cfg(unix)]`:

* `0666` target → `0644` after save.
* `0664` target → `0644` after save.
* `0620` target → `0600` after save.
* `0640` target → `0640`, unchanged. **This is the test that proves you masked
  write and not read.**
* `0400` target → `0400`, unchanged.
* `mode-preserve-0644` — existing, unmodified, green.

Test count goes 39 → 44 unit.

## Required evidence

* **Each new test demonstrated to fail with the mask removed.** Set
  `NON_PROPAGATED_MODE_BITS` to `0o000`, show the failures, revert. A test that
  cannot fail is not evidence — the standard from RFC 033 and task 008.
* The `0640` test's failure output specifically.

  **Corrected after implementation (2026-08-12).** This bullet originally
  claimed a mask of `0o077` or `0o066` "would pass every other test and
  silently break RFC 029's deliberate case." That is false, and was caught by
  the implementer. Both masks also break `mode-preserve-0644`, because `0640`
  and `0644` share their group digit and a mask stripping group-read fails both
  at once.

  Stronger still, established during review 018: `0640`'s set bits are a strict
  subset of `0644`'s, so **`0640` can never fail unless `0644` fails too**,
  while a mask touching only other-read (`0o004`) fails `0644` alone. The
  pre-existing `0644` test strictly dominates the `0640` one. Keep both — the
  `0640` test documents that group *read* survives, where a reader will look
  for it — but the decisive evidence is `0644`, not `0640`.
* `git diff --stat` limited to the five files above.
* `git status` clean after reverting the mask experiment.
* CI green on all three platforms.

## Documentation

`docs/src/operational-contract.md` already has "Preservation can leave the file
looser than the crate would create it," written when this was current behavior.
**Amend it** — do not add a second section that contradicts the first. It must
now say group/other-write bits are refused, that read widening is still
preserved, and that the condition is repaired without being reported.

`CHANGELOG.md` has an `## Unreleased` section already. Add there; do not create
a version heading.

The entry must state the narrowing plainly enough that **a deployment relying on
a group-writable settings file can recognise itself.** The owner accepted that
breakage knowingly; the release notes are how an affected user finds out. Such a
deployment needs `SaveMode::Direct` or its own write path.

## Escalate rather than decide

* `mode-preserve-0644` fails.
* The mask appears to need to differ per platform.
* Any test requires touching `src/core.rs`, `error.rs`, or the public API.
* You conclude the mask should include read bits.

## Review request

Per §9.2, as a file package at
`.git-exclude/review-request/018-rfc-041-slice-1-write-bit-refusal/README.md`.

Lead with the `0640` mask-removed failure. That is the one distinguishing a
correct mask from one that passes by accident.
