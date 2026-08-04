# RFC 037 — Migration guidance for 2.0.x upgraders

**Status.** Proposed
**Tracks.** Upgrade guidance, and honest disclosure of a compatibility break we shipped.
**Touches.** `docs/src/migration-v2.md`, `README.md`, `docs/src/maintainer-notes.md`.
**Handoff.** [implementation handoff](../handoffs/037-migration-guidance-for-2-0-x-upgraders/implementation-handoff.md)

## Summary

Document the source-breaking change this project shipped inside the 2.x line, add
upgrade guidance for consumers still on 2.0.x, correct where the migration guide
is surfaced, and add a release check so the same class of break is not repeated.

No code change. The break itself is already published and cannot be undone.

## Motivation

A downstream team on 2.0.3 asked what they need to care about before upgrading.
Investigating produced four findings, one of which is a mistake of ours that has
never been written down.

### F1 — `ConfigError` gained variants without `#[non_exhaustive]`

`ConfigError::Platform` was added in 2.1.0 and `ConfigError::InvalidPathComponent`
in 2.2.0. The enum is not marked `#[non_exhaustive]`, so adding variants is a
source-breaking change requiring a major version. **We shipped it as two minor
releases.**

Verified rather than reasoned about — 2.0.3-era consumer code compiled against
2.5.1:

```
error[E0004]: non-exhaustive patterns:
  `ConfigError::InvalidPathComponent(_)` and `ConfigError::Platform(_)` not covered
```

The break surface is exactly one thing: an exhaustive `match` on `ConfigError`.
Everything else in the same probe compiled unchanged — `at_custom_dir`,
`with_filename`, `load_or_default`, `save`, `update`. The fix for a consumer is a
single `_ =>` arm.

### F2 — The migration guide never mentions it

`docs/src/migration-v2.md`'s `2.0.x to 2.1.0` section describes `with_root_dir()`
being added and `at_custom_dir()` still working. It says nothing about the
`Platform` variant — the only actually breaking part of that step. The
`2.1.0 to 2.2.0` section has the same omission for `InvalidPathComponent`.

The guide documents the additive changes and omits the breaking one.

### F3 — No guidance for 2.0.x, and incremental upgrade hits a wall

**2.3.0 and 2.4.0 do not compile on Windows** — the defect RFC 027 fixed in
2.4.1. A team upgrading step by step from 2.0.x would stop dead at 2.3.0 on
Windows. We know this and have never said it. They should go straight to the
latest release.

### F4 — The migration guide is filed under the wrong audience

`README.md`'s reader paths, introduced by RFC 035, place
`docs/src/migration-v2.md` under **"Contributing and maintaining"**. A migration
guide is for users upgrading, not for contributors. It belongs under "Using the
crate".

This one is the architect's error: RFC 035's handoff specified that grouping.

## Goals

* A consumer on any 2.0.x version can read one section and know what to do.
* The compatibility break is disclosed where it happened, not buried.
* The migration guide is reachable by the audience that needs it.
* The same class of break becomes harder to repeat.

## Non-goals

* **Not adding `#[non_exhaustive]` to `ConfigError`.** It would prevent
  recurrence, but it is itself a breaking change — it forces `_ =>` on every
  downstream match — so it belongs in a major version. That decision is the
  project owner's and is explicitly out of scope here.
* No code change of any kind.
* No yank, no re-release, no retroactive version renumbering. The break is
  published; the remedy is disclosure.
* No restructuring of `migration-v2.md` beyond the sections named.
* Not documenting every 2.x change. This is upgrade guidance, not a second
  changelog.

## Design

### 1. An "Upgrading from 2.0.x" section

Placed near the top of `migration-v2.md`, because it is the first thing an
affected reader needs:

* **Go straight to the latest release.** Do not upgrade incrementally: 2.3.0 and
  2.4.0 do not compile on Windows (fixed in 2.4.1, see RFC 027).
* **One source change may be required** — an exhaustive `match` on `ConfigError`
  needs a `_ =>` arm or the two new variants. Show the compiler error verbatim so
  a reader recognises it, and the fix.
