# Testing guide

Tests should use caller-provided temporary roots instead of relying on the host
machine's real application data directory.

```rust
let manager = ConfigManager::<Settings>::new()
    .with_root_dir(temp_dir)
    .try_with_filename("settings.json")?;
```

The crate's own tests cover:

* public constructor behavior;
* custom root behavior;
* checked file-name rejection;
* save/load round trips;
* compact JSON output;
* first-run default creation;
* update persistence;
* invalid JSON error classification;
* default atomic save mode;
* direct save mode selection;
* temporary-file cleanup after successful atomic save;
* preservation of existing files when serialization fails before storage is touched.

The default build should also be tested with no features so the optional UWP
support does not become an accidental default dependency.


Executable examples should stay compile-checked in CI:

```sh
cargo test --examples
```

Examples should avoid writing to real user application data paths. Prefer
caller-provided temporary roots inside examples so users can run them safely.
