# RFC 028 — Release verification gate

**Status.** Implemented (v2.4.1)
**Tracks.** CI coverage, RFC integrity tooling, and completion discipline.
**Touches.** `.github/workflows/ci.yml`, `scripts/check-rfcs.sh`, `rfcs/archive/`, `docs/src/maintainer-notes.md`.
**Handoff.** [implementation handoff](../handoffs/028-release-verification-gate/implementation-handoff.md)

## Summary

Make the project's verification gate cover the platforms the project documents,
make the RFC integrity script work in a fresh clone, and write down the rule that
an RFC may not be marked Implemented while its gate is red.

## Motivation

RFC 027 documents three compile errors, one of which shipped in two releases and
blocked a downstream dependent. The defects are small. The reason they survived
two release cycles is not.

Three failures compounded:

1. **The matrix does not cover the promised platforms.** All real checks —
   formatting, clippy, tests, doctests — run only on `ubuntu-latest`. Linux
   cannot see `cfg(windows)` code at all. The single Windows job runs
   `cargo check --features uwp` and nothing else, so a default-feature Windows
   break has no job that could observe it. There is no macOS job, though
   `docs/src/platform-behavior.md` documents macOS path resolution.

2. **The gate has been red continuously and was not read.** Every CI run since
   the workflow was introduced in 2.2.0 has failed, including the runs on the
   2.3.0 and 2.4.0 release tags. A gate that is always red carries no
   information, and a gate carrying no information stops being consulted.

3. **RFC acceptance was recorded against unverified claims.** RFC 021's checklist
   asserts the `uwp` feature builds. RFC 024's asserts platform notes are
   verified. Both were marked Implemented while the evidence said otherwise.
   Per the governance policy, completion requires evidence, and the evidence
   existed — it simply was not consulted.

Fixing RFC 027 without fixing this leaves the project able to repeat the failure
on the next platform-specific change.

## Goals

* Run the full check set on Linux, macOS, and Windows.
* Verify the declared MSRV.
* Make `scripts/check-rfcs.sh` succeed in a fresh clone.
* Record the rule that a red gate blocks RFC completion.
* Keep the workflow simple enough to stay readable.

## Non-goals

* Do not add release or publishing automation.
* Do not add coverage tooling, mutation testing, or benchmark gating.
* Do not add runtime UWP testing. CI cannot host a UWP container.
* Do not rewrite RFC 000. That document is a portable policy shared across
  projects; this RFC records a local project rule, not an amendment to it.
* Do not retroactively reopen RFCs 021, 023, 024, or 026. Their designs are
  sound; only their verification evidence was weak, and RFC 027 corrects the
  substantive consequences.

## Design

### Platform matrix

Promote the real check set to a matrix over `ubuntu-latest`, `macos-latest`, and
`windows-latest`:

```
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --no-default-features
cargo test --examples
cargo test --doc
```

Two checks stay single-platform because running them per-platform buys nothing:

* `cargo fmt --check` — formatting is platform-independent; run on Linux only.
* `scripts/check-rfcs.sh` — a bash script over repository files; run on Linux
  only.

The existing `cargo check --features uwp` step stays as a Windows-only step. It
cannot run elsewhere, and it is the step that verifies RFC 027's D2 and D3.

### RFC integrity script in a fresh clone

`scripts/check-rfcs.sh` currently aborts before doing any work:

```
missing RFC state directory: rfcs/proposed
```

Git does not track empty directories. `rfcs/proposed/` and `rfcs/archive/` have
been empty since 2.2.0, so neither exists in a fresh clone or in the CI
workspace. This single guard is what turns the whole Linux job red.

An empty state folder is a legitimate condition — a project with no withdrawn
RFCs genuinely has an empty `archive/`. The script should treat an absent state
directory as an empty one and continue, rather than failing.

Landing RFCs 027 and 028 in `rfcs/proposed/` populates that directory as a side
effect, but `rfcs/archive/` stays empty. The script fix is therefore still
required; it is not made redundant by this RFC's own files. Adding
`rfcs/archive/.gitkeep` is recommended alongside it so the four-folder layout
stays discoverable to contributors, but the script must not depend on it.

