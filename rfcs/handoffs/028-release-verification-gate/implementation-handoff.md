# Implementation handoff — RFC 028 Release verification gate

**Governing RFC.** [RFC 028](../../done/028-release-verification-gate.md)
**Status.** Inherited from RFC 028 (Implemented, v2.4.1).
**Milestone.** M1 — release 2.4.1.

## Purpose

Make the verification gate cover the platforms the project documents, make the
RFC integrity script work in a fresh clone, and record the rule that a red gate
blocks RFC completion.

## Background

RFC 027's defects survived two releases because the gate could not see them and
because a continuously red gate stopped being read. Read RFC 028 for the full
analysis. This handoff implements the remedy.

This work touches no crate source. If you find yourself editing anything under
`src/`, you are in RFC 027's scope or you have found something new — either way,
stop.

## Implementation slices

Four slices. **Slice A must land first** — it is what makes RFC 027's fix
observable. Slices B, C, and D are independent of each other.

### Slice A — platform matrix (land first)

Promote the real check set in `.github/workflows/ci.yml` to a matrix over
`ubuntu-latest`, `macos-latest`, and `windows-latest`:

```
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --no-default-features
cargo test --examples
cargo test --doc
```

Keep single-platform, on Linux only:

* `cargo fmt --check` — formatting is platform-independent.
* `scripts/check-rfcs.sh` — a bash script over repository files.

Keep the existing `cargo check --features uwp` as a Windows-only step.

**Expect this slice to go red on Windows.** That is the intended outcome and the
evidence RFC 027 needs. Capture the run URL. Do not fix the crate here.

### Slice B — RFC integrity script

`scripts/check-rfcs.sh` aborts when a state directory is absent. Git does not
track empty directories, so `rfcs/archive/` does not exist in a fresh clone.
Reproduce before changing anything:

```sh
scripts/check-rfcs.sh
# missing RFC state directory: rfcs/archive
```

Change the script to treat an absent state directory as an empty one and
continue. An empty `archive/` is a legitimate state for a project with no
withdrawn RFCs.

Add `rfcs/archive/.gitkeep` so the four-folder layout stays discoverable.
**The script must pass even if that file is deleted** — do not make correctness
depend on the placeholder.

Leave the script's other invariants alone: duplicate numbers, status-matches-
folder, and index membership are all sound.

### Slice C — completion rule

Add to `docs/src/maintainer-notes.md`:

> An RFC does not move to `rfcs/done/` while the release gate for its change is
> red. If the gate cannot be made green, the RFC stays in `proposed/` and the
> blocking failure is recorded in it. A checklist item is marked complete only
> against a run that was actually observed, not against an expected result.

Wording may be adjusted for flow. The three constraints — red gate blocks
promotion, blocked RFCs stay in `proposed/` with the reason recorded, checklists
cite observed runs — must survive any rewording.

### Slice D — MSRV job (confirmed for 2.4.1)

`Cargo.toml` declares `rust-version = "1.85.0"`; CI runs `stable` only, so the
claim is unverified.

Add a job pinned to 1.85.0 running `cargo check` and `cargo check --all-targets`.

**Confirmed for 2.4.1 by the project owner.** This slice was originally handed
over as pending; it is now required. Still implement it as a self-contained job
so it stays independently reviewable, but it is no longer optional and must be
green before 2.4.1 is cut.

If the MSRV check fails, the declared 1.85.0 is already wrong. Do not bump the
value to make the job pass — escalate. See "Escalate rather than decide" below.

## Change scope

* `.github/workflows/ci.yml`
* `scripts/check-rfcs.sh`
* `rfcs/archive/.gitkeep` (new)
* `docs/src/maintainer-notes.md`

## Non-change scope

* Anything under `src/`, `tests/`, or `examples/`.
* `Cargo.toml`, including the declared MSRV value itself. Slice D verifies the
  existing claim; it does not change it.
* `rfcs/done/000-rfc-lifecycle-policy.md`. It is a portable policy shared across
  projects. This RFC records a local rule in maintainer notes instead.
