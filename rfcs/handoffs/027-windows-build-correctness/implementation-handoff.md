# Implementation handoff — RFC 027 Windows build correctness

**Governing RFC.** [RFC 027](../../done/027-windows-build-correctness.md)
**Status.** Inherited from RFC 027 (Implemented, v2.4.1).
**Milestone.** M1 — release 2.4.1.

## Purpose

Make `app-json-settings` compile on Windows, under default features and under the
optional `uwp` feature.

## Background

Three compile errors are described in RFC 027 as D1, D2, and D3. Read the RFC
first; this document does not restate the root-cause analysis.

D1 blocks every Windows build of the published 2.3.0 and 2.4.0 crates and is
currently blocking a downstream application's Windows release. D2 and D3 block
the `uwp` feature and have done so since 2.1.0.

All three are compile-time defects. None of them is a behavior change, and none
requires a design decision. If you find yourself making one, stop and escalate.

## Sequencing

**Land RFC 028's CI matrix before this work.** The matrix is what proves this fix.
The intended sequence produces its own evidence:

1. RFC 028 slice A adds the platform matrix. CI goes red on the Windows job,
   visibly, in our own gate. **Capture that run URL.**
2. This handoff lands. The same job goes green. **Capture that run URL.**

A red run followed by a green run on the same job is the evidence for this RFC.
Do not skip step 1 and assert the fix works.

## Change scope

Only these three files:

| File | Change |
|---|---|
| `src/core/save.rs` | Hoist `OsStrExt` import to module scope; remove the function-local one |
| `Cargo.toml` | Add `Storage_Search` to the optional `windows` dependency's features |
| `src/core/dir.rs` | Remove `.to_string_lossy()` from the `Error::message()` call |

## Non-change scope

Do not touch any of the following:

* The atomic save algorithm, temporary-file naming, or retry loop.
* The Unix or fallback `replace_file` implementations.
* `MoveFileExW` flags or the `unsafe extern` block.
* The public API, in any form.
* `windows` crate version.
* Default features. `uwp` stays opt-in.
* Any file under `docs/`, `examples/`, `tests/`, or `README.md`.
* Formatting of code you did not otherwise change.

## Required implementation

### D1 — `src/core/save.rs`

Add at module scope:

```rust
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
```

Remove the function-local `use std::os::windows::ffi::OsStrExt;` currently inside
`replace_file` at line 118.

**Both edits are mandatory.** After the hoist, the local import is unused, and
`cargo clippy --all-targets -- -D warnings` fails on Windows because of it. The
warning is already emitted today. Applying only the hoist trades one red Windows
build for another.

### D2 — `Cargo.toml`

```toml
[target.'cfg(windows)'.dependencies.windows]
version  = "0.62"
optional = true
features = ["Storage", "Storage_Search"]
```

### D3 — `src/core/dir.rs`

```rust
ConfigError::Platform(error.message())
```

`Error::message()` returns `String` in this version of `windows-result`.

## Prohibited shortcuts

* Do not silence the unused import with `#[allow(unused_imports)]`. Remove it.
* Do not add any crate-level `#![allow(...)]`.
* Do not weaken, skip, or add `continue-on-error` to the clippy step.
* Do not remove the Windows `uwp` check to get a green build.
* Do not add a test whose only purpose is to assert that the crate compiles. The
  compiler is the oracle here; such a test adds maintenance cost and proves
  nothing extra.
* Do not "improve" the UWP resolver while you are in `dir.rs`. Runtime behavior
  is explicitly out of scope.

## Required tests

No new tests. These are compile-time defects, and the platform matrix from RFC
028 is the verification vehicle.

Existing tests must pass unchanged on Linux and macOS. If any existing test
changes behavior as a result of this work, that is a signal something is wrong —
stop and escalate.

## Required documentation updates

None in this handoff, with one exception: the changelog entry for 2.4.1 must
state that the `uwp` feature now **compiles** and that its **runtime behavior
remains untested**. Do not write anything that implies UWP support has been
verified end to end. CI cannot host a UWP container.

## Compatibility constraints

* No public API change.
* No change to the default dependency set. `Storage_Search` is a feature of an
  already-optional dependency.
* No persistent format change.
* `ConfigError::Platform` continues to carry a `String`.

## Security constraints

None introduced. D2 widens the generated surface of an opt-in dependency; it does
not grant the crate new capabilities or request additional UWP permissions.

## Known risks

* The platform matrix may reveal **further** Windows or macOS failures beyond
  D1–D3, since neither platform has ever been built. If that happens, report it.
  Do not absorb unrelated fixes into this handoff.
* Cross-compiling with `x86_64-pc-windows-gnu` verifies compilation but not the
  MSVC toolchain. The CI Windows job covers MSVC; rely on it, not only on local
  cross-compilation.

## Required evidence

Submit all of the following:

```sh
cargo check --target x86_64-pc-windows-gnu
cargo check --target x86_64-pc-windows-gnu --features uwp
cargo clippy --all-targets -- -D warnings
cargo test
```

* Output of the two cross-compile commands, before and after the change.
* CI run URL showing the Windows job **red** before the fix (from RFC 028's
  matrix landing).
* CI run URL showing the full matrix **green** after the fix.
* Confirmation that Linux and macOS test counts are unchanged.

## Acceptance criteria

* Windows default build compiles.
* Windows `uwp` build compiles.
* Clippy passes with `-D warnings` on Windows, with and without `uwp`.
* No unused-import warning remains in `src/core/save.rs`.
* Linux and macOS results unchanged.
* Diff touches exactly the three files listed in Change scope.
* Changelog states UWP runtime behavior remains untested.

## Required review-request content

Follow §9.2 of the organization workflow document:

1. Implementation summary
2. Addressed requirements — reference RFC 027 D1, D2, D3 individually
3. Changed files
4. Important implementation decisions
5. Differences from the approved design
6. Executed tests
7. Test results
8. Build and static-analysis results, including the CI run URLs above
9. Unresolved issues
10. Known limitations — must include the UWP runtime-verification gap
11. Requested review focus

## Escalate rather than decide

Stop and raise a request if any of the following occurs:

* A fix requires touching a file outside the Change scope.
* The matrix reveals a platform defect not covered by D1–D3.
* `Storage_Search` proves insufficient and further `windows` features are needed.
* Any change to the public API appears necessary.
