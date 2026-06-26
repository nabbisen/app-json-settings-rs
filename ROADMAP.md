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

Status: planned.

* Add atomic save mode.
* Define overwrite and rename behavior on Windows, macOS, and Unix.
* Add crash-safety-oriented tests where practical.
* Keep direct-write fallback available if a platform cannot support robust atomic replacement.

## Later candidates

* More examples for GUI frameworks.
* Documentation for packaging models such as MSIX desktop and Pure UWP.
* Optional in-memory test helper if it does not complicate the core API.
