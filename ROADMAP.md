# Roadmap

This roadmap is the planning baseline for the RFC portfolio under `rfcs/`.
Milestones group RFCs into release units. Each RFC is implemented through one or
more developer handoffs and moves to `rfcs/done/` when it ships.

Priorities:

* **P0** — release-blocking. Nothing else starts until it lands.
* **P1** — scheduled work with a committed milestone.
* **P2** — accepted direction, milestone may move.

Release tags use the `X.Y.Z` form without a `v` prefix. Major-version timing is
decided by the project owner and is not implied by RFC completion.

## Released

### 2.2.x — Foundation quality

Status: implemented in 2.2.0.

* Adopt RFC lifecycle policy.
* Add mdBook-compatible documentation.
* Add integration tests for public API behavior.
* Add explicit app identity API.
* Add checked file-name API while preserving v2.x compatibility.
* Add basic CI and RFC integrity script.

### 2.3.x — Save reliability

Status: implemented in 2.3.0.

* Added `SaveMode::Atomic` as the default save strategy.
* Added `SaveMode::Direct` and direct-save builder APIs for compatibility.
* Defined overwrite and replacement behavior on Windows, macOS, and Unix.
* Added reliability-oriented tests around save mode and temporary-file cleanup.

### 2.4.x — Minimal executable examples

Status: implemented in 2.4.0.

* Added three runnable Cargo example targets.
* Added an examples documentation page.
* Added example compile-checking to CI.

### M1 — 2.4.1 — Platform correctness and release-gate restoration

Status: implemented in 2.4.1. Priority: **P0**.

**Objective.** Restore a crate that builds on every platform it documents, and
restore a verification gate that would have caught the failure.

**Why now.** The published 2.3.0 and 2.4.0 crates do not compile on any Windows
target under default features. A downstream dependent hit this on its first
Windows release build and is blocked pending a patch release. CI has reported
failure on every push since 2.2.0 without the signal being acted on.

| RFC | Title | Priority | Depends on |
|---:|---|---|---|
| 027 | Windows build correctness | P0 | — |
| 028 | Release verification gate | P0 | — |

* RFC 027 covers the `OsStrExt` scope defect in `src/core/save.rs` and the
  `uwp` feature build failures in `src/core/dir.rs` (`Storage_Search` cargo
  feature; `Error::message()` returning `String`).
* RFC 028 covers the CI platform matrix, the RFC integrity script's failure in a
  fresh clone, and the rule that an RFC may not be marked Implemented while its
  release gate is red.

**Ordering note.** RFC 028's platform matrix is what verifies RFC 027. The CI job
must exist before RFC 027 can be accepted as complete, so 028's CI work lands
first within the release even though the two RFCs are otherwise independent.

**Exit criteria.**

* `cargo check` passes on Windows, macOS, and Linux targets.
* `cargo check --features uwp` passes on Windows.
* `cargo clippy --all-targets -- -D warnings` passes on every matrix platform.
* `scripts/check-rfcs.sh` passes in a fresh clone.
* The full CI run is green.
* No public API change, no dependency change, no behavior change.

**Post-release action.** Notify the downstream dependent that reported the
Windows build failure once 2.4.1 is published, and invite verification against
it. The reply is deliberately held until release rather than sent on scoping.

