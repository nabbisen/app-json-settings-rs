# RFC 030 — Operational contract: concurrency and corrupted files

**Status.** Implemented (v2.5.0)
**Tracks.** Documented behavior under concurrent access and invalid stored data.
**Touches.** `docs/src/` (new page plus `SUMMARY.md`), `docs/src/error-handling.md`.
**Handoff.** [implementation handoff](../handoffs/030-operational-contract-concurrency-and-corruption/implementation-handoff.md)

## Summary

Document two behaviors the crate already has but has never stated: what happens
when two processes write the same settings file, and what happens when the file
exists but is not valid JSON. Documentation only — no API, no behavior change.

This RFC absorbs the scope originally sketched as RFC 031 (corrupted-file
recovery guidance). RFC 031 was a roadmap placeholder and no file was ever
created for it, so no withdrawal is required; the number is retired unused.

## Motivation

Both behaviors are defensible. Neither is written down.

An application developer choosing this crate needs to know whether concurrent
writes are safe, and what to do when a user hand-edits the settings file and
breaks it. Today they have to read the source to find out. That is exactly the
kind of gap that produces a bug report against the library for behavior that was
intentional.

Merging the two topics is deliberate: both answer "what does this crate do when
the environment is not ideal, and what must the application handle?" That is how
the documentation would be organized regardless, and two separate docs-only RFCs
would cost more ceremony than the granularity is worth.

## Goals

* State the concurrency contract precisely, including what atomic save does and
  does not protect against.
* State what `load()` and `load_or_default()` do with an unreadable file, and why
  neither silently discards it.
* Give a worked recovery pattern an application can copy.
* Keep every claim verifiable against current behavior.

## Non-goals

* **No file locking.** See below.
* No automatic reset, repair, or backup of corrupted files.
* No new API, no new error variants, no behavior change of any kind.
* No new dependency.
* Not a general guide to multi-process architecture.

## Content

### Concurrency

The contract, stated precisely:

* **Reads are safe against concurrent writes.** Since v2.3.0 the default save
  replaces the file by atomic rename, so a reader sees either the complete
  previous content or the complete new content — never a partial file. This is a
  real guarantee and worth stating, because it is the failure most people fear.
* **Writes are not coordinated.** `update()` is an unlocked read-modify-write:
  it loads, applies the closure, and saves. Two processes interleaving will lose
  one of the two updates. Last writer wins.
* **There is no cross-process lock**, and the crate will not acquire one.
* An application needing coordinated writes should serialise them itself — a
  single writer, an in-process mutex where one process owns the file, or
  application-level locking.

`SaveMode::Direct` does not provide the torn-read protection; that is already
implied by RFC 024 but should be explicit here.

**Why no locking.** Advisory locking means `flock` on Unix and `LockFileEx` on
Windows — more hand-written platform FFI, in a project that has just spent a
release recovering from exactly that (RFC 027). Advisory locks are also not
honoured across all filesystems, notably some network mounts, so the guarantee
would be conditional in ways that are hard to state honestly. The crate's stated
audience is small desktop, CLI, and local-first apps, which are overwhelmingly
single-writer. If a concrete multi-writer need appears, it can be revisited with
a real use case behind it.

### Corrupted or externally modified files

Current behavior, which the documentation must state rather than change:

* `load()` on a file that is not valid JSON returns `ConfigError::Deserialize`.
* `load_or_default()` creates defaults **only when the file is absent**
  (`NotFound`). A file that exists but does not parse returns
  `ConfigError::Deserialize` — it does **not** silently reset.
* A file that parses as JSON but does not match `T` is also a `Deserialize`
  error. This includes the common case of a settings struct gaining a
  non-`Option` field with no `#[serde(default)]`.

That last point deserves prominence: the most likely corruption in practice is
not a mangled file, it is a schema change in the application's own struct. The
documentation should say so and point at `#[serde(default)]` as the mitigation,
because that is the fix most applications actually need.

**Why `load_or_default()` does not reset.** Silently replacing unreadable data
with defaults destroys user state with no signal. A user who hand-edits their
settings and makes a typo would lose the rest of their configuration. The crate
surfaces the error and lets the application decide — which is the only party that
knows whether the data is precious.

### Recovery pattern

Provide a worked example an application can copy: match on
`ConfigError::Deserialize`, move the unreadable file aside to a `.bak` path,
proceed with defaults, and tell the user. The example must compile under
`cargo test --doc`.

## Compatibility

None affected. Documentation only.

## Security considerations

None introduced. The recovery pattern should note that a `.bak` copy of a
settings file inherits the same sensitivity as the original — relevant alongside
[RFC 029](./029-permission-preservation-on-atomic-save.md).

## Testing and verification

* Documentation examples compile under `cargo test --doc`.
* Every behavioral claim is checked against current behavior before publication.
  Existing tests already cover the invalid-JSON path
  (`load_reports_invalid_json_as_deserialization_error`); the claim about
  `load_or_default()` not resetting on invalid JSON is **not** currently covered
  by a test and one should be added, because this RFC turns it into a documented
  promise.

Adding that test is the one code change this RFC implies. It is a test, not
behavior.

## Risks and unresolved questions

* **Documenting a behavior makes it a commitment.** After this lands, changing
  `load_or_default()` to reset on corruption becomes a breaking change. That is
  intended — the current behavior is right — but it should be a conscious
  trade rather than a side effect.
* **"Last writer wins" may read as a defect** to someone skimming. The wording
  should present it as a deliberate contract with a stated rationale, not an
  apology.

## Alternatives considered

* **Two separate RFCs** (concurrency, corruption). Rejected: both are docs-only
  and share a frame; the ceremony would exceed the benefit.
* **Add advisory locking now.** Rejected above.
* **Add `load_or_reset()` or a `RecoveryPolicy` enum.** Rejected for now: it is
  API surface for a case the recovery pattern already covers in a few lines, and
  an automatic-reset API invites silent data loss. Revisit if applications ask.

## Acceptance criteria

* A documentation page states the concurrency contract, including the atomic-read
  guarantee, the lost-update behavior, and the no-locking decision with rationale.
* The same page or a sibling states corrupted-file behavior, including that
  `load_or_default()` does not reset, and why.
* The schema-change case is called out with `#[serde(default)]` as mitigation.
* A compiling recovery example is included.
* `docs/src/SUMMARY.md` lists the new page.
* A test covers `load_or_default()` returning `Deserialize` for an existing
  invalid file.
* No public API change, no behavior change.
