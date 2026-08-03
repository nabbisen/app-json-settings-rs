# RFC 035 — Documentation currency and recovery example

**Status.** Implemented (2.5.1)
**Tracks.** Accuracy of user-facing documentation, and example coverage of failure paths.
**Touches.** `README.md`, `docs/src/testing.md`, `docs/src/error-handling.md`, `docs/src/examples.md`, `examples/`.
**Amends.** [RFC 026](../done/026-minimal-executable-examples.md) — the example maintenance rule.
**Handoff.** [implementation handoff](../handoffs/035-documentation-currency-and-recovery-example/implementation-handoff.md)

## Summary

Bring the user-facing documentation back in line with what the crate actually
does after 2.5.0, restructure the README's documentation links into reader paths,
and add a fourth executable example covering the failure path that 2.5.0 made
prominent.

## Motivation

2.5.0 changed observable behavior twice — permission preservation and
`for_app()` reporting storage-root resolution failure — and added a
documentation page. The README and two documentation pages were not updated to
match. None of this is wrong in a way that misleads about *correctness*, but the
project has repeatedly committed to documentation that does not outrun or lag
reality, and this is lagging.

Findings from the architect's survey:

1. **README's "More detail" list is stale and incomplete.** It links 7 of 13
   pages and omits `operational-contract.md` — the page 2.5.0 added. Also absent:
   `introduction.md`, `error-handling.md`, `migration-v2.md`, `testing.md`,
   `maintainer-notes.md`.
2. **README's "Features / design notes" predates 2.5.0.** Neither permission
   preservation nor `for_app()`'s new error reporting appears.
3. **README's "Why / when" oversells.** It offers "OS-default config locations
   for desktop apps" with no hint that resolution can fail — reproducing at
   README level the same false-coverage shape RFC 034 was written to remove from
   the API.
4. **No MSRV statement in the README.** The crate declares `1.85.0` and verifies
   it in CI; a reader must open `Cargo.toml` to discover it.
5. **`testing.md`'s test inventory is stale.** It enumerates 12 covered
   behaviors; there are now 26 unit tests, and the three added in 2.5.0 are
   missing.
6. **`error-handling.md` gives `ConfigError::Platform` a single table row.**
   After 2.5.0 that variant has a specific meaning and a specific remedy.
7. **No example demonstrates a failure path.** The corrupted-settings recovery
   pattern exists only as prose in `operational-contract.md`, and is one of the
   28 documentation code blocks that nothing compiles.

## Goals

* Every user-facing claim matches 2.5.0 behavior.
* README links organized as reader paths, per the project's documentation-persona
  rule, rather than a flat file list.
* A runnable example covering recovery from an unreadable settings file.
* Reduce the rate at which this documentation goes stale again.

## Non-goals

* **No compile-verification of documentation examples.** The 28 unchecked blocks
  in `docs/src/` are a separate decision with real cost trade-offs; they get their
  own RFC.
* **No API-surface work.** `at_current_dir()`'s absence from the API guide,
  Windows reserved names, and docs.rs metadata belong to RFC 032.
* **No `uwp` disposition.** Whether to verify, mark experimental, or deprecate is
  a compatibility decision and gets its own RFC.
* No public API change, no behavior change, no new dependency.
* No restructuring of the `docs/src/` page set. Pages get corrected, not
  reorganized.

## Design

### README

Keep the existing six-section structure required by the project rules. Within it:

* **Why / when** — state that `for_app()` reports failure when the platform
  config directory cannot be resolved, and that `with_root_dir()` covers that
  case. One clause, not a paragraph; the README should not become a manual.
* **Features / design notes** — add permission preservation and `for_app()`'s
  error reporting. Remove nothing that is still true.
* **MSRV** — state the declared minimum Rust version, noting it is verified in
  CI.
* **More detail** — replace the flat path list with three short reader paths,
  matching the project's documentation-persona rule:
  * *Getting started* — introduction, quick start, executable examples
  * *Using the crate* — storage model, save behavior, API guide, error handling,
    operational contract, platform behavior, UWP
  * *Contributing and maintaining* — testing guide, migration, maintainer notes

  Every page appears in exactly one path. A page that appears nowhere is the bug
  this replaces.

### `testing.md`

The enumerated inventory of covered behaviors went stale within one release, and
will do so again after every release that adds a test. **Replace the enumeration
with categories plus a pointer to the test modules.**

