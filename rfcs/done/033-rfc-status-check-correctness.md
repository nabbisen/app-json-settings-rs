# RFC 033 — RFC status check correctness

**Status.** Implemented
**Tracks.** Correctness of the RFC integrity script's status invariant.
**Touches.** `scripts/check-rfcs.sh`, `rfcs/archive/` (test fixture only, not retained).
**Handoff.** [implementation handoff](../handoffs/033-rfc-status-check-correctness/implementation-handoff.md)

## Summary

The status-matches-folder check in `scripts/check-rfcs.sh` is wrong in two
opposite directions at once: it silently passes for an RFC whose body quotes a
Status line, and it rejects every genuinely withdrawn or superseded RFC. Fix both
by anchoring the check to the real frontmatter Status line and by using a correct
alternation pattern.

## Motivation

RFC 028 restored the release gate. This RFC fixes the one check inside it that
does not do what it claims.

Both defects predate RFC 028 — they have been present since the script was
introduced in 2.2.0. RFC 028's handoff described the script's remaining
invariants as sound and instructed the implementer to leave them alone. That
premise was incorrect. The implementer found the first defect during
verification, correctly declined to redesign the invariant under a handoff that
told them not to, and reported it. The second was found during review of that
report.

A check that reports success regardless of the condition it tests is worse than
no check, because it is trusted. This is the same failure mode that produced RFC
027's incident, at a smaller scale.

### D1 — false pass on any RFC that quotes a Status line

The check greps the **whole file**:

```sh
if ! grep -Eq "^\*\*Status\.\*\* ($expected)" "$file"; then
```

`rfcs/done/000-rfc-lifecycle-policy.md` is the document that *defines* the Status
convention, so its body contains illustrative lines such as
`**Status.** Implemented (v1.4.0)`. Those satisfy the pattern no matter what the
file's real Status field says.

Reproduced in a fresh clone by setting RFC 000's actual Status line to `Proposed`
while the file stayed in `rfcs/done/`. The script exited 0.

Today only RFC 000 triggers this. Any future RFC that quotes a Status line — a
revision to the lifecycle policy, an RFC discussing status conventions — inherits
the same blind spot.

### D2 — every valid archived RFC is rejected

The archive branch sets:

```sh
rfcs/archive/*) expected="Withdrawn\|Superseded" ;;
```

which is interpolated into a `grep -E` pattern. In ERE, `\|` is a **literal pipe
character**, not alternation. The check therefore matches only the literal string
`Withdrawn|Superseded`:

```
NO MATCH: **Status.** Withdrawn — overlapped with RFC 035
NO MATCH: **Status.** Superseded by RFC 042
MATCH   : **Status.** Withdrawn|Superseded
```

End to end, placing a correctly formatted withdrawn RFC in `rfcs/archive/`
produces:

```
RFC status does not match folder: rfcs/archive/999-test.md
```

This has never fired because `archive/` has always been empty. It will fire on
the first RFC anyone withdraws, and it will look like the RFC is malformed rather
than the check.

### D3 — empty repository state passes silently

RFC 028 made the script tolerate an absent state directory, which was correct.
The implementation exits 0 when **none** of the three directories exists. A
repository with no `rfcs/` tree at all is a broken repository, not a valid empty
one, and should not report success.

Severity is low — it requires the RFC tree to be deleted wholesale — but it is one
line to make it explicit.

## Goals

* The status check reads only the file's real frontmatter Status line.
* Withdrawn and superseded RFCs in `archive/` validate correctly.
* A wholly absent `rfcs/` tree is reported rather than passed.
* The check continues to fail on genuine violations, verified by test.

## Non-goals

* Do not rewrite the script in another language. It is short shell and should
  stay short shell.
* Do not add a Markdown or YAML frontmatter parser.
* Do not change the RFC file format or the Status field convention itself.
  RFC 000 defines that format; this RFC fixes a checker that misreads it.
* Do not change the script's other invariants (duplicate numbers, index
  membership). They are not implicated, and RFC 028's experience is a caution
  against declaring untested invariants sound — but tightening them is separate
  work.
* Do not add CI beyond what RFC 028 established.

