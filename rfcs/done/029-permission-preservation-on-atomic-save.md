# RFC 029 — Permission preservation on atomic save

**Status.** Implemented (2.5.0)
**Tracks.** Durability and safety of the default save path.
**Touches.** `src/core/save.rs`, `src/core/tests.rs`, `docs/src/save-behavior.md`, `docs/src/platform-behavior.md`, `CHANGELOG.md`.
**Amends.** [RFC 024](../done/024-save-reliability-and-atomic-write-policy.md)
**Handoff.** [implementation handoff](../handoffs/029-permission-preservation-on-atomic-save/implementation-handoff.md)

## Summary

Atomic save discards the previous settings file's permission bits. A file the
user or application deliberately restricted to `0600` becomes umask-default
(`0644` on a typical system) after the next save. Preserve the existing mode
across atomic replacement on Unix, and create the temporary file restrictively so
its contents are never briefly world-readable.

## Motivation

RFC 024 made `SaveMode::Atomic` the default in 2.3.0. Atomic replacement works by
writing a temporary file and `rename`-ing it over the target. `rename` swaps the
directory entry, so the resulting file carries the **temporary file's** ownership
and mode, not the replaced file's. RFC 024's design considered crash-safety and
platform replacement semantics; it did not consider permission preservation.

This is a regression against v2.2 behavior, where `fs::write` on an existing file
preserved that file's mode.

Measured on Linux with `umask 022`:

```
before atomic save: 600
after  atomic save: 644     <- permission silently lost
after  direct save: 600     <- v2.2 behavior preserved it
```

There is a second, smaller exposure: the temporary file is created with default
permissions and the settings content is written into it before replacement, so
the content is readable by other local users for the duration of the write, even
when the final file would have been restricted.

No user has reported this. It was found during architectural review, not in the
field. The crate does not advertise itself as a secret store — but application
settings routinely carry API tokens, and an application that deliberately
restricts its settings file has a reasonable expectation that the library will
not quietly widen it.

## Goals

* Preserve the existing settings file's permission bits across atomic replacement
  on Unix.
* Never create the temporary file more permissively than the final file will be.
* Fail secure: if permission handling cannot be completed, the result must be no
  more permissive than intended, never less.
* Leave Windows behavior unchanged, and explain why.
* Keep the change confined to the save path.

## Non-goals