This is a deliberate design decision, not a shortcut: a list that must be edited
every time a test is added is a maintenance trap, and a stale list is worse than
no list because a reader trusts it. Categories change rarely; test counts change
constantly.

Keep the existing guidance on temporary roots and no-default-feature builds,
which remains accurate.

### `error-handling.md`

Give `ConfigError::Platform` prose alongside its table row: what produces it
(`for_app()` when the platform configuration directory cannot be resolved, and
the optional UWP resolver), and what to do about it (`with_root_dir()` with an
explicit path). Cross-reference `migration-v2.md`, which already carries the
upgrade guidance.

### Fourth example

Add `examples/recovery.rs`, demonstrating the pattern documented in
`operational-contract.md`: attempt to load, catch `ConfigError::Deserialize`,
move the unreadable file aside, continue with defaults, and report to the user.

It must write only to a temporary root, per RFC 026's existing rule that examples
never touch real application data directories. It should deliberately create an
invalid settings file so that running it actually exercises the recovery branch
rather than silently taking the happy path — an example of error handling that
never errors teaches nothing.

`examples/README.md` and `docs/src/examples.md` gain matching entries.

### Amendment to RFC 026

RFC 026 states: *"New examples should be added only when they cover a distinct
common task that cannot be made clearer in the existing examples."*

This RFC does not overturn that rule — it records that recovery from an
unreadable settings file **meets** it, and why the judgement changed:

* When RFC 026 was written, corrupted-file behavior was undocumented and
  effectively an edge case.
* RFC 030 made it a documented contract, including a specific pattern the
  application is expected to implement.
* An expected-of-the-caller pattern that exists only as uncompiled prose is
  exactly the case the rule's "distinct common task" clause is meant to admit.

The rule itself stands unchanged for future examples.

## Compatibility

None affected. Documentation and one new example target. No API, behavior, or
dependency change.

## Security considerations

None introduced. The recovery example moves a settings file aside; it should note
that the `.bak` copy carries the same sensitivity as the original, consistent
with `operational-contract.md` and RFC 029.

## Testing and verification

* `cargo test --examples` compiles the new example — this is genuine CI coverage,
  unlike `docs/src/` prose.
* The example is **run**, not merely compiled, and its output shows the recovery
  branch executing.
* Every factual claim added to the README and the two pages is checked against
  the crate before it is written, not transcribed from this RFC.
* Existing tests unchanged; this RFC adds none.

## Risks and unresolved questions

* **A fourth example raises maintenance cost**, which is precisely what RFC 026
  guarded against. Mitigated by the amendment above being narrow: it admits one
  example on stated grounds rather than loosening the rule.
* **The README grows.** The project rules require it stay concise. The reader
  paths replace an existing list rather than adding a section, and the new
  factual material is a few clauses, so net growth should be small. If the
  README becomes long, detail moves to `docs/src/`, not the reverse.
* **This RFC fixes staleness but does not prevent it.** Only the `testing.md`
  change is structurally anti-stale. A broader answer — compile-checking
  documentation — is deliberately a separate RFC.

## Alternatives considered

* **Do it without an RFC**, as maintenance. Rejected: the fourth example relaxes
  a constraint RFC 026 established, and a handoff that silently amends an
  accepted RFC is an anti-pattern RFC 000 names explicitly.
* **Fold this into RFC 032.** Rejected: 032 is API-surface work with its own
  decisions; combining them would produce one RFC with two unrelated
  justifications.
* **Keep `testing.md`'s enumerated list and just update it.** Rejected: it went
  stale in a single release, and updating it re-arms the same trap.

## Acceptance criteria

* README states MSRV, `for_app()`'s failure mode, and 2.5.0's behavior changes.
* README's "More detail" is replaced by three reader paths, and every
  `docs/src/` page appears in exactly one.
* `testing.md` describes categories with a pointer to the test modules, not an
  enumeration.
* `error-handling.md` gives `ConfigError::Platform` prose with cause and remedy.
* `examples/recovery.rs` exists, writes only to a temporary root, and exercises
  the recovery branch when run.
* `examples/README.md` and `docs/src/examples.md` list it.
* `cargo test --examples` passes.
* No API change, no behavior change, no new dependency.
* Every added claim verified against the crate.
