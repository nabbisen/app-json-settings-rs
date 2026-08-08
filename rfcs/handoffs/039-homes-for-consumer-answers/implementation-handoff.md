# Implementation handoff — RFC 039 Homes for consumer-facing answers

**Governing RFC.** [RFC 039](../../done/039-homes-for-consumer-answers.md)
**Status.** Inherited from RFC 039 (Implemented, 2.7.0).
**Milestone.** M3 — slice 7.

## Purpose

Place two answers that currently exist only in a private reply to one consumer,
and record the rule that keeps future answers out of private replies.

## Background

Read RFC 039. In short: a downstream consumer's audit changed the crate — RFC 038
exists because of it — and most of what we told them reached the documentation
because RFCs 037 and 038 put it there. Two answers did not.

**The RFC weighed a FAQ page and rejected it.** Both answers have natural homes
in existing pages. Do not create one; see the prohibited shortcuts.

## Change scope

| File | Change |
|---|---|
| `docs/src/api-guide.md` | Document `folder_path()` as the inspection seam |
| `docs/src/introduction.md` | No-logging position in Non-goals |
| `docs/src/maintainer-notes.md` | The consumer-answer rule |

## Non-change scope

* **No new `docs/src/` page.** Not a FAQ, not anything else. Reader paths stay
  exactly as they are, still covering every page exactly once.
* Anything under `src/`, `tests/`, `.github/`, `Cargo.toml`.
* `README.md` — reader paths are unchanged because no page is added.
* `CHANGELOG.md` — nothing user-visible at the crate level changes.
* Other `docs/src/` pages, including the constructor comparison RFC 038 added.
  You are adding to it, not rewriting it.
* **The archival question** — where consumer reports and replies live. RFC 039
  explicitly excludes it.

## Required implementation

### 1. `folder_path()` in `docs/src/api-guide.md`

Document it as the **inspection seam**: a caller who wants to keep using `new()`
can assert the resolved directory matches expectation, rather than switching to
`try_new()` or `for_app()`.

Position it with the constructor comparison RFC 038 added — it is the third
remedy for the collision hazard, alongside those two.

**Show what to assert, not merely that you can.** "You can inspect the resolved
path" leaves the reader exactly where they started. Give the concrete shape:
check the final component against the expected application name. A short example
beats a sentence here.

### 2. The no-logging position in `docs/src/introduction.md`

Add to the existing Non-goals section: the crate does not take a logging or
tracing dependency, and failures are reported through `Result` rather than
logged.

**State the consequence, not just the fact.** "No logging dependency" alone reads
as trivia. What a consumer needs to know is that *nothing will ever warn them* —
so a caller who wants to know a fallback fired must ask, and `try_new()` and
`for_app()` are how they ask. That connects the dependency policy to the
behavior the reader actually cares about.

### 3. The rule in `docs/src/maintainer-notes.md`

Alongside the completion rule and the enum-stability check:

> When a reply to a consumer contains an answer of general interest, put the
> answer in the relevant `docs/src/` page and let the reply cite it. A reply
> reaches one team; the documentation reaches the next one to ask.

Wording may be adjusted. The two constraints that must survive: the answer goes
in the documentation, and the reply cites rather than duplicates it.

## Prohibited shortcuts

* **Do not create a FAQ page.** RFC 039 weighed it and rejected it with reasons.
  If it feels like the tidier option while implementing, that reaction is
  anticipated and the answer is still no — the trigger for revisiting is recorded
  in the RFC and it has not fired.
* Do not duplicate either answer across pages. One home each.
* Do not restate the whole collision hazard in `api-guide.md` — RFC 038 already
  documented it in `platform-behavior.md` and `new()`'s rustdoc. Reference, do
  not re-explain.
* Do not touch `README.md`'s reader paths. No page is added, so nothing moves.
* Do not write code.

## Required tests

No crate tests change. Verify the claims before writing them:

* `folder_path()`'s actual signature and return type, from `src/core.rs` — not
  from the RFC.
* That `Cargo.toml` genuinely has no logging or tracing dependency, so the
  non-goal is accurate.
* Reader paths still cover every `docs/src/` page exactly once. No page is added,
  so this should be unchanged — confirm it rather than assume it, using RFC 035's
  method of listing the directory and checking off.
* Existing tests pass unchanged.

## Compatibility constraints

None. Documentation only.

## Security constraints

None. The `folder_path()` guidance is a correctness aid, not a control — a caller
who asserts on the derived directory catches the collision, but nothing enforces
that they do. Do not describe it as protection.

## Known risks

* **Vagueness in item 1.** The failure mode is documenting that inspection is
  possible without showing what to inspect. That would technically satisfy the
  acceptance criterion while leaving the gap open.
* **Scope pull toward a FAQ.** Anticipated and prohibited above.

## Required evidence

* The `folder_path()` signature as read from `src/core.rs`, alongside what you
  wrote, so the reviewer can check they agree.
* `grep -n "log\|tracing" Cargo.toml` showing no such dependency.
* `ls docs/src/` alongside README's reader paths, confirming still exactly-once
  and no page added.
* `git diff --stat` showing nothing outside the three files.
* CI green.

## Acceptance criteria

* `api-guide.md` documents `folder_path()` as the inspection seam, with a
  concrete assertion example rather than a bare statement that inspection is
  possible.
* `introduction.md`'s Non-goals states the no-logging position **and** its
  consequence for callers.
* `maintainer-notes.md` records the consumer-answer rule.
* No new page; reader paths unchanged and still exactly-once.
* No duplication across pages.
* No code, CI, `README.md`, or `CHANGELOG.md` change.

## Required review-request content

Per §9.2, as a file package at
`.git-exclude/review-request/013-rfc-039-consumer-answers/README.md`.

Quote the `folder_path()` example you wrote — it is the part most likely to be
too vague to be useful, and the part the review will look at first.

## Escalate rather than decide

* `folder_path()` turns out not to support the assertion pattern the RFC assumes.
* An answer genuinely fits none of the three pages — that is the FAQ trigger and
  it is a decision, not an implementation call.
* The archival question starts to look like it must be solved to finish this.
