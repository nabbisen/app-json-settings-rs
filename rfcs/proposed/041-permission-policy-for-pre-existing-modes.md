# RFC 041 — Permission policy for pre-existing loose modes

**Status.** Proposed
**Tracks.** Durability and safety of the default save path.
**Touches.** `src/core/save.rs`, `src/core/tests.rs`, `docs/src/save-behavior.md`, `docs/src/operational-contract.md`, `CHANGELOG.md`.
**Amends.** [RFC 029](../done/029-permission-preservation-on-atomic-save.md)
**Relates to.** [RFC 024](../done/024-save-reliability-and-atomic-write-policy.md), which made atomic save the default; [RFC 042](./042-config-error-variant-stability.md), which carries this RFC's deferred reporting work.
**Handoff.** [implementation handoff](../handoffs/041-permission-policy-for-pre-existing-modes/implementation-handoff.md) — slice 1 only.

## Summary

RFC 029 made atomic save preserve the target file's mode, deliberately in both
directions. A consequence not considered at the time: preservation also
perpetuates a dangerously loose mode indefinitely. A settings file at `0666`
keeps `0666` through every subsequent save, where without preservation a single
save would have returned it to `0600`.

This RFC does not propose reversing RFC 029. It proposes stopping the
propagation of the bits that are never deliberate, and separately considering an
opt-in owner-only guarantee for applications that want one.

Reporting such a condition to the caller — rather than repairing it silently —
was considered here and **moved to [RFC 042](./042-config-error-variant-stability.md)**,
because it requires a new `ConfigError` variant and cannot ship before a major
version. Keeping it here would have prevented this RFC from ever closing
honestly.

## Motivation

### What RFC 029 decided, and what it did not consider

RFC 029 is explicit that preservation runs both ways. Its testing section
required "a settings file at `0644` retains `0644` after an atomic save —
preservation works in both directions; this is not 'always restrict'." That test
exists today (`src/core/tests.rs`, `mode-preserve-0644`). This behavior is
specified, intended, and covered.

What RFC 029 did not consider is the *asymmetry of consequence*. Preserving
`0644` respects a user who widened the file deliberately. Preserving `0666` also
respects the widening — but `0666` on a settings file is essentially never
deliberate, and preservation converts a transient bad state into a permanent one.
Without `apply_target_mode`, the next save would repair it, because the
replacement carries the temporary file's `0600`.

The crate therefore does not merely decline to improve an insecure mode. It
actively prevents the self-repair that would otherwise happen.

Measured (2026-08-12), with `apply_target_mode` disabled:

```
restrictive target (0400): target before = 400 -> after replacement = 600
loose target      (0644): target before = 644 -> after replacement = 600
world-writable    (0666): target before = 666 -> after replacement = 600
```

and as shipped:

```
loose target   (0644): target before = 644 -> after replacement = 644
world-writable (0666): target before = 666 -> after replacement = 666
```

### Why this surfaced now

Raised by the orbok team on 2026-08-12, correcting a claim this project had
published. Their write path forces `0600` unconditionally and does not preserve;
they asked whether preservation was the right default given that the success
path is where loosening occurs. The `apply_target_mode` rustdoc's "never less
restrictive" note describes the *failure* path only, which read as a stronger
guarantee than it is.

### Scope is wider than per-user config

`with_root_dir()` and `at_custom_dir()` let a caller point the manager at any
directory, including shared and system locations. RFC 029's reasoning leaned on
settings being "per-user state stored under a per-user directory." That is the
common case, not the guaranteed one, and it matters for any option that would
impose a mode.

## Goals

* Stop propagating permission bits that are never deliberate for a settings file.
* Preserve the deliberate cases RFC 029 protects, including `0644`, unchanged.
* Give applications that need a hard owner-only floor a way to ask for one,
  without imposing it on applications that do not.
* Change nothing on Windows.

## Non-goals

* **No reversal of RFC 029.** Preservation stays the default. The `0644`
  preservation test stays green and unmodified.
