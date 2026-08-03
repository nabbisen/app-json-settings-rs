# Implementation handoff — RFC 030 Operational contract

**Governing RFC.** [RFC 030](../../done/030-operational-contract-concurrency-and-corruption.md)
**Status.** Inherited from RFC 030 (Implemented, v2.5.0).
**Milestone.** M2 — release 2.5.0.
**Implementation order.** Third of three, after RFCs 029 and 034.

## Purpose

Write down two behaviors the crate already has and has never stated: what happens
when two processes write the same settings file, and what happens when the file
exists but does not parse.

## Background

Read RFC 030. This is documentation plus one test. **No behavior changes.**

The governing constraint for this work, stated up front because it inverts the
usual instinct: **the code is correct and the documentation is missing.** If you
find a discrepancy between what the RFC says the crate does and what it actually
does, the RFC is wrong — report it. Do not change code to match the document.

## Sequencing

**Last of the three.** It is nearly disjoint from 029 and 034, so it carries no
merge risk, but it should describe the crate as it stands after both land.

## Change scope

| File | Change |
|---|---|
| `docs/src/operational-contract.md` | **New page.** Concurrency and corrupted-file behavior |
| `docs/src/SUMMARY.md` | List the new page |
| `docs/src/error-handling.md` | Cross-reference the new page |
| `src/core/tests.rs` | One test (see below) |

## Non-change scope

* **All crate behavior.** `load`, `load_or_default`, `save`, `update` are
  untouched.
* The public API. No `load_or_reset()`, no `RecoveryPolicy`, no new error variant.
* **No file locking.** No `flock`, no `LockFileEx`, no advisory-lock crate.
* No new dependency of any kind.
* `src/core/save.rs` and `src/core/dir.rs` — RFC 029 and 034 territory.
* Other documentation pages, beyond the one cross-reference.

## Required implementation

### 1. The new documentation page

Cover, in this order:

**Concurrency**

* **Reads are safe against concurrent writes.** Since v2.3.0 the default save
  replaces by atomic rename, so a reader sees either the complete old content or
  the complete new content — never a partial file. Lead with this: it is the
  failure people most fear, and the crate genuinely prevents it.
* **Writes are not coordinated.** `update()` is an unlocked read-modify-write.
  Two processes interleaving lose one update. Last writer wins.
* There is no cross-process lock, and the crate will not take one.
* An application needing coordinated writes should serialise them itself.
* `SaveMode::Direct` does not provide the torn-read guarantee.
* **Why no locking**, briefly: advisory locking means more hand-written platform
  FFI in a project that just spent a release recovering from exactly that, and
  advisory locks are not honoured on all filesystems, so the guarantee would be
  conditional in ways that are hard to state honestly.

Present last-writer-wins as a **deliberate contract with a rationale**, not as an
apology or a known defect.

**Corrupted or externally modified files**

* `load()` on invalid JSON returns `ConfigError::Deserialize`.
* `load_or_default()` creates defaults **only when the file is absent**. An
  existing file that does not parse returns `ConfigError::Deserialize` — it does
  **not** silently reset.
* **Give the schema-change case prominence.** The most common real-world
  "corruption" is not a mangled file; it is the application's own settings struct
  gaining a non-`Option` field without `#[serde(default)]`. Say so, and give
  `#[serde(default)]` as the mitigation. This is the case developers actually hit.
* **Why `load_or_default()` does not reset**: silently replacing unreadable data
  with defaults destroys user state with no signal. Only the application knows
  whether that data is precious.

**Recovery pattern**

A worked example the reader can copy: match on `ConfigError::Deserialize`, move
the unreadable file aside to a `.bak` path, continue with defaults, tell the user.

The example must compile under `cargo test --doc`. Note in the text that a `.bak`
copy carries the same sensitivity as the original — relevant alongside RFC 029.

### 2. The test

RFC 030 turns "`load_or_default()` does not reset on invalid JSON" into a
documented promise, and that promise is **not currently covered by a test**. The
existing `load_reports_invalid_json_as_deserialization_error` covers `load()`,
not `load_or_default()`.

Add a test in `src/core/tests.rs`: with an existing file containing invalid JSON,
`load_or_default()` returns `ConfigError::Deserialize`, and **the file is still
on disk with its original content afterwards**. Assert both — the second half is
what proves no silent reset happened.

## Prohibited shortcuts

* **Do not change behavior to match nicer documentation.** If reality and the RFC
  disagree, report it. This is the specific way documentation work goes wrong.
* Do not add locking, in any form, for any platform.
* Do not add `load_or_reset()` or any automatic-recovery API. An automatic-reset
  API invites silent data loss, which is the thing the current design correctly
  refuses.
* Do not add a dependency to make the example nicer.
* Do not write claims you have not checked. Every behavioral statement on the
  page must be verified against the actual crate first.
* Do not soften "last writer wins" into vagueness. State it plainly.

## Required tests

* The new `load_or_default()` test above.
* `cargo test --doc` passes — the recovery example compiles.
* All existing tests pass unchanged. **Any existing test changing behavior means
  something is wrong with this work**, since it should change nothing.

## Required documentation updates

The new page, the `SUMMARY.md` entry, and the `error-handling.md`
cross-reference. No `CHANGELOG.md` entry is required for the documentation
itself, though the 2.5.0 entry may mention the new page.

## Compatibility constraints

None. Documentation and one test.

## Security constraints

None introduced. The recovery example must note the `.bak` sensitivity point.

## Known risks

* **Documenting a behavior makes it a commitment.** After this lands, changing
  `load_or_default()` to reset on corruption becomes a breaking change. That is
  intended — the current behavior is right — but be aware you are fixing it in
  place, not just describing it.
* **Verification drift.** It is easy to transcribe the RFC's description rather
  than check the crate. The RFC was written from source reading, not from
  execution. Check.

## Required evidence

* Each behavioral claim on the page, matched to the test or the observed run that
  confirms it. A short claim-to-evidence table in the review request is the
  cleanest form.
* `cargo test --doc` output.
* Output of the new `load_or_default()` test.
* `git diff --stat` confirming no source file outside `src/core/tests.rs` changed.
* CI green on all three platforms.

## Acceptance criteria

* The new page states the concurrency contract, including the atomic-read
  guarantee, lost-update behavior, and the no-locking decision with its rationale.
* Corrupted-file behavior is stated, including that `load_or_default()` does not
  reset, and why.
* The schema-change case is called out with `#[serde(default)]` as mitigation.
* A compiling recovery example is included.
* `docs/src/SUMMARY.md` lists the page.
* A test covers `load_or_default()` returning `Deserialize` for an existing
  invalid file **and** leaving that file intact.
* No public API change, no behavior change, no new dependency.
* Every claim on the page verified against actual behavior.

## Required review-request content

Per §9.2, as a file package under `.git-exclude/review-request/NNN-slug/README.md`.
Include the claim-to-evidence table — for a documentation RFC, the proof that
each statement is true *is* the deliverable.

## Escalate rather than decide

* The crate does not actually behave as RFC 030 describes.
* Documenting a behavior honestly would make it look like a defect worth fixing —
  that is a design conversation, not a wording problem.
* The recovery example cannot be written without new API.
