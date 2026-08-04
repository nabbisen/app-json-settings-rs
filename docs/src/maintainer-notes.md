# Maintainer notes

The project follows a small RFC lifecycle policy under `rfcs/`.

* Proposed RFCs live in `rfcs/proposed/`.
* Implemented RFCs live in `rfcs/done/`.
* Withdrawn or superseded RFCs live in `rfcs/archive/`.
* Optional handoffs live under `rfcs/handoffs/NNN-slug/` and inherit the RFC's state.

## Completion rule

An RFC does not move to `rfcs/done/` while the release gate for its change is
red. If the gate cannot be made green, the RFC stays in `proposed/` and the
blocking failure is recorded in it. A checklist item is marked complete only
against a run that was actually observed, not against an expected result.

## Enum stability check

Adding a variant to a public enum is a breaking change unless the enum is
`#[non_exhaustive]`. `ConfigError` is not. Check before adding one, and if
a variant is genuinely needed, that is a major-version conversation, not a
minor release.

This rule exists because `ConfigError` already broke it once:
`ConfigError::Platform` (2.1.0) and `ConfigError::InvalidPathComponent`
(2.2.0) both shipped as minor releases and both break an exhaustive
`match` downstream. See [Migration to v2](migration-v2.md) for the
disclosure and upgrade guidance.

## Consumer-answer rule

When a reply to a consumer contains an answer of general interest, put the
answer in the relevant `docs/src/` page and let the reply cite it. A reply
reaches one team; the documentation reaches the next one to ask.

## Release gate

The CI workflow runs the following on every push:

* On Linux, macOS, and Windows: `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, `cargo test --no-default-features`, `cargo test --examples`,
  and `cargo test --doc`.
* On Linux only: `cargo fmt --check` and `scripts/check-rfcs.sh`. Formatting
  and RFC integrity are platform-independent, so running them once is enough.
* On Windows only: `cargo check --features uwp`, since the optional `uwp`
  feature is Windows-specific.
* Pinned to the declared MSRV (`rust-version` in `Cargo.toml`): `cargo check`
  and `cargo check --all-targets`.

Before release, run the same set locally:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --no-default-features
cargo test --examples
cargo test --doc
scripts/check-rfcs.sh
```

For Windows UWP support, also run a Windows check with the `uwp` feature. This
crate's platform-specific behavior lives in code that only builds under
`cfg(windows)` or `cfg(unix)`; cross-compilation checks compilation but not
runtime behavior, so the CI matrix running tests on each target platform is
the authoritative signal, not a local cross-compile.
