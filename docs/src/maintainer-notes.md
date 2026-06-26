# Maintainer notes

The project follows a small RFC lifecycle policy under `rfcs/`.

* Proposed RFCs live in `rfcs/proposed/`.
* Implemented RFCs live in `rfcs/done/`.
* Withdrawn or superseded RFCs live in `rfcs/archive/`.
* Optional handoffs live under `rfcs/handoffs/NNN-slug/` and inherit the RFC's state.

Before release:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --no-default-features
scripts/check-rfcs.sh
```

For Windows UWP support, also run a Windows check with the `uwp` feature.