The script's other invariants — duplicate numbers, status matching folder, index
membership — are sound and stay as they are.

### MSRV verification

`Cargo.toml` declares `rust-version = "1.85.0"`. CI runs `stable` only, so the
claim is unverified and would break silently the first time a newer API is used.
Add a job pinned to 1.85.0 running `cargo check` and `cargo check --all-targets`.

**Confirmed for 2.4.1 by the project owner.** This item was raised as separable,
because it addresses a gap adjacent to the ones that caused the incident rather
than one of them. The owner has decided it ships in 2.4.1 rather than moving to
M2.

### Completion rule

Add to `docs/src/maintainer-notes.md`:

> An RFC does not move to `rfcs/done/` while the release gate for its change is
> red. If the gate cannot be made green, the RFC stays in `proposed/` and the
> blocking failure is recorded in it. A checklist item is marked complete only
> against a run that was actually observed, not against an expected result.

This is deliberately a written rule rather than automation. The failure was not
that CI lacked a signal; it was that a red signal did not stop the release.
Tooling cannot fix that.

## Compatibility

No crate change. No public API, dependency, or behavior effect. This RFC touches
only CI configuration, a shell script, a placeholder file, and documentation.

## Operational considerations

The matrix raises CI cost from one Linux job to three platform jobs plus two
small auxiliary jobs. For a crate with 22 fast tests and two dependencies this is
negligible in wall-clock time. macOS runners bill at a higher multiplier than
Linux; at this project's push volume the cost stays trivial, and the alternative
is shipping unverified macOS support.

The matrix will surface pre-existing macOS and Windows failures on its first run.
That is the point. Any such failure that is not already covered by RFC 027 must
be reported before 2.4.1 is cut, not silently fixed inside this RFC's scope.

## Testing and verification

* A full CI run is green on all three platforms.
* `scripts/check-rfcs.sh` passes in a clean checkout — verify with
  `git clone` into a temporary directory, or `git stash -u`, not only in the
  working tree where the directories may already exist locally.
* The MSRV job passes on 1.85.0, if that item is retained.
* Deliberately breaking one check locally is confirmed to fail the matrix on the
  affected platform.

## Risks and unresolved questions

* **The matrix could reveal further platform defects.** Likelihood: moderate,
  given that macOS and Windows have never been built. Impact: 2.4.1 scope grows.
  Mitigation: report findings to the owner as a scope decision rather than
  absorbing them.
* **A green gate can still be ignored.** This RFC reduces the chance of an
  unnoticed failure but cannot compel anyone to read the result. The completion
  rule is the mitigation, and it depends on discipline.

## Alternatives considered

* **Require `.gitkeep` files and leave the script strict.** Rejected as the sole
  fix: it makes correctness depend on a placeholder file that any cleanup commit
  could remove, reintroducing the same failure. Recommended as a complement.
* **Keep Linux-only CI and cross-compile to Windows and macOS from it.**
  Cross-compilation would have caught RFC 027's defects and is cheaper. Rejected:
  it verifies compilation but never runs the tests on the target platform, so
  path resolution and file-replacement behavior stay unverified — which is most
  of what is platform-specific in this crate.
* **Automate the completion rule by blocking merges on green CI.** Attractive,
  but branch protection is repository administration rather than crate design,
  and the project is single-maintainer. Worth revisiting if contributors join.

## Acceptance checklist

* CI runs clippy, tests, no-default-feature tests, example tests, and doctests on
  Linux, macOS, and Windows.
* `cargo fmt --check` and `scripts/check-rfcs.sh` run once, on Linux.
* The Windows `uwp` feature check is retained.
* An MSRV job pinned to 1.85.0 exists and passes.
* `scripts/check-rfcs.sh` treats an absent RFC state directory as empty and
  passes in a fresh clone.
* `rfcs/archive/.gitkeep` exists.
* `docs/src/maintainer-notes.md` records the completion rule.
* A full CI run is observed green before this RFC moves to `rfcs/done/`.
