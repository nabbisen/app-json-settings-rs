# RFC 036 — Documentation example verification

**Status.** Proposed
**Tracks.** Verification of user-facing code examples, and honesty about what is verified.
**Touches.** `docs/src/` pages, `examples/`, `docs/src/testing.md`.
**Handoff.** [implementation handoff](../handoffs/036-documentation-example-verification/implementation-handoff.md)

## Summary

Move code a reader would copy wholesale into `examples/`, which CI already
compile-checks on three platforms. Keep the remaining documentation fragments
short, illustrative, and explicitly marked as unverified.

Do **not** attempt to compile-check all 28 documentation blocks. Both mechanisms
that would do so degrade the only reading path this project currently has.

## Motivation

No `docs/src/` example is verified by anything. Measured, not assumed — a
throwaway mdbook was initialized against the real crate and run:

```
passed=0  failed=28

25  cannot find type `Settings`
20  cannot find type `ConfigManager`
 6  cannot find value `manager`
 4  unresolved import `app_json_settings`
 2  cannot find module or crate `serde`
 2  cannot find value `chosen_path`
 1  each: `SaveMode`, `ConfigError`, `uwp_local_folder_path`, `temp_dir`
```

Every block is a fragment: no `use` statements, no `Settings` definition, no
`main`, and placeholder identifiers that were never meant to resolve. Not one is
self-contained. `cargo test --doc` reaches exactly one doctest, in `src/lib.rs`.

### Why full verification was rejected

Two mechanisms can verify markdown code blocks. Both were investigated
empirically rather than reasoned about.

**`#[doc = include_str!(…)]` works.** A probe wired two markdown pages onto a
`#[doc(hidden)]` module and `cargo test --doc` picked them up — the block with
hidden setup lines passed, the bare fragment failed as expected. This needs no
`book.toml`, no mdbook, and no new CI job; it rides the existing `cargo test
--doc` step that already runs on all three platforms. It is strictly cheaper
than `mdbook test`.

**But hidden setup lines are not hidden on GitHub.** The `#`-prefix convention
is implemented by rustdoc and mdbook. GitHub renders code fences literally, so
`# use app_json_settings::ConfigManager;` appears as visible noise.

That matters because [RFC 035](../done/035-documentation-currency-and-recovery-example.md)
made the README hard-link every `docs/src/` page, there is no `book.toml`, no
book build, and nowhere the book is published. **Reading the files on GitHub is
currently the only way anyone reads this documentation.**

|  | Verified | Single source | Readable on GitHub |
|---|:-:|:-:|:-:|
| Today | ✗ | ✓ | ✓ |
| Hidden setup lines (`include_str!` or mdbook) | ✓ | ✓ | ✗ — `#` noise |
| mdbook include + anchors | ✓ | ✓ | ✗ — literal directive |

Both verification routes tax the primary reading path permanently. The risk being
mitigated is "a reader copies a short fragment that does not compile" — real, but
mild for a crate whose examples are a handful of lines. Paying for it with
permanent rendering noise is disproportionate.

### What is worth verifying

The blocks a reader pastes into their own project and expects to run. Those
belong in `examples/`, which CI already compiles on Linux, macOS, and Windows.
Everything else is illustration, and illustration should say so.

## Goals

* Code a reader would copy wholesale is compile-checked by existing CI.
* No document claims or implies verification it does not have.
* No new CI tooling, no new job, no new dependency.
* Resolve the recovery-pattern duplication rather than inherit it.
* GitHub rendering unchanged.

## Non-goals

* **Not verifying all 28 blocks.** That is the decision, not an omission.
* **Not publishing the book.** A worthwhile candidate, and the trigger that would
  reverse this RFC's central trade-off — but out of scope here.
* No `book.toml`, no mdbook in CI, no `include_str!` wiring.
* No API or behavior change; no new dependency.
* No restructuring of the `docs/src/` page set.
* Not inventing new examples for their own sake. RFC 026's rule still binds.

## Design

### 1. Classify every block

The test: **would a reader paste this into their project and expect it to run?**

* **Copy-critical** — yes. Belongs in `examples/`.
* **Illustrative** — no; it shows an API shape, a contrast, or a fragment of a
  larger flow. Stays inline, marked.

Two categories are illustrative by force, not by judgement:

* **`uwp.md`** — its examples call `at_uwp_local_folder()`, which is
  `cfg(all(windows, feature = "uwp"))`. It cannot compile on a Linux doctest run
  regardless of how it is written.
