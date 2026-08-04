# RFC 040 — `uwp` feature disposition

**Status.** Proposed
**Tracks.** Whether the optional `uwp` feature is supported, experimental, or withdrawn.
**Touches.** Decision only — implementation depends on which option is chosen.
**Relates to.** [RFC 021](../done/021-uwp-compatible-storage.md), which introduced the feature; [RFC 027](../done/027-windows-build-correctness.md), which repaired it.

## Summary

The optional `uwp` feature has never been verified to work. It compiles, and that
is the entirety of what is known about it.

**Decided: verify it once, manually** (option A). The options and the reasoning
that led there are recorded below, followed by the verification procedure. The
result of that run closes this RFC, or reopens the choice in favour of
deprecation.

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

**Chosen by the project owner**, as a single manual run rather than CI
automation. The procedure is in a section of its own below.

**Why not CI.** Getting package identity on a GitHub Actions Windows runner is
plausible but not cheap: either build and install an MSIX (packaging tools, a
signing certificate, trusting it, then capturing output from a packaged process),
or borrow an installed package's identity with `Invoke-CommandInDesktopPackage`,
which runs detached so results must go through a file. Both are real engineering,
and both are fragile in a way that produces flaky CI — for a six-line code path
that has not changed since 2.1.0 apart from RFC 027's fix.

A single recorded run converts *never verified* into *verified once, at a stated
version, by a stated person*. That is a genuine improvement, and the regression
risk is small because the code does not move. **It is not continuous verification
and must not be described as such.**

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

## Decision

**Option A, by the project owner** — verified once, manually, per the procedure
below.

A is the only option that converts an unknown into a fact; B and C are both ways
of managing not knowing. B in particular would leave the feature in exactly its
current state with a new label, and it has already spent three releases broken
without anyone noticing — which is what that label tends to produce.

**If verification fails**, C becomes the recommendation and the failure is the
argument for it.

## Verification procedure

**Status of this procedure: reasoned, not tested.** Nobody on this project runs
Windows. The probe below is confirmed to compile for
`x86_64-pc-windows-gnu --features uwp`; the identity mechanics are from
documented Windows behavior and may need adjusting on the day.

### What it verifies, and what it does not

* **Verifies:** that `at_uwp_local_folder()` resolves without error, returns a
  usable path, and that a save-then-load round trip works in that directory.
* **Does not verify:** behavior inside a real packaged application of our own. If
  identity is borrowed from another installed package, `LocalFolder` returns that
  package's directory — which still exercises the whole call chain, since the API
  contract is "the current package's local folder," but is not an end-to-end test.

### The probe

Confirmed to compile. `Cargo.toml`:

```toml
[dependencies]
app-json-settings = { version = "2.6", features = ["uwp"] }
serde = { version = "1", features = ["derive"] }
```

`src/main.rs`:

```rust
use app_json_settings::ConfigManager;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Default, Debug, Serialize, Deserialize)]
struct Probe { runs: u32 }

fn main() {
    let mut out = String::new();
    match ConfigManager::<Probe>::new().at_uwp_local_folder() {
        Ok(manager) => {
            out.push_str(&format!("resolved path: {}\n", manager.path().display()));
            match manager.update(|p| p.runs += 1) {
                Ok(v) => out.push_str(&format!("save + load OK: {v:?}\n")),
                Err(e) => out.push_str(&format!("save + load FAILED: {e}\n")),
            }
        }
        Err(e) => out.push_str(&format!("at_uwp_local_folder FAILED: {e}\n")),
    }
    let dir = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
    let target = format!("{dir}\\uwp-probe-result.txt");
    if let Ok(mut f) = std::fs::File::create(&target) {
        let _ = f.write_all(out.as_bytes());
    }
}
```

It writes to a file rather than stdout deliberately: a process launched with
borrowed package identity is detached, so console output is not reliably
capturable.

### Running it with package identity

`ApplicationData::Current()` throws without package identity, so a plain `.exe`
run proves nothing except that it throws. Give it identity by whichever route is
cheapest on the machine:

1. `cargo build --release`
2. List installed packages:
   `Get-AppxPackage | Select-Object Name, PackageFamilyName`
3. Run the probe inside one package's identity using
   `Invoke-CommandInDesktopPackage`, supplying the package family name, its
   `AppId`, and the path to the probe executable.
4. Read the result file written under `%USERPROFILE%`.

If that cmdlet proves awkward, a minimal MSIX or a sparse package achieves the
same thing at higher setup cost. **Any route that gives the process package
identity is acceptable** — the identity is the point, not the packaging method.

### What to record

The result file's contents verbatim, the crate version tested, the Windows
version, and which identity route was used. That becomes this RFC's evidence.

### Outcomes

* **Resolves and round-trips** — the feature is verified. This RFC closes under
  A, the documentation drops its "runtime untested" caveat, and the state is
  recorded as *verified once at a stated version*, not continuously.
* **Fails** — a third defect in this feature, found the first time anyone ran it.
  That is strong evidence for option C, and the failure is the argument.

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

## Remaining

The verification run itself, by the project owner. Its result closes this RFC
under A, or reopens the choice in favour of C.
