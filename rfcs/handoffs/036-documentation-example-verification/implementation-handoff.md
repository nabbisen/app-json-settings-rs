# Implementation handoff — RFC 036 Documentation example verification

**Governing RFC.** [RFC 036](../../done/036-documentation-example-verification.md)
**Status.** Inherited from RFC 036 (Implemented, 2.5.1).
**Milestone.** M3 — slice 2 of 4.

## Purpose

Make the code readers actually copy verifiable, and stop the documentation
implying coverage it does not have.

## Background

Read RFC 036. The measurement behind it: all 28 Rust blocks in `docs/src/` fail
to compile, because every one is a fragment. Both mechanisms that could verify
them put visible noise on GitHub, which is currently the only place this
documentation is read — so this slice moves copy-critical code into `examples/`
instead and marks the rest honestly.

**The governing constraint:** this slice *reduces* what the documentation claims.
If you find yourself adding tooling, you are in the wrong RFC.

## Change scope

| File | Change |
|---|---|
| `docs/src/*.md` | Classify blocks; add illustrative markers; reference examples |
| `examples/` | Promoted copy-critical code, if any |
| `examples/README.md`, `docs/src/examples.md` | List any new example |
| `docs/src/testing.md` | State the verification boundary |

## Non-change scope

* **Anything under `src/`.** No crate change.
* `.github/workflows/ci.yml` — **no new job, no mdbook, no tooling.** If this
  slice needs a CI change, stop and escalate.
* `Cargo.toml` — no `book.toml`, no dependency, no `include_str!` wiring.
* The `docs/src/` page set — pages get edited, not added, removed, or reordered.
* `README.md` — RFC 035 brought it current; leave it alone unless a promoted
  example needs listing in its `## Examples` block.
* RFC 026's example maintenance rule. It still binds.

## Required implementation

### 1. Classify all 28 blocks

Apply the test from RFC 036: **would a reader paste this into their project and
expect it to run?**

Record the classification in the review request as a table — page, block, verdict.
That table is a deliverable, not scratch work; it is how the reviewer checks the
judgement rather than re-deriving it.

Two categories are illustrative by force. Do not attempt to make them compile:

* **`uwp.md`** — calls `at_uwp_local_folder()`, which is
  `cfg(all(windows, feature = "uwp"))`.
* **`migration-v2.md` "before" snippets** — deliberately pre-2.5.0 code.

### 2. Promote copy-critical code, do not invent it

**Check the existing four examples first** — `basic.rs`, `custom_root.rs`,
`update.rs`, `recovery.rs`. Most copy-critical content is likely already covered,
in which case the documentation just needs a pointer.

Promote only code that is genuinely copy-critical or already duplicated. **Do not
manufacture an example to satisfy a classification.** A fifth or sixth example
that exists only because this slice created it is a failure of this slice, not a
success.

**The recovery duplication is the one known case.** The pattern exists in both
`docs/src/operational-contract.md` and `examples/recovery.rs`. Resolve it:
`examples/recovery.rs` is canonical; the page keeps a short excerpt with an
explicit pointer to it. The page should still read well on its own — a pointer
with no code at all is worse for the reader than a short excerpt.

### 3. Mark illustrative blocks in prose

**Do not use ` ```ignore `.** Nothing runs rustdoc over these files, so the
annotation would do nothing — and GitHub does not recognise `ignore` as a
language, so it would drop syntax highlighting. That is a regression bought for
zero benefit.

Use prose:

* Uniformly illustrative pages — `api-guide.md`, `migration-v2.md`, `uwp.md` —
  get one sentence near the top.
* Mixed pages get the marker on the individual block.

Keep markers short. The goal is honesty, not a disclaimer on every page.

### 4. State the boundary in `testing.md`

Add explicitly: `examples/` and `src/` doctests are compile-checked on all three
platforms; `docs/src/` fragments are illustrative and are not.

This is the sentence whose absence let the gap survive. Do not bury it.

## Prohibited shortcuts

* **Do not add `mdbook`, `book.toml`, `include_str!` wiring, or any CI step.**
  RFC 036 rejected all of them with reasons; re-adding one silently would
  reverse a recorded decision.
* Do not mark blocks ` ```ignore `.
* Do not invent examples to make the classification tidy.
* Do not rewrite fragments to be self-contained. RFC 036 considered and rejected
  that as making every page more verbose.
* Do not delete illustrative blocks to avoid classifying them. A page that loses
  its code loses its value.
* Do not touch `src/` or CI.

## Required tests

* `cargo test --examples` passes.
* Any promoted example is **run**, and its output captured.
* Existing tests unchanged — this slice should change none.

## Required documentation updates

Covered above. No `CHANGELOG.md` entry — nothing user-visible changes at the
crate level, unless an example is promoted, in which case the next release entry
may mention it.

## Compatibility constraints

None. Documentation and possibly example targets.

## Security constraints

None introduced. If the recovery excerpt is trimmed, keep the `.bak` sensitivity
note — it is a security-relevant caveat, not decoration.

## Known risks

* **Classification is judgement.** The review will check your table, not redo it,
  so state the reasoning where a call was close.
* **Marker fatigue.** Too many disclaimers read as noise and get ignored. Page-level
  markers where possible, block-level only where genuinely mixed.
* **Scope creep toward tooling.** This slice deliberately declines full
  verification. If that feels wrong while implementing, escalate rather than
  quietly adding a job — RFC 036 records a revisit trigger for exactly that.

## Required evidence

* The classification table: page, block, copy-critical or illustrative, and why
  for close calls.
* `cargo test --examples` output.
* Run output for any promoted example.
* `git diff --stat` showing nothing under `src/` and nothing in
  `.github/workflows/`.
* Confirmation that no block was marked ` ```ignore ` —
  `grep -rn '```ignore' docs/` returning nothing.
* CI green on all three platforms.

## Acceptance criteria

* All 28 blocks classified, with the table submitted.
* Copy-critical code lives in `examples/`, referenced rather than duplicated.
* Recovery duplication resolved; `examples/recovery.rs` canonical; the page still
  reads well.
* Illustrative blocks marked in prose; no ` ```ignore ` anywhere.
* `testing.md` states the verification boundary.
* No new example invented purely to satisfy classification.
* No CI change, no tooling, no `src/` change.
* `cargo test --examples` passes; CI green.

## Required review-request content

Per §9.2, as a file package under `.git-exclude/review-request/NNN-slug/README.md`
— the next number is `008`. The classification table is the centrepiece.

## Escalate rather than decide

* A block is genuinely ambiguous and the classification changes what ships.
* Resolving the recovery duplication would leave the page unreadable.
* Full verification starts to look necessary after all — that reverses RFC 036's
  central decision and is not an implementation call.
* Any of this appears to require a CI or tooling change.