* **`migration-v2.md`'s "before" snippets** — they deliberately show pre-2.5.0
  code. Making them compile would be wrong.

### 2. Promote copy-critical code to `examples/`

Documentation references the example by name rather than duplicating it.

**The recovery pattern is the live case.** It exists twice today —
`docs/src/operational-contract.md` carries it as prose, and
`examples/recovery.rs` implements it. RFC 035 introduced that duplication and the
review that approved it did not flag it. Resolve it: `examples/recovery.rs` is
canonical, and the page keeps a short excerpt with an explicit pointer.

Before adding any new example, check `basic.rs`, `custom_root.rs`, `update.rs`,
and `recovery.rs` for existing coverage. **Promotion means moving code that
already exists in two places, or that is genuinely copy-critical — not
manufacturing examples.** RFC 026's maintenance rule is unchanged by this RFC.

### 3. Mark illustrative blocks in prose, not in the fence

Do **not** mark blocks ` ```ignore `. Two reasons:

* Nothing runs rustdoc over these files under this design, so the fence
  annotation would have no effect.
* GitHub does not recognise `ignore` as a language and would drop syntax
  highlighting — a readability regression bought for nothing.

Use prose instead. Where a page is uniformly illustrative — `api-guide.md`,
`migration-v2.md`, `uwp.md` — one sentence near the top is enough and is quieter
than per-block noise. Where a page mixes copy-critical and illustrative content,
mark the individual block.

### 4. State the verification boundary in `testing.md`

`testing.md` currently describes what the crate's tests cover and notes that
`cargo test --examples` keeps examples compiled. It is silent on documentation
fragments, which is how the gap survived unnoticed.

Add the boundary explicitly: `examples/` and `src/` doctests are compile-checked
on all three platforms; `docs/src/` fragments are illustrative and are not.

## Compatibility

None affected. Documentation, plus possible example targets. No API, behavior, or
dependency change.

## Security considerations

None introduced.

## Testing and verification

* `cargo test --examples` passes — existing coverage, now carrying more of the
  documentation's weight.
* Any newly promoted example is **run**, not merely compiled, and its output
  captured.
* Each page is checked for claims of verification it does not have.
* No new CI configuration is added; if this RFC's implementation requires a CI
  change, something has gone wrong.

## Risks and unresolved questions

* **Illustrative fragments stay unverified and can rot.** This RFC reduces
  exposure and removes false confidence; it does not eliminate the risk. Stated
  plainly rather than dressed up.
* **Classification is a judgement call.** Two people could split the 28 blocks
  differently. The stated test — "would a reader paste this and expect it to
  run?" — bounds it, and the two forced-illustrative categories remove the
  hardest cases.
* **Promoting code to `examples/` raises maintenance cost**, which RFC 026
  guarded against. Constrained above to promotion, never invention.

## Alternatives considered

* **Full verification via `include_str!` with hidden setup lines.** Verified to
  work by probe. Rejected: permanent `#` noise on the only reading path that
  exists. This is the option to revisit if the book is ever published.
* **`mdbook test` with a `book.toml`.** Same reader-facing cost as above, plus an
  mdbook install in CI and a new job. Strictly worse than `include_str!` for this
  repository, so rejected for two reasons rather than one.
* **mdbook `{{#include}}` with anchors.** Genuinely single-source and verified,
  but renders as a literal directive on GitHub.
* **Rewriting all 28 blocks self-contained.** Identical everywhere and honest,
  but makes every page substantially more verbose for a small benefit.
* **Do nothing.** Rejected: the documentation would keep implying a coverage it
  does not have.

## Revisit trigger

If the book is published — a CI build, GitHub Pages, and a README pointing at the
book URL rather than raw files — then raw-file reading stops being canonical, the
central trade-off in this RFC reverses, and `include_str!` with hidden setup lines
becomes the right answer. At that point this RFC should be superseded rather than
amended.

## Acceptance criteria

* Every `docs/src/` Rust block is classified copy-critical or illustrative.
* Copy-critical code lives in `examples/` and is referenced, not duplicated.
* The recovery-pattern duplication is resolved, with `examples/recovery.rs`
  canonical.
* Illustrative blocks are marked in prose; no block is marked ` ```ignore `.
* `testing.md` states the verification boundary explicitly.
* `cargo test --examples` passes; any promoted example runs.
* No new CI job, tooling, dependency, or `book.toml`.
* No API or behavior change.
