# RFC 023 — Test coverage and public API validation

**Status.** Implemented (v2.2.0)
**Tracks.** Test quality and API behavior stability.
**Touches.** `src/core/tests.rs`, `tests/public_api.rs`, `.github/workflows/ci.yml`.

## Summary

Replace the old commented integration test placeholder with real tests that
exercise public API behavior, persistence behavior, invalid JSON handling, and
checked file-name validation.

## Motivation

A settings crate is small, but users depend on stable behavior. The project had
some internal tests after v2.1.0, but integration coverage was missing and an old
commented test file remained in `tests/`.

## Goals

* Add public API integration tests.
* Keep internal unit tests for smaller behavior details.
* Test default feature builds without requiring UWP.
* Confirm JSON serialization mode and update persistence.
* Confirm invalid JSON maps to deserialization errors.

## Non-goals

* Do not add a large test framework.
* Do not add filesystem mocking.
* Do not test actual Pure UWP runtime behavior in portable CI.

## External design

Integration tests live in:

```text
tests/public_api.rs
```

Internal tests stay near implementation details in:

```text
src/core/tests.rs
```

CI runs formatting, clippy, default tests, and no-default-feature tests. A
Windows UWP feature check is included as a Windows-only job.

## Acceptance checklist

* No commented placeholder integration test remains.
* Public API integration tests cover custom root, checked filename, compact JSON,
  round trip, and update behavior.
* Unit tests cover explicit app identity and invalid JSON classification.
* CI workflow exists.
