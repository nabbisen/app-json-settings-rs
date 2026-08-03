# Implementation handoff — RFC 029 Permission preservation on atomic save

**Governing RFC.** [RFC 029](../../done/029-permission-preservation-on-atomic-save.md)
**Status.** Inherited from RFC 029 (Implemented, 2.5.0).
**Milestone.** M2 — release 2.5.0.
**Implementation order.** First of three. See "Sequencing" below.

## Purpose

Stop atomic save from silently widening a settings file's permissions, and stop
the temporary file from exposing settings content while it is being written.

## Background

Read RFC 029. In short: `rename` swaps the directory entry, so the replaced file
carries the temporary file's umask-derived mode rather than the previous file's.
A file deliberately restricted to `0600` becomes `0644` on the next save. This is
a regression against v2.2 introduced by RFC 024 in 2.3.0.

## Decision recorded by the project owner

**Newly created settings files get `0600` on Unix.**

The rationale, stated accurately because it goes into the changelog:

* Most of this RFC's security value is in *preserving* a deliberate `0600` and in
  closing the temp-file window. Those happen regardless of the new-file default.
* The `0600` new-file default is defence in depth on top of that. Its concrete
  effect is that other local users cannot read the settings — real on a shared
  machine, near-zero on a single-user desktop.
* Existing files are unaffected: preservation keeps whatever mode they have. The
  behavior change is confined to file *creation*.

**Do not overstate this in the changelog.** Do not write that the crate is now
secure, or that settings are protected. RFC 021's checklist asserting an unbuilt
feature worked is the failure mode to avoid.

## Sequencing

Implement **029 first**, then 034, then 030 — serially, not in parallel.

029 and 034 both touch `CHANGELOG.md` and `docs/src/platform-behavior.md`.
Concurrent work conflicts in exactly the files carrying the upgrade narrative for
applications pinned to 2.4.x.

## Change scope

| File | Change |
|---|---|
| `src/core/save.rs` | Temp file created at `0600`; apply target's mode before replace (Unix only) |
| `src/core/save/tests.rs` | **New file.** Unit tests needing access to `save`'s private items |
| `src/core/tests.rs` | Behavior tests through the public API |
| `docs/src/save-behavior.md` | Preservation, fail-secure fallback, caveats |
| `docs/src/platform-behavior.md` | Windows position and its epistemic status |
| `CHANGELOG.md` | 2.5.0 entry |

## Non-change scope

* **All Windows code.** No ACL work, no security descriptors, no new FFI.
* `MoveFileExW`, its flags, and the `unsafe extern` block.
* `SaveMode::Direct` — it already preserves mode by writing in place.
* The atomic algorithm's ordering, temp-file naming, or retry loop.
* **Directory permissions.** `create_dir_all` continues to produce `0755`-typical
  directories. This RFC is about the file. Do not widen scope into directory
  hardening.
* Ownership (uid/gid). Changing a file's owner needs privileges the library will
  not assume.
* The public API. No `with_file_mode()`, no permission builder, no new variant.
* Non-Unix, non-Windows targets.

## Required implementation

All permission work is Unix-only and must be `#[cfg(unix)]`-gated. On every other
platform the save path is byte-for-byte what it is today.

### 1. Create the temporary file restrictively

Use `std::os::unix::fs::OpenOptionsExt::mode(0o600)` when opening the temp file
in `create_temp_file()`. **Use `std` — do not add `libc` or any other dependency,
and do not write `unsafe` for this.**

### 2. Apply the target's mode before replacement

Before calling `replace_file()`: if the target path already exists, read its
permissions and apply them to the temporary file. This must happen **before** the
rename — applying afterwards leaves a window in which the live file has the wrong
mode.

### 3. Best-effort, and that is deliberate

If reading the target's mode or applying it fails, **proceed with the save**. The
temp file is already at `0600`, so the outcome is more restrictive than intended,
never less. The failure mode is fail-secure, which is why a hard error is wrong
here — hard-failing would break portable use on FAT and some network mounts where
permission bits are not meaningful.

### 4. New files

When no target exists, the file keeps the `0600` it was created with. No extra
code — this falls out of step 1.

## Prohibited shortcuts

* Do not apply the mode *after* `rename`. It creates the window this RFC closes.
* Do not hard-fail when `chmod` fails. See step 3.
* Do not `chmod` the directory.
* Do not add `libc`, `nix`, `rustix`, or any other dependency. `std` has what is
  needed.
