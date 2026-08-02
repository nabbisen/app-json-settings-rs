# Implementation handoff — RFC 033 RFC status check correctness

**Governing RFC.** [RFC 033](../../done/033-rfc-status-check-correctness.md)
**Status.** Inherited from RFC 033 (Implemented).
**Milestone.** M2 — release 2.5.0.

## Purpose

Fix the status-matches-folder check in `scripts/check-rfcs.sh`. It currently
passes when it should fail, and fails when it should pass.

## Background

Read RFC 033 for the analysis. In short:

* **D1** — the check greps the whole file, so `rfcs/done/000-rfc-lifecycle-policy.md`
  satisfies it via illustrative `**Status.**` lines in its body regardless of its
  real Status field.
* **D2** — the archive branch uses `\|` inside a `grep -E` pattern, which is a
  literal pipe, not alternation. Every genuinely withdrawn or superseded RFC is
  rejected.
* **D3** — when no state directory exists at all, the script exits 0. A deleted
  `rfcs/` tree should be reported, not passed.

Both D1 and D2 predate RFC 028. D2 has never fired because `archive/` has always
been empty; it will fire on the first withdrawal.

## Change scope

`scripts/check-rfcs.sh` — and nothing else.

## Non-change scope

* **`rfcs/done/000-rfc-lifecycle-policy.md`.** See the first prohibited shortcut
  below. This is the most important line in this document.
* Any other RFC file, in any folder.
* The RFC file format or the Status field convention. RFC 000 defines that; this
  work fixes a checker that misreads it.
* The script's other invariants — duplicate numbers, index membership. They are
  not implicated here. Note that RFC 028's handoff called *this* invariant sound
  without testing it, which is exactly how these defects survived; if you have
  reason to believe another invariant is also broken, report it rather than
  fixing it here.
* `.github/workflows/ci.yml`. RFC 028 already runs this script; no CI change is
  needed.
* Anything under `src/`, `docs/`, `tests/`, or `examples/`.

## Required implementation

### D1 — anchor to the frontmatter Status line

Read only the **first** `**Status.**` line in the file, then compare that single
line against the expected value:

```sh
status_line="$(grep -m1 -E '^\*\*Status\.\*\* ' "$file" || true)"
```

`grep -m1` stops at the first match, which is the metadata line near the top of
every RFC. Body prose appears later and is therefore never consulted.

A file with **no** Status line at all must fail. The current code fails it only
incidentally; make that explicit.

### D2 — real alternation

```sh
rfcs/archive/*) expected="Withdrawn|Superseded" ;;
```

Keep the surrounding matcher as `grep -E`. The pattern must match the state word
as a prefix and tolerate the trailing reason or version that RFC 000 documents:

```markdown
**Status.** Withdrawn — overlapped with RFC 035; merged there.
**Status.** Superseded by RFC 042
**Status.** Implemented (v1.4.0)
```

### D3 — absent RFC tree

Distinguish "no RFCs yet" from "the RFC tree is gone." If `rfcs/` itself does not
exist, report and exit non-zero. An empty-but-present `archive/` must still pass —
do not regress RFC 028's fix.

## Prohibited shortcuts

* **Do not edit RFC 000 to remove its illustrative `**Status.**` lines.** This is
  the tempting one-line "fix" and it is wrong twice over: it mutilates the
  document that defines the Status convention in order to satisfy a broken
  checker, and it leaves the checker broken for the next RFC that quotes a Status
  line. Fix the checker.
* Do not rewrite the script in Python, Rust, or anything else. It is short shell
  and stays short shell.
* Do not add a Markdown or YAML frontmatter parser.
* Do not restrict the grep with `head -N`. It encodes an arbitrary constant that
  breaks silently if a metadata block moves. RFC 033 considered and rejected it.
* Do not delete the status check to make the problem go away.
* Do not commit test fixtures. Every fixture you create under `rfcs/archive/` for
  verification must be removed before you commit.

## Required tests

**This work fixes a checker, so its verification cannot lean on the checker.**
Every case below must be demonstrated by hand against a real file layout, with
captured output. "The script passes" is not evidence here.

| # | Case | Expected |
|---|---|---|
| 1 | RFC 000 in `done/` with its real Status set to `Proposed` | **reject** |
| 2 | Every RFC currently committed, unmodified | pass |
| 3 | `archive/999-test.md` with `**Status.** Withdrawn — reason` | pass |
| 4 | `archive/998-test.md` with `**Status.** Superseded by RFC 042` | pass |
| 5 | `archive/997-test.md` with `**Status.** Implemented (v2.4.1)` | **reject** |
| 6 | An RFC with no `**Status.**` line at all | **reject** |
| 7 | Fresh clone, empty `archive/`, and again with `.gitkeep` deleted | pass |
| 8 | `rfcs/` removed entirely | **reject** |

Case 1 is the exact scenario that passes today — it is the regression test for
D1. Case 3 and 4 are the scenarios that fail today — they are the regression
tests for D2.

Cases 3–5 need index entries in `rfcs/README.md` to get past the
index-membership check; revert those along with the fixtures.

## Required documentation updates

None. The script's behavior change is internal to the gate. Do not add a section
to `docs/src/maintainer-notes.md` — the completion rule recorded there is
unaffected.

## Compatibility constraints

None. Script only. No crate, API, dependency, or behavior effect.

## Security constraints

None.

## Known risks

* **`grep -m1` assumes the frontmatter Status line precedes any prose example.**
  True for every RFC in the repository and implied by RFC 000's format. Accepted
  in the RFC; the alternative is a parser, which is disproportionate.
* **Fixture cleanup.** Cases 3–6 create files in the RFC tree. Leaving one behind
  would commit a fake RFC. `git status` must be clean before you commit, and the
  review request must say so.

## Required evidence

* Captured terminal output for all eight cases in the table, each showing the
  command, the script's output, and its exit code.
* `git status` output after verification, showing a clean tree.
* `git diff` of the final change, confirming `scripts/check-rfcs.sh` is the only
  file touched.
* Confirmation that CI's `Formatting and RFC integrity` job is green.

## Acceptance criteria

* The status check consults only the first `**Status.**` line per file.
* All eight test cases behave as specified in the table.
* No file outside `scripts/check-rfcs.sh` is modified.
* RFC 000 is byte-identical to its current committed state.
* No test fixture is committed; `git status` is clean.
* CI is green.

## Required review-request content

Per §9.2, delivered as a file package under
`.git-exclude/review-request/NNN-slug/README.md`. Include the eight-case evidence
table with real captured output — for this RFC the evidence *is* the deliverable.

## Escalate rather than decide

* An RFC in the repository turns out to place prose `**Status.**` lines above its
  frontmatter, breaking the `grep -m1` assumption.
* Another of the script's invariants appears broken.
* Fixing the check appears to require changing the RFC file format.
* The fix cannot be expressed without adding a parser or a new dependency.