## Design

### Anchor the status check to the frontmatter line

Read the first Status line in the file rather than grepping the whole body. RFC
000 places `**Status.**` in a metadata block within the first few lines of every
RFC, and every RFC in the repository follows this.

Extract the first matching line, then compare that single line against the
expected value:

```sh
status_line="$(grep -m1 -E '^\*\*Status\.\*\* ' "$file" || true)"
```

`grep -m1` stops at the first match, which is the frontmatter line. Prose
examples appear later in the body and are therefore never consulted.

An RFC with no Status line at all must fail rather than pass. The current code
would also fail it, but only incidentally; make it explicit.

### Fix the alternation

Use a real ERE alternation for the archive case:

```sh
rfcs/archive/*) expected="Withdrawn|Superseded" ;;
```

with the surrounding pattern kept as `grep -E`. Verify against the exact Status
formats RFC 000 documents, which carry trailing text:

```markdown
**Status.** Withdrawn — overlapped with RFC 035; merged there.
**Status.** Superseded by RFC 042
**Status.** Implemented (v1.4.0)
```

The pattern must match the state word as a prefix and tolerate the trailing
reason or version.

### Report an entirely absent RFC tree

When no state directory exists, distinguish "no RFCs yet" from "the RFC tree is
gone". If `rfcs/` itself is absent, report and exit non-zero.

## Compatibility

No crate change. No public API, dependency, or behavior effect. Script only.

## Security considerations

None.

## Testing and verification

The verification requirement is the point of this RFC. Each case must be
demonstrated, not asserted:

* RFC 000 with a deliberately wrong Status field in `done/` is **rejected**.
  This is the exact case that passes today.
* Every RFC currently in the repository still passes unchanged.
* A withdrawn RFC placed in `archive/` with
  `**Status.** Withdrawn — reason` **passes**.
* A superseded RFC with `**Status.** Superseded by RFC NNN` **passes**.
* An RFC in `archive/` carrying `**Status.** Implemented (v2.4.1)` is
  **rejected**.
* An RFC with no Status line is **rejected**.
* A fresh clone with an empty `archive/` still passes, and still passes with
  `.gitkeep` removed — RFC 028's behavior must not regress.
* A repository with `rfcs/` removed entirely is **rejected**.

Test fixtures placed in `archive/` for these checks are removed before commit.
`git status` must be clean afterwards.

## Risks and unresolved questions

* **`grep -m1` assumes the frontmatter Status line precedes any prose example.**
  True for every RFC in the repository and implied by RFC 000's format. If a
  future RFC violates it, the check reads the wrong line. Accepted: the
  alternative is a frontmatter parser, which is disproportionate here.
* **This RFC fixes a checker, so its own verification cannot lean on the
  checker.** Every case above is demonstrated by hand against a real file layout.

## Alternatives considered

* **Restrict the grep to the first N lines** (`head -20 | grep`). Simpler than
  `grep -m1` in appearance, but encodes an arbitrary constant that silently
  breaks if an RFC's metadata block moves. Rejected.
* **Require Status in a machine-readable YAML frontmatter block.** Robust, but
  changes the RFC format for every existing file and contradicts RFC 000's
  deliberately loose "exact format is up to each project". Rejected as
  disproportionate.
* **Delete the status check.** It has been unreliable since 2.2.0 and nobody
  noticed, which is an argument that it earns little. Rejected: the folder is the
  source of truth and a lying Status field is a documented anti-pattern in RFC
  000. The check is worth having; it is worth having correct.

## Acceptance criteria

* The status check consults only the first `**Status.**` line in each file.
* RFC 000 with a mismatched Status field is rejected.
* Withdrawn and superseded RFCs in `archive/` validate correctly.
* An RFC with no Status line is rejected.
* An entirely absent `rfcs/` tree is reported and exits non-zero.
* An empty but present `archive/` still passes, with and without `.gitkeep`.
* Every currently committed RFC still passes.
* Each test case above is demonstrated with captured output in the review
  request.
* `git status` is clean — no test fixture is committed.
* No change outside `scripts/check-rfcs.sh`.