* RFCs 021, 023, 024, and 026. Their checklists were recorded against weak
  evidence, but their designs are sound and RFC 027 corrects the substantive
  consequences. Do not retroactively edit them.
* The script's existing invariants.
* Release or publishing automation, coverage tooling, branch protection.

## Prohibited shortcuts

* **Do not make CI green by weakening it.** No `continue-on-error`, no removed
  steps, no downgraded `-D warnings`, no `--no-fail-fast` used to hide a failure.
* Do not delete the `check-rfcs.sh` step to get past slice B.
* Do not satisfy slice B by adding `.gitkeep` alone and leaving the script
  strict. The script fix is the requirement; the placeholder is a complement.
* **If the matrix reveals a platform failure not covered by RFC 027 D1–D3, do not
  fix it here.** Report it. It is a scope decision for the owner, not an
  implementation detail. This is the single most likely way this work goes wrong.
* Do not reduce the matrix to Linux-plus-cross-compilation because it is cheaper.
  RFC 028 considered and rejected that: it never runs the tests on the target
  platform, which is where this crate's platform-specific behavior lives.

## Required tests

No crate tests change. Verification is of the gate itself:

* Run `scripts/check-rfcs.sh` in a **fresh clone**, not only in your working tree
  where `rfcs/archive/` may already exist locally:

  ```sh
  git clone <repo> /tmp/gate-check && cd /tmp/gate-check && scripts/check-rfcs.sh
  ```

* Confirm the script still passes with `rfcs/archive/.gitkeep` removed.
* Confirm the script still **fails** correctly when an invariant is genuinely
  violated — for example, temporarily set a Status field that disagrees with its
  folder. A script that passes unconditionally is worse than one that fails.
* Deliberately break one check locally and confirm the matrix fails on the
  affected platform, then revert.

## Required documentation updates

`docs/src/maintainer-notes.md` per slice C. Its existing pre-release command list
should also reflect the final CI check set so the two do not drift.

## Compatibility constraints

None. No crate change, no API, dependency, or behavior effect.

## Security constraints

None. Do not add secrets, tokens, or registry credentials to the workflow. 2.4.1
publishing stays manual.

## Known risks

* **The matrix surfaces pre-existing macOS or Windows failures.** Likelihood
  moderate — neither platform has ever been built. Mitigation: report, do not
  absorb.
* **macOS runners bill at a higher multiplier.** Accepted in the RFC; at this
  project's push volume the cost is trivial.
* **A green gate can still be ignored.** Slice C is the mitigation and it depends
  on discipline, not tooling.

## Required evidence

* CI run URL for slice A showing the matrix running on all three platforms,
  including the expected Windows failure.
* Terminal output of `scripts/check-rfcs.sh` in a fresh clone, before and after
  slice B.
* Output showing the script still passes without `.gitkeep`.
* Output showing the script still fails on a deliberately violated invariant.
* CI run URL showing the full matrix green after RFC 027 lands.
* MSRV job output on 1.85.0.

## Acceptance criteria

* Clippy, tests, no-default-feature tests, example tests, and doctests run on
  Linux, macOS, and Windows.
* `cargo fmt --check` and `scripts/check-rfcs.sh` run once, on Linux.
* The Windows `uwp` feature check is retained.
* `scripts/check-rfcs.sh` passes in a fresh clone and does not depend on
  `.gitkeep`.
* `rfcs/archive/.gitkeep` exists.
* `docs/src/maintainer-notes.md` records the completion rule.
* Slice D is implemented and the MSRV job passes on 1.85.0.
* No file outside the Change scope is modified.
* A full CI run is **observed** green before RFC 028 moves to `rfcs/done/` — this
  RFC is the one that must not be marked complete against an expected result.

## Required review-request content

Follow §9.2 of the organization workflow document, with slice-by-slice reporting
so A–D can be reviewed independently.

## Escalate rather than decide

* The matrix reveals a failure outside RFC 027 D1–D3.
* Making CI green appears to require weakening a check.
* A platform's runner cannot execute part of the check set.
* Slice D's MSRV check fails, meaning the declared 1.85.0 is already wrong. That
  is a compatibility question for the owner, not a value to quietly bump.
