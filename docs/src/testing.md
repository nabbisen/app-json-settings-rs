# Testing guide

Tests should use caller-provided temporary roots instead of relying on the host
machine's real application data directory.

```rust
let manager = ConfigManager::<Settings>::new()
    .with_root_dir(temp_dir)
    .try_with_filename("settings.json")?;
```

The crate's own tests cover, by category:

* constructor and storage-root behavior, including checked file names and
  storage-root resolution failure;
* save and load round trips, including JSON formatting options and error
  classification;
* atomic save mode selection, temporary-file handling, and Unix permission
  preservation;
* update and first-run-default persistence.

The tests live in `src/core/tests.rs`, `src/core/save/tests.rs`,
`src/core/dir/tests.rs`, and `tests/public_api.rs`. Read those files for the
current, exact set of cases — an enumerated list here would go stale the next
time a test is added.

The default build should also be tested with no features so the optional UWP
support does not become an accidental default dependency.


Executable examples should stay compile-checked in CI:

```sh
cargo test --examples
```

Examples should avoid writing to real user application data paths. Prefer
caller-provided temporary roots inside examples so users can run them safely.

## Verification boundary

`examples/` and the doctest in `src/lib.rs` are compiled and run by CI on
Linux, macOS, and Windows. The Rust snippets scattered through the rest of
`docs/src/` are illustrative fragments — they show API shape, not complete
programs — and nothing compiles or runs them. Where a page's snippet
overlaps what an example already demonstrates, the example is canonical;
the snippet is there for reading context, not as a copy source.
