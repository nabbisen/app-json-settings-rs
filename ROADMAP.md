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

### v2.2.x — Foundation quality

Status: implemented in v2.2.0.

* Adopt RFC lifecycle policy.
* Add mdBook-compatible documentation.
* Add integration tests for public API behavior.
* Add explicit app identity API.
* Add checked file-name API while preserving v2.x compatibility.
* Add basic CI and RFC integrity script.

### v2.3.x — Save reliability

Status: implemented in v2.3.0.

* Added `SaveMode::Atomic` as the default save strategy.
* Added `SaveMode::Direct` and direct-save builder APIs for compatibility.
* Defined overwrite and replacement behavior on Windows, macOS, and Unix.
* Added reliability-oriented tests around save mode and temporary-file cleanup.

### v2.4.x — Minimal executable examples

Status: implemented in v2.4.0.

* Added three runnable Cargo example targets.
* Added an examples documentation page.
* Added example compile-checking to CI.

### M1 — v2.4.1 — Platform correctness and release-gate restoration

Status: implemented in v2.4.1. Priority: **P0**.

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

## Planned

### M2 — v2.5.0 — Durability and safety hardening

Status: planned. Priority: **P1**. Sequence: after M1.

**Objective.** Close the reliability and safety gaps that the atomic-save work
introduced or left undefined.

| RFC | Title | Priority | Depends on |
|---:|---|---|---|
| 029 | Permission and ownership preservation on atomic save | P1 | 027 |
| 030 | Concurrency contract for load, save, and update | P1 | — |
| 031 | Recovery guidance for externally corrupted settings files | P2 | — |

* RFC 029 addresses atomic replacement discarding the previous file's mode. A
  settings file at `0600` becomes umask-default after the first atomic save, and
  the temporary file is readable by others while being written. This is a
  regression against v2.2 direct-write behavior for applications whose settings
  hold credentials.
* RFC 030 defines and documents what happens when two processes update the same
  settings file. The current behavior is unlocked read-modify-write with
  last-writer-wins, which is defensible but must be stated rather than implied.
* RFC 031 covers the existing candidate: what an application should do when the
  settings file exists but is not valid JSON.

**Version note.** RFC 029 changes observable file-permission behavior. It is a
minor release, not a patch.

### M3 — v2.6.0 — Documentation and API completeness

Status: planned. Priority: **P2**. Sequence: after M2.

**Objective.** Close the gap between the documented API surface and the real one.

| RFC | Title | Priority | Depends on |
|---:|---|---|---|
| 032 | API surface and documentation completeness | P2 | — |

* `at_current_dir()` is public but appears in no documentation page and has no
  test.
* `for_app()` accepts Windows-reserved device names such as `CON` and `NUL`.
  RFC 025 declared full OS-specific filename legality a non-goal; that limitation
  should be stated explicitly rather than left to be discovered.
* Review docs.rs rendering and doctest coverage.

## Candidates — not scheduled

These have no milestone. They enter the portfolio only after a planning
discussion.

* GUI-framework examples, if they remain small and dependency-light.
* Documentation for packaging models such as MSIX desktop and Pure UWP.
* Optional in-memory test helper, if it does not complicate the core API.

## Decisions reserved for the project owner

* **Decided.** M1 ships as 2.4.1 carrying RFCs 027 and 028 together.
* **Decided.** No calendar targets are tracked. Milestones are ordered by
  sequence and dependency, not by date.
* **Decided.** The MSRV verification job in RFC 028 ships in 2.4.1 rather than
  moving to M2.
* **Decided.** 2.3.0 and 2.4.0 are not yanked from crates.io. The failure is at
  compile time, so no user has shipped broken runtime behavior.
* Any major-version transition. Nothing currently planned requires one.
