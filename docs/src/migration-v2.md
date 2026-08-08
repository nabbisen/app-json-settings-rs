# Migration to v2

The Rust snippets on this page are illustrative, not compiled or run by CI
— see [Testing guide](testing.md#verification-boundary). Some deliberately
show pre-2.5.0 code and would not compile against the current crate even if
verified.

## Upgrading from 2.0.x

Start here if you are still on any 2.0.x release.

**Go straight to the latest release. Do not upgrade incrementally.** 2.3.0
and 2.4.0 do not compile on any Windows target, a defect fixed in 2.4.1. An
incremental upgrade stops dead on Windows at 2.3.0.

**One source change may be required.** `ConfigError` gained two variants
across 2.1.0 and 2.2.0 — see the disclosures below — so an exhaustive
`match` written against 2.0.x will not compile. Reproduced against the
current crate:

```
error[E0004]: non-exhaustive patterns: `&ConfigError::InvalidPathComponent(_)` and `&ConfigError::Platform(_)` not covered
  --> src/main.rs:8:11
   |
 8 |     match error {
   |           ^^^^^ patterns `&ConfigError::InvalidPathComponent(_)` and `&ConfigError::Platform(_)` not covered
```

The fix is one match arm:

```rust
match error {
    ConfigError::Io(_) => "io",
    ConfigError::Serialize(_) => "serialize",
    ConfigError::Deserialize(_) => "deserialize",
    _ => "other",
}
```

**Three behavioral changes cross this upgrade**, none requiring a code
change:

* The default save mode became `SaveMode::Atomic` in 2.3.0 (previously a
  direct overwrite). See [Save behavior](save-behavior.md).
* Newly created settings files are `0600` on Unix since 2.5.0. Existing
  files keep whatever mode they already have. See
  [Save behavior](save-behavior.md#permissions-on-unix).
* `ConfigManager::new()` stopped panicking when the executable name cannot
  be resolved, in 2.1.0. It falls back to the literal name `app` instead.
  **If the settings directory's identity is load-bearing for your
  application, use [`for_app()`](api-guide.md) instead of `new()`.**
  `for_app()` takes an explicit name, so nothing is derived and there is no
  fallback to silently take, and since 2.5.0 it reports storage-root
  resolution failure rather than substituting a relative path.

**Staying on 2.0.x does not avoid silent substitution.** It is easy to read
the previous point as meaning 2.0.x fails loudly and later versions do not.
That is true only of the executable name. 2.0.x resolved the *configuration
directory* with `HOME`, `XDG_CONFIG_HOME`, and `%APPDATA%` each falling back
silently to `.`, so an unresolvable environment already produced a settings
file relative to the process's working directory — with no error and no
panic. Only the executable-name lookup panicked.

That is the more damaging of the two: it misplaces the file entirely rather
than misnaming its directory. If your environment can lack those variables —
services, containers, kiosk sessions — 2.0.x already exposes you to it, and
`for_app()` on 2.5.0 or later is the first version that reports the failure
instead of substituting. This argues for upgrading sooner, not later.

**MSRV moved from `1.90.0` (2.0.3) to `1.85.0` (2.0.4 onward)** — a
loosening, not a tightening. Worth stating plainly because the opposite is
the natural assumption for a version bump. The corollary matters too: the
loosening only helps if this crate was your binding constraint. If your
own MSRV floor is set by a different dependency, this change leaves you
exactly where you were.

## v2.0.x to 2.1.0

`with_root_dir()` was added as the preferred name for caller-provided storage
roots. Existing `at_custom_dir()` code still works.

Pure UWP support is available through either host-resolved roots or the optional
`uwp` feature.

**This release added `ConfigError::Platform`.** `ConfigError` is not
`#[non_exhaustive]`, so adding a variant breaks any exhaustive `match` on
it — a source-breaking change. **This should have shipped as a major
version and shipped as a minor instead.** If you have an exhaustive match
on `ConfigError`, see [Upgrading from 2.0.x](#upgrading-from-20x) above for
the compiler error and the fix.

## 2.1.0 to 2.2.0

`for_app()` was added as the recommended production constructor:

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?;
```

`try_with_filename()` was added for checked file names:

```rust
let manager = manager.try_with_filename("settings.json")?;
```

`with_filename()` remains available for v2.x compatibility.

**This release added `ConfigError::InvalidPathComponent`.** Same issue as
`Platform` in 2.1.0: `ConfigError` is not `#[non_exhaustive]`, so this also
breaks any exhaustive `match` on it. **This should have shipped as a major
version and shipped as a minor instead.** See
[Upgrading from 2.0.x](#upgrading-from-20x) above for the compiler error
and the fix.


## 2.2.0 to 2.3.0

`SaveMode` was added and `save()` now uses `SaveMode::Atomic` by default.

Applications that intentionally want the previous direct overwrite behavior can
select it explicitly:

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?
    .with_direct_save();
```

The public loading, saving, and update methods remain source-compatible.


## v2.4.x to 2.5.0

`ConfigManager::for_app()` now reports storage-root resolution failure
instead of silently substituting a relative path.

**Who is affected.** Only applications running where the platform
configuration directory cannot be resolved: on Unix (excluding macOS) when
neither `XDG_CONFIG_HOME` nor `HOME` is set, on macOS when `HOME` is not set,
and on Windows when `%APPDATA%` is not set. This essentially never happens on
a normal desktop environment. It can happen for services or containers run
without a user environment — for example, a systemd unit without `User=`, or
some minimal container configurations.

**How it shows up.** Previously, `for_app()` would succeed and silently
resolve to a path under the current working directory. Now it returns
`Err(ConfigError::Platform(_))`, with a message naming the missing
environment variable.

**Before:**

```rust
// Succeeded even without HOME/%APPDATA%, silently writing under $CWD.
let manager = ConfigManager::<Settings>::for_app("my-app")?;
```

**After:**

```rust
let manager = match ConfigManager::<Settings>::for_app("my-app") {
    Ok(manager) => manager,
    Err(ConfigError::Platform(message)) => {
        eprintln!("could not resolve a config directory: {message}");
        // Supply a path explicitly instead of relying on platform resolution.
        ConfigManager::<Settings>::new().with_root_dir(chosen_path)
    }
    Err(error) => return Err(error),
};
```

`ConfigManager::new()` is unaffected — it keeps falling back to the current
directory, since it cannot report an error without breaking its signature.
Code that only ever calls `new()` sees no behavior change from this release.
