# RFC 039 — Homes for consumer-facing answers

**Status.** Implemented
**Tracks.** Where answers given to consumers live, so the next consumer finds them.
**Touches.** `docs/src/api-guide.md`, `docs/src/introduction.md`, `docs/src/maintainer-notes.md`.
**Handoff.** [implementation handoff](../handoffs/039-homes-for-consumer-answers/implementation-handoff.md)

## Summary

Place two answers that currently exist only in a one-off reply to a single
consumer, and record the rule that produced them, so the next consumer asking the
same question finds the answer in the documentation instead of receiving another
private reply.

**Recommends against a FAQ page**, having weighed it. Both answers have natural
homes in existing pages, and a fourteenth page holding two entries would compete
with those pages rather than complement them.

## Motivation

A downstream consumer's audit produced an exchange that changed the crate: RFC
038 exists because of it, and RFC 037 was amended twice before implementation.
Most of what we told them landed in published documentation, because those RFCs
put it there.

Two answers did not:

1. **Why the crate will never warn when a fallback fires.** They proposed a
   `log`/`tracing` warning as a remedy for `new()`'s silent substitution. We
   declined — the crate has no logging dependency and would not take one for a
   rare case. That reasoning exists only in the reply. A consumer who hits the
   same silence will reasonably wonder whether a warning exists and how to enable
   it. The answer is that none exists, by design.

2. **How to inspect what the crate derived.** `folder_path()` returns the
   resolved directory, so a caller can assert it matches expectation before
   relying on it. This is the answer for someone who wants to keep using `new()`
   but verify it — as opposed to switching to `try_new()` or `for_app()`.
   `folder_path()` appears nowhere in `docs/src/`.

The second is the sharper gap. RFC 038 shipped documentation of a hazard — two
applications colliding on one settings file — together with two remedies that
both require changing constructor. The third remedy, *check it yourself*, is
undocumented, and it is the one that costs a consumer nothing to adopt.

### The systemic version

Consumer exchanges produce answers of general interest. Today the destination is
a reply file, which reaches one team. The next team asks the same question. This
RFC places the two current answers and records the rule so the pattern does not
repeat.

## Goals

* Both orphaned answers reachable from the published documentation.
* A recorded rule for where future consumer answers go.
* No new documentation page unless one is genuinely warranted.

## Non-goals

* **Not creating a FAQ page.** Weighed and rejected below.
* **Not deciding where consumer reports and replies are archived.** They
  currently live in `.git-exclude/tmp/`, which is local, unversioned, and named
  for temporary files, despite one of them having motivated a shipped API
  addition. That is a real problem and a separate one — it concerns repository
  convention, not user documentation, and mixing them would give this RFC two
  unrelated justifications.
* Not documenting the rest of the API surface. That is RFC 032's scope; see the
  note below.
* No code change.

## Design

### 1. `folder_path()` in `docs/src/api-guide.md`

Document it as the inspection seam, positioned with the constructor comparison
RFC 038 added: a caller who wants to keep `new()` can assert the resolved
directory matches expectation rather than switching constructor.

Give the concrete shape — check the final component against the expected
application name — because "you can inspect it" without showing what to inspect
leaves the reader where they started.

### 2. The no-logging stance in `docs/src/introduction.md`

The Non-goals section already states what the crate is not. Add that it does not
take a logging or tracing dependency, and that failures are reported through
`Result` rather than logged — so a caller who wants to know something happened
must ask, and `try_new()` and `for_app()` are how they ask.

This connects a dependency-policy fact to its user-visible consequence. Stated
alone, "no logging dependency" reads as trivia; stated with the consequence, it
answers a question a consumer will actually have.

### 3. The rule in `docs/src/maintainer-notes.md`

Alongside the completion rule and the enum-stability check:

> When a reply to a consumer contains an answer of general interest, put the
> answer in the relevant `docs/src/` page and let the reply cite it. A reply
> reaches one team; the documentation reaches the next one to ask.

Same shape as the other two rules recorded there — a written practice where
tooling cannot help.

### Why not a FAQ page

Considered seriously, because it was the initially proposed shape.

Against:

* **Both answers have natural existing homes.** `folder_path()` is an API method
  and belongs with the API. The logging stance is a non-goal and belongs with the
  non-goals. A FAQ would hold copies or, worse, the only copies — putting API
  documentation somewhere other than the API guide.
* **A fourteenth page with two entries competes with the thirteen.** RFC 035
  established reader paths on the principle that every page appears in exactly
  one; a thin FAQ dilutes that rather than extending it.
* **The project's stated documentation principle is "less is more."** A FAQ is
  the standard place where documentation goes to avoid being organised.

For, and honestly:

* A FAQ is a natural destination for future answers that fit nowhere else, and
  this RFC's rule requires judging "the relevant page" each time.

**Recommendation: no FAQ page now.** If the rule in §3 produces answers that
genuinely fit nowhere, that is the evidence for creating one — and it will then
be created with real content rather than two entries and hope.

### Effect on RFC 032

RFC 032 (M3 slice 3, unwritten) covers API-surface documentation completeness and
names `at_current_dir()` as undocumented. `folder_path()` is the same class of
gap and is handled here instead, so 032's eventual scope is one item smaller. Not
a conflict, but worth recording so the two do not both claim it.

## Compatibility

None affected. Documentation only.

## Security considerations

None. The `folder_path()` guidance is a correctness aid, not a control — a caller
who asserts on the derived directory catches the collision hazard, but nothing
enforces that they do.

## Testing and verification

* Both claims verified against the crate before writing: `folder_path()`'s
  signature and return, and the absence of any logging dependency in
  `Cargo.toml`.
* Reader paths still cover every `docs/src/` page exactly once — unchanged here,
  since no page is added, but worth confirming rather than assuming.
* No code change; existing tests unaffected.

## Risks and unresolved questions

* **"The relevant page" is a judgement call**, and the rule in §3 does not
  eliminate it. Accepted: the alternative is a FAQ that absorbs everything
  without judgement, which is worse.
* **The archival question stays open.** Consumer reports and replies remain in
  `.git-exclude/tmp/`. This RFC deliberately does not address it.

## Alternatives considered

* **A FAQ page.** Weighed above and rejected, for now, with the trigger for
  revisiting stated.
* **Leave both answers in the reply.** Rejected — that is the problem.
* **Fold `folder_path()` into RFC 032 and the logging stance nowhere.** Rejected:
  032 is unwritten and unscheduled, and the logging answer would stay orphaned.

## Acceptance criteria

* `api-guide.md` documents `folder_path()` as the inspection seam, with a
  concrete assertion example.
* `introduction.md`'s Non-goals states the no-logging position and its
  consequence for callers.
* `maintainer-notes.md` records the consumer-answer rule.
* No new `docs/src/` page; reader paths unchanged and still exactly-once.
* No code change, no CI change.