* **No Windows ACL work.** See [Windows](#windows) below for the reasoning.
* **No ownership (uid/gid) preservation.** Changing a file's owner requires
  privileges the library will not assume. Ownership follows the writing process,
  as it already does.
* No new public API. No `with_file_mode()`, no permission builder.
* No claim that this crate is a credential store.
* No change to `SaveMode::Direct`, which already preserves mode by writing in
  place.
* No change to the atomic algorithm's ordering, temp-file naming, or retry loop.

## Design

### Unix

1. Create the temporary file with mode `0600` from the start, using
   `OpenOptionsExt::mode`. Content is never world-readable during the write.
2. Before replacement, if the target file already exists, read its mode and apply
   it to the temporary file.
3. Replace as today.

**If the target does not exist, what mode should a new settings file get?** This
is a deliberate decision, recorded here rather than left to the implementer:

| Option | Result | Trade-off |
|---|---|---|
| **A — `0600`** | New settings files are owner-only | Safer default; changes today's behavior; would surprise an application that wants a group-readable settings file |
| B — umask default | Matches today (`0644` typically) | No change; a new file holding a token is world-readable by default |

**Recommendation: A.** Settings are per-user state stored under a per-user
directory; owner-only is the honest default, and an application that genuinely
wants wider access can widen it itself after the first save — whereas under B an
application that wants narrower access silently loses it on every save, which is
the bug this RFC exists to fix. This is a behavior change and belongs in the
release notes.

**Decision required from the project owner.**

### Failure handling

Applying the target's mode is **best-effort**, and this is safe by construction.

The temporary file starts at `0600`. If reading the target's mode or applying it
fails — non-POSIX filesystems such as FAT, some network mounts — the file remains
at `0600`, which is *more* restrictive than intended, never less. The failure
mode is fail-secure, so a hard error is not warranted and would break portable-mode
users on filesystems where permission bits are not meaningful anyway.

The residual cost: an application that deliberately made its settings file
group-readable, stored on a filesystem where `chmod` fails, would find it
narrowed to `0600`. That is a functional inconvenience on a filesystem that does
not model permissions, not a security failure. It must be documented.

### Windows

**No change.** Windows does not have mode bits; access is controlled by a
security descriptor — a DACL of access-control entries against SIDs, plus owner
and group SIDs and inheritance flags.

Parity would mean reading the target's security descriptor with
`GetNamedSecurityInfoW` and applying it to the temporary file with
`SetNamedSecurityInfoW` before the move. This is rejected because:

* It requires substantial new `unsafe` FFI with raw `PSECURITY_DESCRIPTOR` and
  `PACL` pointers and `LocalFree` ownership rules. RFC 027 documents what happened
  the last time this project carried hand-written Windows FFI that CI could not
  meaningfully exercise.
* ACL semantics vary across NTFS, network shares, and domain versus local
  accounts. None of that is reachable from CI.
* Setting an owner SID generally requires `SeRestorePrivilege`, so a naive
  implementation would fail or partially apply for ordinary users.
* **The gain is close to zero.** Per-user `%APPDATA%` is already restricted to
  that user by directory ACL inheritance, and the temporary file is created in
  that same directory, so it inherits the same protection. The Unix exposure
  exists precisely because umask-based creation *loses* a deliberate `0600`;
  Windows inheritance already lands in the right place.

The last point is the actual argument — not that parity is hard, but that
inheritance already covers it. **This reasoning is derived from the documented
inheritance model and has not been verified empirically on Windows.** The
documentation must say that rather than assert equivalence.

### Other targets

Unchanged. Targets that are neither `unix` nor `windows` already fall back to a
non-atomic path documented in RFC 024.

## Compatibility

* No API change; no signature changes.
* **Behavior change on Unix**: saved files retain the previous file's mode, and
  newly created files get `0600` if option A is chosen. Minor release, not a
  patch.
* Windows and other targets unaffected.
* `SaveMode::Direct` unaffected.

## Security considerations

This RFC is itself a security fix, of modest severity: it closes a silent
widening of permissions on a file that may hold credentials, plus a transient
window in which content was readable by other local users.

It does not make the crate a secret store. Applications with real secret-handling
requirements should use a platform keychain, and the documentation should
continue to say so.

## Testing and verification

Unix-only tests, gated `#[cfg(unix)]`:

* A settings file at `0600` retains `0600` after an atomic save.
* A settings file at `0644` retains `0644` after an atomic save — preservation
  works in both directions; this is not "always restrict".
* The temporary file is never created more permissively than `0600`. Verify by
  inspecting the mode of the temp file mid-write, not by inference.
* A newly created settings file has the mode chosen by the owner's decision above.
* `SaveMode::Direct` behavior is unchanged.
* Existing save/load tests pass unchanged on all platforms.

Windows and macOS must stay green on the existing matrix. No new Windows test is
added, because no Windows behavior changes.

## Risks and unresolved questions

* **New-file mode is an open decision** (see the table above) and must be settled
  before implementation.
* **A `0600` default could surprise** an application relying on a group-readable
  settings file. Mitigated by release notes and by the fact that such an
  application can widen the file itself.
* **The Windows rationale is reasoned, not measured.** Documented as such.

## Alternatives considered

* **Copy mode after `rename` instead of before.** Leaves a window in which the
  file is live at the wrong mode. Rejected.
* **`with_file_mode(0o600)` builder.** Adds API for an edge case and leaves the
  silent-widening default in place for everyone who does not call it. Rejected —
  the default is the bug.
* **Hard-fail when `chmod` fails.** Rejected: the fallback state is already
  fail-secure, and hard-failing breaks portable use on non-POSIX filesystems.
* **Do nothing, document only.** Rejected: this is a regression we introduced,
  it is silent, and the fix is small on the platform where it is well-defined.

## Acceptance criteria

* Temporary files are created at `0600` on Unix.
* An existing target's mode is applied to the temporary file before replacement.
* Mode application is best-effort and cannot leave the file more permissive than
  intended.
* New-file mode matches the owner's decision.
* Tests cover `0600` and `0644` preservation, temp-file mode, and new-file mode.
* `docs/src/save-behavior.md` documents preservation, the fail-secure fallback,
  and the non-POSIX-filesystem caveat.
* `docs/src/platform-behavior.md` documents that Windows relies on directory ACL
  inheritance and that this is reasoned, not measured.
* `CHANGELOG.md` records the behavior change under a minor version.
* No public API change.
