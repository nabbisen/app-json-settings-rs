# Implementation handoff — RFC 037 Migration guidance for 2.0.x upgraders

**Governing RFC.** [RFC 037](../../done/037-migration-guidance-for-2-0-x-upgraders.md)
**Status.** Inherited from RFC 037 (Implemented).
**Milestone.** M3 — additional slice, ahead of slices 3 and 4.

## Purpose

Tell a consumer still on 2.0.x exactly what to do, including the source break we
shipped and never disclosed.

## Background

Read RFC 037 for the four findings. The one that matters most: `ConfigError`
gained two variants across 2.1.0 and 2.2.0 without `#[non_exhaustive]`, which
breaks any exhaustive `match` downstream. That warranted a major version and got
two minors instead.

**This slice discloses a mistake of ours.** Write it plainly. A reader who hit
`E0004` should learn it was our error, not their misuse.

## Sequencing

Ahead of M3 slices 3 and 4. Those serve internal completeness; this serves a
consumer who has asked a live question.

## Change scope

| File | Change |
|---|---|
| `docs/src/migration-v2.md` | New "Upgrading from 2.0.x" section; disclose the break in two existing sections |
| `README.md` | Move the migration guide between reader paths |
| `docs/src/maintainer-notes.md` | Enum-variant release check |

## Non-change scope

* **`src/` — anything at all.** No code change. In particular **do not add
  `#[non_exhaustive]` to `ConfigError`.** It is breaking, it is a major-version
  decision, and RFC 037 puts it explicitly out of scope.
* `CHANGELOG.md` — this documents history, it does not change it.
* `Cargo.toml`, `Cargo.lock`, CI, tests.
* Other `docs/src/` pages, and the rest of `README.md`.
* The structure of `migration-v2.md` beyond the sections named.

## Required implementation

### 1. "Upgrading from 2.0.x" in `migration-v2.md`

Near the top — it is the first thing an affected reader needs. Cover:

* **Go straight to the latest release; do not upgrade incrementally.** 2.3.0 and
  2.4.0 do not compile on Windows (fixed in 2.4.1). Say why, briefly.
* **The one possible source change** — an exhaustive `match` on `ConfigError`.
  Show the compiler error and the fix.
* **Three behavioral changes crossed**, none needing code edits: default save
  mode became `SaveMode::Atomic` (2.3.0); new files are `0600` on Unix since
  2.5.0 while existing files keep their mode; `new()` no longer panics when the
  executable name is unresolvable (2.1.0), falling back to the literal name
  `app`.
* **On that last one, point affected readers at `for_app()`.** A consumer whose
  settings-directory identity is load-bearing should not use `new()` at all:
  `for_app()` takes an explicit name, so nothing is derived and nothing can fall
  back, and it reports storage-root resolution failure instead of substituting a
  relative path. This addition comes from a real 2.0.x consumer who read the
  fallback as a robustness improvement and found it was the opposite for them.
* **MSRV moved `1.90.0` → `1.85.0`** — a loosening, not a tightening. State it,
  because the opposite is the natural assumption. **Also state the corollary:**
  it only helps if this crate was the binding constraint; a consumer whose floor
  is set by another dependency gains nothing.

### 2. Disclose the break where it happened

`2.0.x to 2.1.0` — `ConfigError::Platform` added.
`2.1.0 to 2.2.0` — `ConfigError::InvalidPathComponent` added.

Both: note that `ConfigError` is not `#[non_exhaustive]`, so this breaks
exhaustive matches, and **that it warranted a major version**.

Do not soften this into "the error type was extended."

### 3. `README.md` reader paths

Move `migration-v2.md` from **"Contributing and maintaining"** to **"Using the
crate"**.

**Verify every `docs/src/` page still appears exactly once** — list the directory
and check off, per RFC 035's method. Do not eyeball it.

### 4. `maintainer-notes.md`

Add, alongside RFC 028's completion rule:

> Adding a variant to a public enum is a breaking change unless the enum is
> `#[non_exhaustive]`. `ConfigError` is not. Check before adding one; if a
> variant is genuinely needed, that is a major-version conversation, not a minor
> release.

## Prohibited shortcuts

* **Do not add `#[non_exhaustive]`.** The single most tempting "fix" here, and
  out of scope for the reasons above.
* Do not touch `src/`, `Cargo.toml`, or CI.
* Do not soften the disclosure.
* Do not transcribe the compiler error from RFC 037 — reproduce it. See below.
* Do not rewrite unrelated parts of `migration-v2.md`.

## Required tests

No crate tests change. Verification is of the claims:

* **Reproduce the compiler error.** Build a throwaway consumer crate depending on
  this crate by path, with an exhaustive `match` over the three 2.0.3-era
  variants, and capture the real `E0004` output. Quote *that*.
* **Confirm the documented fix compiles** — add the `_ =>` arm and rebuild.
* Confirm the three behavioral claims against the changelog and the code before
  writing them.
* Existing tests unchanged.

## Required documentation updates

Covered above.

## Compatibility constraints

None. Documentation only.

## Security constraints

None.

## Known risks

* **Understating the break.** The failure mode is a reader hitting `E0004` and
  concluding the crate is careless, rather than finding the step documented.
* **Scope pull toward `#[non_exhaustive]`.** It is the real fix and it is not
  this slice's to make. Escalate if it feels necessary.

## Required evidence

* The reproduced `E0004` output, and the same code compiling after the `_ =>`
  arm is added.
* `ls docs/src/` alongside README's three reader paths, showing every page
  accounted for exactly once after the move.
* `git diff --stat` showing nothing under `src/`, `Cargo.toml`, or `.github/`.
* CI green.

## Acceptance criteria

* `migration-v2.md` carries the "Upgrading from 2.0.x" section with all four
  points.
* The break is disclosed in both step sections, stated as warranting a major
  version.
* `README.md` lists the migration guide under "Using the crate"; page coverage
  still exactly once each.
* `maintainer-notes.md` carries the enum-variant check.
* The quoted compiler error was reproduced, not transcribed.
* No `src/`, `Cargo.toml`, or CI change.

## Required review-request content

Per §9.2, as a file package at
`.git-exclude/review-request/010-rfc-037-migration-guidance/README.md`. Lead with
the reproduced compiler output.

## Escalate rather than decide

* Adding `#[non_exhaustive]` starts to look necessary within this slice.
* A behavioral claim turns out not to match the code.
* The reader-path move breaks the exactly-once invariant in a way that needs
  restructuring.
* Another undisclosed compatibility break surfaces while writing this.