* **No unconditional enforcement of `0600`.** See
  [Alternatives](#alternatives-considered).
* No Windows ACL work. RFC 029's reasoning stands unchanged.
* No ownership (uid/gid) handling.
* No claim that this crate is a secret store.
* **No reporting of the condition to the caller.** Moved to
  [RFC 042](./042-config-error-variant-stability.md); see
  [Alternatives](#alternatives-considered).
* No new dependency, and no logging or tracing dependency.

## Design

Two slices, independently shippable, in the order they should be considered.

### Slice 1 — Stop propagating group and other **write** bits

`apply_target_mode` copies the target's mode wholesale. Instead, mask off the
group-write and other-write bits (`0o022`) before applying, so those bits are
never carried forward from a pre-existing file.

| Target mode | Today | Proposed |
|---|---|---|
| `0600` | `0600` | `0600` |
| `0400` | `0400` | `0400` |
| `0644` | `0644` | `0644` — unchanged, deliberate widening respected |
| `0640` | `0640` | `0640` — unchanged |
| `0664` | `0664` | `0644` |
| `0666` | `0666` | `0644` |

Read access is left entirely alone. Only *writability by other users* is
refused, which is the property that lets another local account alter what the
application reads back.

This is the one silent change in this RFC, and it is defensible precisely
because it cannot override a decision anyone plausibly made: a settings file
that other local users may write is not a configuration choice, it is a mistake
or an attack artifact.

The existing `mode-preserve-0644` test is unaffected — `0644` contains no
group- or other-write bits.

Slice 1 repairs the condition. It does not *report* it — see
[Alternatives](#alternatives-considered) for why that work now lives in RFC 042.

### Slice 2 — Opt-in owner-only enforcement

A builder method by which an application that knows its settings are sensitive
can ask for a `0600` floor regardless of what mode it finds:

```rust
ConfigManager::<Settings>::for_app("my-app")?.enforce_owner_only()
```

Additive, no new error variant, no change for callers who do not use it.

**RFC 029 rejected a similar builder** (`with_file_mode(0o600)`) on the grounds
that it "adds API for an edge case and leaves the silent-widening default in
place for everyone who does not call it — the default is the bug." That
reasoning was sound in its context, where the default *lost* a deliberate
`0600`. It does not transfer: after slice 1 the default is no longer a bug, and
this method serves a requirement the default deliberately declines to impose.

The obvious objection — that a caller can `chmod` after `save()` and needs no
API — does not hold, and by this project's own standard. We argued to orbok that
`OpenOptionsExt::mode` beats create-then-`set_permissions` because the latter
leaves a window in which the file exists at the wrong mode. Caller-side `chmod`
after save has exactly that window. Applying the standard consistently, this
cannot be done correctly from outside the crate.

Enforcement must be documented as **best-effort**: `set_permissions` fails
silently on filesystems that do not model permission bits, so the floor is not
absolute and the documentation must say so rather than imply a guarantee.

## Compatibility

* **Slice 1** is a behavior change on Unix: files at `0664`/`0666`/`0620` and
  similar are narrowed on the next save. Minor release, not a patch. No API
  change.
* **Slice 2** is purely additive. Minor release.
* Windows, other targets, and `SaveMode::Direct` unaffected throughout.
* No `ConfigError` variant is added by any slice as proposed.

## Security considerations

Modest severity, and narrower than RFC 029's. This does not close an exposure
the crate creates — the crate never creates a group- or other-writable file. It
stops the crate from perpetuating one that something else created.

The realistic scenario is a settings file loosened by an unrelated process, a
careless `chmod`, a bad umask in an installer, or an archive extracted with
permissive modes. Today that state survives every save for the life of the file.

This does not make the crate a secret store, and the documentation should
continue to direct applications with real secret-handling requirements to a
platform keychain.

## Testing and verification

Unix-only, `#[cfg(unix)]`:

* `0666` target is narrowed to `0644` on the next save.
* `0664` target is narrowed to `0644`.
* `0644` target is preserved unchanged — the existing `mode-preserve-0644` test,
  which must stay green and must not be modified.
* `0640` target is preserved unchanged: group *read* is not touched.
* `0400` and `0600` targets are preserved unchanged.
* Slice 2, if taken: an enforced manager lands at `0600` from a `0644` target,
  and a non-enforcing manager on the same file does not.

Each test must be shown to fail with the mask removed, per the standard set by
the RFC 033 fixes and task 008 — a test that cannot fail is not evidence.

Windows and macOS stay green on the existing matrix; no Windows behavior changes.

## Risks and unresolved questions

* **Slice 1 is silent.** It repairs without telling anyone, which is the same
  class of behavior this project has been reducing elsewhere. Accepted only
  because the bits involved cannot represent a real decision — and because
  [RFC 042](./042-config-error-variant-stability.md) carries the work to address
  the silence properly once it can be done cleanly.
* **A legitimate group-writable deployment would be narrowed.** A shared
  service directory where a group is genuinely expected to write the settings
  file would break. **The owner accepted this risk on 2026-08-12 and approved
  slice 1 to ship.** Slice 2 does not help such a deployment — it would need
  `SaveMode::Direct` or its own path. Release notes must state the narrowing
  plainly so an affected deployment can recognise itself.
* **The mask is a policy encoded in a constant.** `0o022` is a judgement about
  which bits are never deliberate. It should be a named constant with the
  reasoning attached, not an inline literal.
* **The condition remains unreported** until RFC 042's major lands. Repaired,
  but never surfaced — a real if minor cost of the deferral, recorded there.

## Alternatives considered

* **Enforce `0600` unconditionally** (the orbok posture). Rejected as a default.
  It reverses a decision the user made, silently, on every save, so the user
  cannot win by re-applying their own mode. It contradicts the direction that
  produced `try_new()`. Its guarantee is soft — `set_permissions` is best-effort
  — so it would advertise a floor it silently fails to deliver on exactly the
  filesystems where that matters. It would require modifying RFC 029's
  `mode-preserve-0644` test, which is the clearest signal that it reverses a
  specified decision rather than refining one. Offered instead as opt-in, slice 2.
* **Report the condition rather than repair it silently** — the `ssh` posture,
  which refuses a too-open private key and does not `chmod` it for you. Not
  rejected on the merits; it is the better answer. It requires a dedicated
  `ConfigError` variant, and `ConfigError` is not `#[non_exhaustive]`
  (`src/core/error.rs`; `src/core.rs:94` records the consequence), so it cannot
  ship before a major version. Reporting through `ConfigError::Platform` with a
  message was considered and rejected: it repeats the `try_new()` workaround and
  makes string-matching the only way a caller can discriminate. Failing `load`
  outright was rejected as worse than the condition — it strands an application
  over a state it did not cause. **Moved to
  [RFC 042](./042-config-error-variant-stability.md)** rather than retained here,
  so this RFC can close on the work it delivers instead of waiting on an
  unscheduled major. Owner decision, 2026-08-12.
* **Do nothing, document only.** Already partly done — the behavior is now
  documented in `docs/src/operational-contract.md`. Rejected as the whole answer:
  documentation does not stop the crate from perpetuating the state, and the
  perpetuation is the part that is ours.
* **Mask read bits too** (narrow `0644` to `0600`). Rejected: that is
  unconditional enforcement wearing a mask, and it overrides the deliberate case
  RFC 029 exists to protect.
* **Repair only on first load rather than every save.** Rejected: more surface,
  more surprise, and it leaves the file loose for any application that saves
  without loading first.

## Acceptance criteria

Slice 1:

* Group-write and other-write bits are never propagated from a pre-existing
  target.
* Read bits, and all owner bits, are propagated unchanged.
* The mask is a named constant carrying its rationale.
* The existing `mode-preserve-0644` test is green and unmodified.
* New tests cover `0666`, `0664`, `0640`, `0600`, and `0400`, each demonstrated
  to fail with the mask removed.
* `docs/src/save-behavior.md` and `docs/src/operational-contract.md` document the
  refusal, including that it overrides a group-writable deployment.
* `CHANGELOG.md` records the behavior change under a minor version.
* No public API change, no new `ConfigError` variant.

Slice 2, if taken:

* Additive builder method, documented as best-effort with the
  non-POSIX-filesystem caveat stated.
* Tests covering enforced and non-enforced behavior on the same target mode.
* No change to the default path.

This RFC is complete when slice 1 has shipped and slice 2 has been either
shipped or declined. It does **not** wait on RFC 042.