* Do not write `unsafe` anywhere in this change.
* Do not add a public API to control the mode. The default is the bug.
* Do not attempt any Windows equivalent, even a partial one.
* Do not "fix" `SaveMode::Direct` — it is already correct.

## Required tests

### In `src/core/save/tests.rs` (new file)

`create_temp_file()` is private to the `save` module, so it cannot be reached
from `src/core/tests.rs`. Add `#[cfg(test)] mod tests;` to `src/core/save.rs` and
create `src/core/save/tests.rs`, following the project's documented convention
(`src/some_mod.rs` + `src/some_mod/tests.rs`).

* `create_temp_file()` returns a file whose mode is exactly `0600`. Assert on the
  created file directly — do not infer it from the final saved file, because that
  path also applies the target's mode and would mask a wrong temp mode.
* Clean up the temp file the test creates.

### In `src/core/tests.rs`

All `#[cfg(unix)]`:

* A settings file at `0600` still has `0600` after an atomic save.
* A settings file at `0644` still has `0644` after an atomic save. **This test
  matters as much as the previous one** — it proves preservation works in both
  directions and that the change is not "always restrict".
* A newly created settings file has mode `0600`.
* `SaveMode::Direct` behavior is unchanged.

Existing tests must pass unchanged on all three platforms.

## Required documentation updates

`docs/src/save-behavior.md`:

* Atomic save preserves an existing file's permissions on Unix.
* New files are created `0600`.
* Mode application is best-effort and fail-secure; on filesystems that do not
  model permissions the file stays `0600`.
* **The group-shared caveat**: a settings file under a shared `with_root_dir()`
  path is created owner-only. It bites only on first creation — one `chmod g+r`
  fixes it permanently, because subsequent saves preserve the mode.

`docs/src/platform-behavior.md`:

* Windows is unchanged: per-user `%APPDATA%` is already restricted by directory
  ACL inheritance, and the temp file inherits the same protection.
* **State that this is reasoned from the documented inheritance model and has not
  been verified empirically.** Do not write it as measured fact.

`CHANGELOG.md`: 2.5.0, behavior change, with the accurate framing above.

## Compatibility constraints

* No API change, no signature change.
* Behavior change on Unix only; minor release, not a patch.
* Existing files unaffected — preservation keeps their mode.
* Windows, macOS file-permission semantics, and other targets unaffected.

## Security constraints

This is itself a modest security fix. Do not let it grow into a general
hardening exercise. Do not add claims that the crate is a secret store; the
documentation should continue to point at platform keychains for real secrets.

## Known risks

* **Mixed-UID access.** A CLI run under `sudo` with `HOME` still pointing at the
  user's home creates a root-owned settings file. At `0644` the user could still
  read it; at `0600` they cannot. Already broken either way — the user could not
  write it — but strictly worse for recovery. Document, do not engineer around.
* **`0644` preservation test is the guard** against someone implementing "always
  set `0600`" instead of "preserve, defaulting to `0600`". Do not skip it.

## Required evidence

* Test output showing all new tests passing on Linux.
* Actual observed modes, not assertions alone — e.g. `stat -c %a` output or the
  equivalent printed from a test, for: existing `0600`, existing `0644`, new
  file, and the temp file.
* Full CI matrix green, including macOS and Windows.
* `git diff --stat` confirming the change scope above.

## Acceptance criteria

* Temp files are created at `0600` on Unix.
* An existing target's mode is applied to the temp file before replacement.
* Mode handling is best-effort and cannot leave a file more permissive than
  intended.
* New files are `0600`.
* Tests cover `0600` preservation, `0644` preservation, new-file mode, temp-file
  mode, and unchanged `Direct` behavior.
* No `unsafe`, no new dependency, no Windows change, no directory-mode change.
* Documentation covers preservation, fail-secure behavior, the group-shared
  caveat, and the Windows position marked as reasoned rather than measured.
* Changelog states the behavior change without overstating the benefit.
* CI green on all three platforms.

## Required review-request content

Per §9.2, as a file package under `.git-exclude/review-request/NNN-slug/README.md`.
Include observed modes as raw output — for this RFC the measured permission bits
are the evidence.

## Escalate rather than decide

* Preserving the mode appears to require `unsafe` or a new dependency.
* `OpenOptionsExt::mode` does not behave as expected on the CI Linux or macOS
  runner.
* A test reveals that `SaveMode::Direct` does *not* preserve mode as assumed.
* The change appears to require touching the Windows path.
