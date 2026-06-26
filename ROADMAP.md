# Roadmap

## v2.2.x — Foundation quality

Status: implemented in v2.2.0.

* Adopt RFC lifecycle policy.
* Add mdBook-compatible documentation.
* Add integration tests for public API behavior.
* Add explicit app identity API.
* Add checked file-name API while preserving v2.x compatibility.
* Add basic CI and RFC integrity script.

## v2.3.x — Save reliability

Status: implemented in v2.3.0.

* Added `SaveMode::Atomic` as the default save strategy.
* Added `SaveMode::Direct` and direct-save builder APIs for compatibility.
* Defined overwrite and replacement behavior on Windows, macOS, and Unix.
* Added reliability-oriented tests around save mode and temporary-file cleanup.

## v2.4.x — Candidate follow-ups

Status: planned.

* Consider recovery guidance for externally corrupted settings files.
* Add GUI-framework examples if they remain small and dependency-light.
* Review docs.rs examples and doctest coverage.

## Later candidates

* More examples for GUI frameworks.
* Documentation for packaging models such as MSIX desktop and Pure UWP.
* Optional in-memory test helper if it does not complicate the core API.
