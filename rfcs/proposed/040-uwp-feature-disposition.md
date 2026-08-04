# RFC 040 — `uwp` feature disposition

**Status.** Proposed
**Tracks.** Whether the optional `uwp` feature is supported, experimental, or withdrawn.
**Touches.** Decision only — implementation depends on which option is chosen.
**Relates to.** [RFC 021](../done/021-uwp-compatible-storage.md), which introduced the feature; [RFC 027](../done/027-windows-build-correctness.md), which repaired it.

## Summary

The optional `uwp` feature has never been verified to work. It compiles, and
that is the entirety of what is known about it. This RFC states the options and
recommends one, but the choice is a compatibility decision reserved for the
project owner.

**No option is implemented here.** This RFC exists to make the decision, not to
pre-empt it.

## The situation

`ConfigManager::at_uwp_local_folder()` resolves
`Windows.Storage.ApplicationData.Current.LocalFolder` and has existed since
2.1.0. Facts, not inference:

* **It did not compile from 2.1.0 to 2.4.0.** RFC 027 found two errors — a
  missing `Storage_Search` cargo feature and `Error::message()` returning
  `String` — that had been present since the feature shipped. For three releases,
  anyone enabling `uwp` got a build failure.
* **It has still never been run.** RFC 027 fixed compilation and said so
  explicitly: the feature moved from "known broken" to "compiles, runtime
  untested." Nothing since has changed that. `ApplicationData::Current()` throws
  without package identity, so verification needs an MSIX-packaged application —
  which CI cannot host.
* **Our own documentation routes users around it.** `docs/src/uwp.md` presents
  host-resolved storage via `with_root_dir()` as *"the recommended pattern"*, and
  the crate resolver as the optional alternative. A reader following our advice
  never enables the feature.

### What it costs to keep

| Cost | Detail |
|---|---|
| Optional dependency | `windows` 0.62 with `Storage` and `Storage_Search` |
| CI job | A Windows-only `cargo check --features uwp` step |
| Conditional code | Two `cfg(all(windows, feature = "uwp"))` blocks in `src/core.rs` and `src/core/dir.rs` |
| Documentation | A page that spends its first paragraphs explaining why not to use the feature it documents |
| Verification debt | An untestable path that survived three releases broken |

None of this is large. The point is that it is all being spent on a path with no
evidence anyone uses it, which our own documentation discourages, and which we
cannot verify.

### What is unknown

**Whether anyone uses it.** crates.io does not report feature-level adoption, so
this cannot be measured. Any claim about usage — in either direction — would be
invention.

## Options

### A — Verify it, then support it properly

Someone with Windows tooling packages a minimal MSIX application, calls
`at_uwp_local_folder()`, and records the result. If it works, the feature becomes
genuinely supported and this RFC closes. If it does not, we have found a real bug
and choose again from a better position.

**Cost:** roughly half a day for someone with Windows dev tooling — MSIX
packaging, an app manifest, likely a developer certificate.
**Blocker:** nobody on this project has that environment.
**Live possibility:** the reply sent to the downstream consumer on 2026-08-04
asked whether this is within reach for them. They volunteered to test a
pre-release against their usage. **Their answer changes this option from
theoretical to available.**

### B — Mark it experimental

Document that the feature compiles but is unverified, in the feature's
documentation and in `Cargo.toml`'s metadata. Costs almost nothing and is honest.

**Weakness:** it is a holding position, not a resolution. "Experimental" with no
plan to leave that state is how features rot — and this one has already spent
three releases broken without anyone noticing.

### C — Deprecate it

Mark `at_uwp_local_folder()` deprecated, pointing at `with_root_dir()`, and
document the feature as retained for compatibility only. Removal would be a
major-version matter and is not proposed here.

**Case for:** our documentation already recommends the alternative. The feature
is a convenience wrapper around one platform call that the host application can
make itself — which is precisely what `docs/src/uwp.md` tells readers to do.
Deprecating it aligns the API with the advice we already give.

**Case against:** `#[deprecated]` warns every user who has enabled the feature,
and we do not know that there are none. It is a compatibility event for anyone
relying on it, in exchange for removing a cost that is small.

### D — Do nothing

Leave it compiling, unverified, and undocumented as such. Rejected: the state is
already recorded as a residual risk in three release reports, and carrying it
silently is what the project has spent this milestone correcting elsewhere.

## Recommendation

**Wait briefly for the consumer's answer, then take A or C.**

If verification is within their reach, take **A** — it is the only option that
converts an unknown into a fact, and every other option is a way of managing not
knowing.

If it is not, take **C**. Between B and C, deprecation is the more honest
resolution: our own documentation has recommended against this feature since RFC
021 shipped it, and a deprecation notice simply makes the API say what the
documentation already says. "Experimental" would leave it in the same
unverified state with a new label.

**Not recommended: deciding this today.** The consumer's answer is outstanding
and materially changes the option set. If no answer arrives within a reasonable
window, that itself is the answer and C follows.

## Non-goals

* **Not removing the feature.** Removal is a major-version matter, separate from
  deprecation, and not proposed here.
* Not changing `with_root_dir()`, which is and remains the recommended path.
* Not adding UWP runtime testing to CI. It cannot host the required environment;
  that is the premise of this RFC, not a gap in it.

## Risks

* **Deprecating something in use.** Unmeasurable, as noted. Mitigated by
  deprecation rather than removal: existing code keeps working and gets a
  warning.
* **Waiting indefinitely for an answer that never comes.** Mitigated by treating
  silence as a decision after a reasonable window rather than leaving this open.
* **Verifying it and finding it broken.** Not a risk but an outcome worth naming:
  it would be the third defect found in this feature, and would argue for C
  regardless.

## Decision required from the project owner

Which of A, B, or C — and, if A, whether to wait for the consumer's response or
seek verification another way.

This is a compatibility decision. Per §2.5 and §6.7 it is not the architect's to
make, and this RFC deliberately implements none of the options.
