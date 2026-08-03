# Implementation handoff — RFC 035 Documentation currency and recovery example

**Governing RFC.** [RFC 035](../../proposed/035-documentation-currency-and-recovery-example.md)
**Status.** Inherited from RFC 035 (Proposed).
**Milestone.** M3 — slice 1 of 4.

## Purpose

Make the user-facing documentation match 2.5.0, restructure the README's
documentation links into reader paths, and add a runnable recovery example.

## Background

Read RFC 035 for the seven findings and the reasoning. In short: 2.5.0 changed
observable behavior twice and added a documentation page; the README and two
pages did not follow.

**The governing constraint, same as RFC 030's:** the code is correct and the
documentation is behind. If a claim in RFC 035 does not match the crate, the RFC
is wrong — report it. Do not change code to match a document.

## Change scope

| File | Change |
|---|---|
| `README.md` | Why/when, features, MSRV, reader paths |
| `docs/src/testing.md` | Replace the enumerated test list with categories |
| `docs/src/error-handling.md` | Prose for `ConfigError::Platform` |
| `docs/src/examples.md` | List the new example |
| `examples/recovery.rs` | **New file** |
| `examples/README.md` | List the new example |

## Non-change scope

* **Any file under `src/`.** This slice changes no crate behavior. If you find
  yourself editing source, stop.
* `docs/src/operational-contract.md`, `save-behavior.md`, `platform-behavior.md`,
  `api-guide.md`, `storage-model.md`, `quick-start.md`, `introduction.md`,
  `migration-v2.md`, `maintainer-notes.md`, `uwp.md` — accurate as they stand.
  They are *linked* from the new reader paths, not edited.
* `at_current_dir()` documentation, Windows reserved names, docs.rs metadata —
  RFC 032, a later slice.
* The `uwp` feature's status — a later slice.
* Compile-checking of `docs/src/` examples — a separate RFC. **Do not add
  `mdbook test`, `include_str!` doctests, or any CI step for it here.**
* The existing three examples. Leave `basic.rs`, `custom_root.rs`, `update.rs`
  alone.
* `CHANGELOG.md` — no entry needed; nothing user-visible changes. The next
  release entry may mention the example.

## Required implementation

### 1. README

Keep the six-section structure the project rules require. Within it:

* **Why / when** — add that `for_app()` reports failure when the platform
  configuration directory cannot be resolved, and that `with_root_dir()` covers
  that case. **One clause, not a paragraph.** The README must not become a
  manual.
* **Features / design notes** — add permission preservation and `for_app()`'s
  error reporting. Remove nothing still true.
* **MSRV** — state the declared minimum Rust version and that CI verifies it.
  Read the value from `Cargo.toml`; do not hard-code from memory.
* **More detail** — replace the flat path list with three reader paths:
  * *Getting started* — `introduction.md`, `quick-start.md`, `examples.md`
  * *Using the crate* — `storage-model.md`, `save-behavior.md`, `api-guide.md`,
    `error-handling.md`, `operational-contract.md`, `platform-behavior.md`,
    `uwp.md`
  * *Contributing and maintaining* — `testing.md`, `migration-v2.md`,
    `maintainer-notes.md`

  **Every page in `docs/src/` except `SUMMARY.md` must appear in exactly one
  path.** Verify by listing the directory and checking off, not by eye — a page
  appearing nowhere is the exact bug this replaces.

### 2. `docs/src/testing.md`

Replace the enumerated "the crate's own tests cover" list with a short
description of the **categories** covered, plus a pointer to where the tests
live (`src/core/tests.rs`, `src/core/save/tests.rs`, `src/core/dir/tests.rs`,
`tests/public_api.rs`).

Do not write a test count. Counts go stale as fast as lists.

Keep the existing guidance on temporary roots, no-default-feature builds, and
`cargo test --examples` — all still accurate.

### 3. `docs/src/error-handling.md`

Give `ConfigError::Platform` prose alongside its table row: what produces it
(`for_app()` when the platform configuration directory cannot be resolved; the
optional UWP resolver), and the remedy (`with_root_dir()` with an explicit path).
Cross-reference `migration-v2.md`.

### 4. `examples/recovery.rs`

Demonstrate the pattern from `operational-contract.md`.

**The example must actually exercise the recovery branch when run.** Deliberately
write an invalid settings file first, so running it prints the recovery path
rather than silently succeeding. An error-handling example that never errors
teaches nothing — this is the single most important requirement here.

Constraints, all inherited from RFC 026 and still binding:

* Write only to a temporary root. Never a real application data directory.
* No dependency beyond what the existing examples use.
* Short. It demonstrates one workflow, not the API surface.

Note in a comment that the `.bak` copy carries the same sensitivity as the
original settings file.

### 5. Example listings

Add matching entries to `examples/README.md` and `docs/src/examples.md`, in the
style of the existing three.

## Prohibited shortcuts

* **Do not transcribe claims from RFC 035 without checking them against the
  crate.** The RFC was written from a survey; verify before publishing.
* Do not add a test count or a test enumeration to `testing.md`.
* Do not "improve" the other documentation pages while nearby. They are in the
  non-change scope for a reason — this slice is about the pages that are wrong.
* Do not add CI steps, `mdbook test`, or doctest wiring. Separate RFC.
* Do not expand the README into a manual. Detail belongs in `docs/src/`.
* Do not change `src/` in any way.

## Required tests

* `cargo test --examples` passes — the new example compiles under real CI
  coverage.
* The example is **run** and its output captured, showing the recovery branch
  executing.
* All existing tests pass unchanged. Any change there means something is wrong.

## Compatibility constraints

None. Documentation and one example target.

## Security constraints

None introduced. The example's comment about `.bak` sensitivity is required, not
optional.

## Known risks

* **The README grows.** Reader paths replace an existing list, and the new
  factual material is a few clauses, so net growth should be small. If it grows
  materially, move detail into `docs/src/` rather than trimming accuracy.
* **A fourth example raises maintenance cost.** RFC 035 admits it on stated
  grounds; do not treat that as licence for a fifth.

## Required evidence

* `ls docs/src/` alongside the README's three reader paths, showing every page
  accounted for exactly once.
* `cargo test --examples` output.
* `cargo run --example recovery` output, showing the recovery branch running.
* `git diff --stat` confirming nothing under `src/` changed.
* CI green on all three platforms.

## Acceptance criteria

* README states MSRV, `for_app()`'s failure mode, and 2.5.0's behavior changes.
* README's reader paths cover every `docs/src/` page exactly once.
* `testing.md` describes categories with a pointer to the test modules; no
  enumeration, no count.
* `error-handling.md` gives `ConfigError::Platform` cause and remedy.
* `examples/recovery.rs` exists, uses a temporary root, and exercises the
  recovery branch when run.
* Both example listings updated.
* No `src/` change, no behavior change, no new dependency, no CI change.
* Every added claim verified against the crate.

## Required review-request content

Per §9.2, as a file package under `.git-exclude/review-request/NNN-slug/README.md`
— the next number is `007`. Include the page-coverage checklist and the example's
actual run output.

## Escalate rather than decide

* A claim in RFC 035 does not match the crate.
* A `docs/src/` page turns out to be wrong in a way this slice's scope does not
  cover.
* The recovery example cannot exercise its error branch without new API.
* The README cannot absorb the required facts while staying concise.