* **Three behavioral changes crossed**, none requiring code changes:
  * default save mode became `SaveMode::Atomic` in 2.3.0;
  * newly created settings files are `0600` on Unix since 2.5.0, existing files
    keep their mode;
  * `ConfigManager::new()` no longer panics when the executable name cannot be
    resolved (2.1.0); it falls back to the literal name `app`. **If the settings
    directory's identity is load-bearing for the application, use `for_app()`
    instead** — it takes an explicit name, so no derivation happens and there is
    nothing to fall back from, and it reports storage-root resolution failure
    rather than substituting a relative path.
* **MSRV moved from `1.90.0` in 2.0.3 to `1.85.0`** — a loosening, not a
  tightening. Worth stating because the opposite is the reasonable assumption.
  The corollary is worth stating too: the loosening only helps if this crate was
  the binding constraint. A consumer whose floor is set by another dependency is
  left where it was.

### 2. Disclose the break at the steps where it happened

Add to `2.0.x to 2.1.0`: `ConfigError::Platform` was added, and because
`ConfigError` is not `#[non_exhaustive]`, this breaks exhaustive matches. Same
for `InvalidPathComponent` under `2.1.0 to 2.2.0`.

**State plainly that this should have been a major version.** Do not soften it
into "the error type was extended". A reader who hit a compile error deserves to
know it was our mistake, not their misuse.

### 3. Move the migration guide to the right reader path

In `README.md`, move `migration-v2.md` from "Contributing and maintaining" to
"Using the crate". Every page must still appear exactly once — the invariant RFC
035 established.

### 4. A release check against recurrence

Add to `docs/src/maintainer-notes.md`, alongside RFC 028's completion rule:

> Adding a variant to a public enum is a breaking change unless the enum is
> `#[non_exhaustive]`. `ConfigError` is not. Check before adding one, and if a
> variant is genuinely needed, that is a major-version conversation, not a minor
> release.

This is the RFC 028 pattern: a written rule where tooling cannot help. Nothing in
CI can detect a semver-breaking enum addition here.

### Publication note

`docs/src/` is packaged into the crate but rendered nowhere — it reaches users
through GitHub, so migration-guide changes are live on merge and need no release.
The `README.md` change is different: `readme = "README.md"` makes it the crates.io
landing page, so F4's fix reaches users only when a release is next cut. It does
not justify a release on its own and should ride the next one.

## Compatibility

None affected. Documentation only.

## Security considerations

None.

## Testing and verification

* The compiler error quoted in the guide is reproduced against the current crate
  before publication, not copied from this RFC.
* The `_ =>` fix is confirmed to compile.
* README reader paths still cover every `docs/src/` page exactly once — check by
  listing the directory, per RFC 035's method.
* No code change; existing tests unaffected.

## Risks and unresolved questions

* **Disclosing our own semver break is uncomfortable and correct.** The risk of
  understating it is that a consumer hits `E0004` and concludes the crate is
  careless rather than that this specific step is documented.
* **`ConfigError` remains not `#[non_exhaustive]`,** so the next variant repeats
  the break. The written rule reduces the chance; only a major version removes
  it. Left open deliberately.
* **F4's fix is not visible until a release.** Acceptable — the guide itself is
  reachable on GitHub immediately.

## Alternatives considered

* **Add `#[non_exhaustive]` now.** Rejected as out of scope: breaking, and a
  major-version decision.
* **Say nothing; the changelog already lists the variants.** Rejected — the
  changelog records that variants were *added*, not that adding them broke
  downstream matches. A consumer reading it would not predict `E0004`.
* **Yank 2.1.0 and 2.2.0.** Rejected as disproportionate and useless: the
  variants exist in every later release too.
* **Write a standalone upgrade document.** Rejected: `migration-v2.md` already
  exists for this and a second document would split the audience.

## Acceptance criteria

* `migration-v2.md` has an "Upgrading from 2.0.x" section covering the jump
  target, the `ConfigError` break with its verbatim compiler error and fix, the
  three behavioral changes, and the MSRV clarification.
* The break is disclosed in the `2.0.x to 2.1.0` and `2.1.0 to 2.2.0` sections,
  stated as a change that warranted a major version.
* `README.md` lists the migration guide under "Using the crate"; every page still
  appears exactly once.
* `maintainer-notes.md` carries the enum-variant release check.
* The quoted compiler error is reproduced, not transcribed.
* No code change, no CI change, no new dependency.