**Evidence.** RFC 028's platform matrix landed first and observed the Windows
job red
([run](https://github.com/nabbisen/app-json-settings-rs/actions/runs/30699271396)),
matching D1-D3. RFC 027 then landed and the same job went green
([run](https://github.com/nabbisen/app-json-settings-rs/actions/runs/30699425111)).
RFC 028's remaining slices (B-D) landed after, and a full CI run was observed
green on every job
([run](https://github.com/nabbisen/app-json-settings-rs/actions/runs/30700304434)).

### M2 — 2.5.0 — Durability and safety hardening

Status: released in 2.5.0. Priority: **P1**.

**Objective.** Close the reliability and safety gaps that the atomic-save work
introduced or left undefined.

| RFC | Title | Priority | Depends on | Status |
|---:|---|---|---|---|
| 033 | RFC status check correctness | P1 | 028 | Implemented — `main`, tooling-only |
| 029 | Permission preservation on atomic save | P1 | 024, 027 | Implemented — 2.5.0 |
| 034 | Explicit storage root resolution failure | P1 | — | Implemented — 2.5.0 |
| 030 | Operational contract: concurrency and corrupted files | P1 | — | Implemented — 2.5.0 |

* RFC 029 addresses atomic replacement discarding the previous file's mode. A
  settings file at `0600` becomes umask-default after the first atomic save, and
  the temporary file is readable by others while being written. This is a
  regression against v2.2 direct-write behavior for applications whose settings
  hold credentials.
* RFC 030 documents two behaviors the crate already has but has never stated:
  concurrent writes (unlocked read-modify-write, last-writer-wins, but atomic
  save does guarantee no torn reads), and what happens when the settings file
  exists but is not valid JSON. Documentation only. It absorbs the scope
  originally sketched as RFC 031; that number was a roadmap placeholder, no file
  was ever created for it, and it is retired unused.
* RFC 034 makes storage-root resolution failure explicit. `for_app()` returns
  `Result`, but that `Result` reports only an invalid app name — when `HOME` or
  `%APPDATA%` cannot be resolved the crate silently substitutes a relative path.
  Careful error handling therefore looks complete while the failure passes
  through. Fixed by making the existing `Result` carry the information callers
  already assume it carries.
* RFC 033 fixed two defects in the RFC integrity script's status check, found
  during the 2.4.1 review. The check silently passed for any RFC whose body
  quoted a Status line, and rejected every genuinely withdrawn or superseded RFC.
  Both predated RFC 028; neither affected crate behavior. **Landed on `main`
  (`f7c8205`, `ca60a5c`); tooling-only, so it carries no crate version.** It was
  a prerequisite for archiving any RFC — until it landed, the gate rejected
  correct withdrawals.

**Version note.** RFCs 029 and 034 both changed observable behavior — file
permissions and storage-root failure reporting respectively — so M2 shipped as a
minor release with release notes and a migration section for applications pinned
to 2.4.x.

**Decided.** RFC 029 creates new settings files at `0600` on Unix. Existing files
keep whatever mode they have; the change is confined to file creation.

**Evidence.** Implemented serially, each unit reviewed against artifacts before
the next began: RFC 029 (`26bbc41`), RFC 034 (`e4e354d`), RFC 030 (`64998ee`),
each followed by its close-out commit. Release candidate `6ce3fb2`; version-format
correction `a94a6fa`. Full CI matrix green on both
([run](https://github.com/nabbisen/app-json-settings-rs/actions/runs/30778518231),
[run](https://github.com/nabbisen/app-json-settings-rs/actions/runs/30781349789)).
Tagged `2.5.0` at `a94a6fa` and published to crates.io.

Before tagging, the release check carried over from the M1 reviews was
discharged: every RFC claiming `2.5.0` was confirmed present in the release —
029, 030, and 034, and only those. RFC 033 correctly claims `main`, being
tooling-only.

**Carried forward.** Three residual items were recorded rather than silently
accepted: UWP runtime behavior remains unverified (unchanged since 2.4.1); no
`docs/src/` example is compile-checked by CI; and the `0600` new-file default can
surprise a shared, group-readable settings directory on first creation, which the
documentation covers along with its one-time `chmod` remedy. The first two are M3
candidates.

## Planned

### M3 — Documentation and API completeness

Status: slices 1 and 2 released in 2.5.1; slices 5 and 6 proposed, targeting
2.6.0. Priority: **P2**.

**Objective.** Make the documentation true, verified, and complete, and close the
gap between the documented API surface and the real one.

The milestone runs as four ordered slices. Slice 1 is truth-fixing with no design
decisions beyond one recorded amendment; slices 2 to 4 each carry a decision, so
each gets its own RFC.

| Slice | RFC | Title | Priority | Status |
|---:|---:|---|---|---|
| 1 | 035 | Documentation currency and recovery example | P1 | Implemented — 2.5.1 |
| 2 | 036 | Documentation example verification | P1 | Implemented — 2.5.1 |
| 3 | 032 | API surface completeness | P2 | Not yet written |
| 4 | — | `uwp` feature disposition | P2 | Not yet written |
| 5 | 037 | Migration guidance for 2.0.x upgraders | P1 | Implemented — 2.6.0 |
| 6 | 038 | Fail-closed constructor for executable-derived identity | P1 | Implemented — 2.6.0 |
| 7 | 039 | Homes for consumer-facing answers | P2 | Proposed |

**This milestone is not one release.** It was originally scoped as 2.6.0, on the
assumption that its slices would ship together. Slices 1 and 2 turned out to
contain no code — their only user-facing effect is the crate's crates.io landing
page, which was stale — so the owner decided to release them as **2.5.1**, a
documentation patch, rather than hold them behind slices 3 and 4.

Consequences worth recording:

* RFCs 035 and 036 stop carrying provisional `main` labels and take `2.5.1`. The
  release-time check introduced in review 003 and extended in review 007 fires
  here.
* Whether slices 3 and 4 warrant their own release, and at what level, is
  decided on their own merits once scoped. If the `uwp` disposition ends in
  deprecation that is a compatibility event and earns a minor bump; if slice 3
  stays documentation-shaped it may not.
* Milestones are planning units, not release units. RFC and release boundaries
  may differ, which the governance policy already permits.

**Slice 1 — RFC 035.** The README and two documentation pages lag 2.5.0: the
"More detail" list links 7 of 13 pages and omits `operational-contract.md`
entirely, "Features / design notes" predates the release, "Why / when" offers
OS-default config locations without noting that resolution can fail, there is no
MSRV statement, and `testing.md`'s enumerated test inventory went stale within a
single release. Adds `examples/recovery.rs` covering the corrupted-file recovery
path, which RFC 030 turned from an edge case into a documented contract. Amends
RFC 026's example maintenance rule — narrowly, recording that this example meets
the existing bar rather than lowering it.

**Slice 2 — RFC 036.** No `docs/src/` example is compile-checked. Measured rather
than estimated: initializing a throwaway mdbook against the real crate and running
`mdbook test` gives **0 passed, 28 failed** — every block is a fragment with no
`use`, no type definitions, and placeholder identifiers.

The decision turned out not to be a choice of tooling. `#[doc = include_str!(…)]`
was verified by probe to work and needs no mdbook, no `book.toml`, and no new CI
job — but hidden setup lines are a rustdoc and mdbook convention, and GitHub
renders them as visible noise. Since RFC 035 made the README hard-link every
`docs/src/` page and the book is not published anywhere, GitHub is the only
reading path that exists, so every full-verification route taxes it permanently.

RFC 036 therefore moves copy-critical code into `examples/` — already
compile-checked on three platforms by existing CI — and marks the remaining
fragments as illustrative in prose. It accepts partial verification deliberately
and says so. If the book is ever published, the trade-off reverses and RFC 036
should be superseded rather than amended.

**Slice 3 — RFC 032.** `at_current_dir()` is public but has one incidental
mention in the documentation and no test. `for_app()` accepts Windows-reserved
device names such as `CON` and `NUL`; RFC 025 declared full OS-specific filename
legality a non-goal, and that limitation should be stated rather than discovered.
Also docs.rs rendering: the crate's Windows-only API is invisible on docs.rs
because builds run on Linux.

**Slice 4 — `uwp` disposition.** The feature compiles but its runtime behavior
has never been verified, and `docs/src/uwp.md` already recommends host-resolved
roots via `with_root_dir()` instead — so the crate's own documentation routes
users around it. Verify once by hand, mark experimental, or deprecate. A
compatibility decision, and therefore the project owner's.

## Candidates — not scheduled

These have no milestone. They enter the portfolio only after a planning
discussion.

* GUI-framework examples, if they remain small and dependency-light.
* Documentation for packaging models such as MSIX desktop and Pure UWP.
* Optional in-memory test helper, if it does not complicate the core API.
* **Exposing the atomic-write and permission primitives independently of the
  `ConfigManager` load/save flow.** Raised by a downstream consumer that had
  narrowed its usage to path derivation and so could benefit from neither
  2.3.0's atomic save nor 2.5.0's permission preservation — its own replacement
  write path had to reimplement both. Cuts against the crate's stated non-goals,
  which keep the API surface deliberately small, so it wants a second consumer
  asking before it is designed. Recorded so the observation is not lost.

## Decisions reserved for the project owner

* **Decided.** M1 ships as 2.4.1 carrying RFCs 027 and 028 together.
* **Decided.** No calendar targets are tracked. Milestones are ordered by
  sequence and dependency, not by date.
* **Decided.** The MSRV verification job in RFC 028 ships in 2.4.1 rather than
  moving to M2.
* **Decided.** 2.3.0 and 2.4.0 are not yanked from crates.io. The failure is at
  compile time, so no user has shipped broken runtime behavior.
* Any major-version transition. Nothing currently planned requires one.
