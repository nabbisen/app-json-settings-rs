# RFC 027 — Windows build correctness

**Status.** Implemented (v2.4.1)
**Tracks.** Platform correctness for the documented Windows target.
**Touches.** `src/core/save.rs`, `src/core/dir.rs`, `Cargo.toml`.
**Handoff.** [implementation handoff](../handoffs/027-windows-build-correctness/implementation-handoff.md)

## Summary

Fix three compile errors that prevent `app-json-settings` from building on
Windows. One breaks the default build for every Windows target. Two break the
optional `uwp` feature. None of them are behavioral changes; the crate has simply
never been compiled for Windows.

## Motivation

The crate documents Windows as a first-class platform. `README.md`,
`docs/src/platform-behavior.md`, and `docs/src/save-behavior.md` describe
`%APPDATA%` resolution and `MoveFileExW`-based atomic replacement.

Neither path compiles.

The default-feature failure was reported by a downstream dependent that hit it
during its first Windows release build. That application is blocked on any
currently published version. The `uwp` failures have been present since the
feature was introduced in 2.1.0 and have never been detected, because the only
Windows CI job fails at the `uwp` step and no plain Windows build job exists.

## Defects

### D1 — `OsStrExt` imported into the wrong scope

`src/core/save.rs:118` places the import inside `replace_file`:

```rust
#[cfg(windows)]
fn replace_file(temp_path: &Path, target_path: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;   // scoped to this function only
    ...
}

#[cfg(windows)]
fn wide_null_terminated(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain([0]).collect()   // trait not in scope
}
```

A function-local `use` does not extend to a sibling function. The call site that
actually needs the trait is in `wide_null_terminated`.

```
error[E0599]: no method named `encode_wide` found for reference `&OsStr`
   --> src/core/save.rs:146:22
```

Affects **2.3.0 and 2.4.0**, default features, every Windows target.
`src/core/save.rs` is unchanged since commit `e68f59a`, so both published
versions carry it identically. The `uwp` feature is irrelevant to this defect —
`replace_file` and `wide_null_terminated` are compiled for any `cfg(windows)`
build.

### D2 — `LocalFolder()` requires an unenabled cargo feature

`src/core/dir.rs:50` calls `ApplicationData::LocalFolder()`. In `windows` 0.62
that method is gated:

```rust
// windows-0.62.2/src/Windows/Storage/mod.rs:169
#[cfg(feature = "Storage_Search")]
pub fn LocalFolder(&self) -> windows_core::Result<StorageFolder> {
```

`Cargo.toml` enables only `features = ["Storage"]`, so the method is not
generated. `StorageFolder` implements query operations from the
`Storage::Search` namespace, which is why the gate exists.

### D3 — `Error::message()` returns `String`, not `HSTRING`

`src/core/dir.rs:58` calls `.to_string_lossy()` on the result of
`error.message()`. Since `windows-result` 0.2 that method returns `String`
directly, which has no `to_string_lossy`.

D2 and D3 affect **2.1.0 through 2.4.0** whenever the `uwp` feature is enabled.

## Goals

* The default build compiles on every Windows target.
* `cargo check --features uwp` compiles on Windows.
* `cargo clippy --all-targets -- -D warnings` passes on Windows.
* No public API change, no behavior change, no new dependency.

## Non-goals

* Do not change Unix or macOS behavior.
* Do not bump the `windows` crate major version.
* Do not add `uwp` to default features.
* Do not add runtime UWP integration testing.
* Do not refactor the save path beyond what the fix requires.

## Design

### D1

Hoist the import to module scope and remove the now-redundant function-local
one:

```rust
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
```

**Removing the local import at line 118 is mandatory, not cosmetic.** Once the
module-scope import exists, the local one becomes an unused import. The project
gate runs `cargo clippy --all-targets -- -D warnings`, which promotes that
warning to an error on Windows. Applying only the hoist leaves Windows CI red for
a new reason. The warning is already emitted today:

```
warning: unused import: `std::os::windows::ffi::OsStrExt`
   --> src/core/save.rs:118:9
```

### D2

```toml
[target.'cfg(windows)'.dependencies.windows]
version  = "0.62"
optional = true
features = ["Storage", "Storage_Search"]
```

`Storage_Search` is a feature of the already-optional `windows` dependency. The
default build, which does not enable `uwp`, is unaffected.

### D3

Drop the `.to_string_lossy()` call:

```rust
ConfigError::Platform(error.message())
```

## Compatibility

No public API change. No persistent format change. No change to the default
dependency set. `ConfigError::Platform` continues to carry a `String`.

Applications currently pinned to 2.3.0 or 2.4.0 on non-Windows platforms see no
difference. Applications on Windows go from "does not build" to "builds".

## Security considerations

None introduced. D2 widens the generated surface of an optional, opt-in
dependency; it does not grant the crate new capabilities or request additional
UWP permissions.

## Testing and verification

Verification runs through the CI matrix defined in RFC 028. Within this RFC:

* `cargo check` must pass for a Windows target.
* `cargo check --features uwp` must pass for a Windows target.
* `cargo clippy --all-targets -- -D warnings` must pass on Windows with and
  without the `uwp` feature.
* Existing tests must continue to pass unchanged on Linux and macOS. No new test
  is required for D1–D3, because the defects are compile-time and the compiler is
  the oracle. Adding a test that asserts "this compiles" would be noise.

Cross-compilation is sufficient for local verification:

```sh
cargo check --target x86_64-pc-windows-gnu
cargo check --target x86_64-pc-windows-gnu --features uwp
```

## Residual risk

**The `uwp` resolver remains unverified at runtime.** This RFC makes
`at_uwp_local_folder()` compile. It does not demonstrate that
`ApplicationData.Current.LocalFolder` resolves correctly inside a real UWP
application container, which CI cannot host. The feature moves from "known
broken" to "compiles, runtime behavior untested". That distinction must not be
blurred in the changelog or release notes.

## Alternatives considered

* **Move `wide_null_terminated`'s body into `replace_file`.** Fixes D1 with one
  fewer function, but the helper is a reasonable seam and inlining it to dodge an
  import is the wrong reason to restructure code.
* **Drop the `uwp` feature entirely.** It has never worked, so removing it would
  cost no user anything today. Rejected: the feature is documented, cheap to
  repair, and RFC 021's design rationale remains sound. Removal would be a
  compatibility decision reserved for the project owner, not a side effect of a
  patch release.
* **Bump `windows` to a newer major version.** Out of scope. The current version
  works once the correct feature is enabled.

## Acceptance checklist

* `src/core/save.rs` imports `OsStrExt` at module scope under `#[cfg(windows)]`.
* The function-local import inside `replace_file` is removed.
* `Cargo.toml` enables `Storage_Search` for the optional `windows` dependency.
* `src/core/dir.rs` no longer calls `to_string_lossy()` on `Error::message()`.
* Windows default build compiles.
* Windows `uwp` build compiles.
* Clippy passes with `-D warnings` on Windows.
* Linux and macOS test results are unchanged.
* No public API, dependency-set, or behavior change is introduced.
* The changelog states that UWP runtime behavior remains untested.
