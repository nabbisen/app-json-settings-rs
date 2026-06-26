# RFC 022 — Documentation and governance baseline

**Status.** Implemented (v2.2.0)
**Tracks.** Project documentation, RFC structure, and contributor navigation.
**Touches.** `README.md`, `docs/src/`, `rfcs/`, `ROADMAP.md`, `CHANGELOG.md`, `NOTICE`.

## Summary

Adopt the RFC lifecycle policy, move implemented RFCs under `rfcs/done/`, add a
state-grouped RFC index, and create mdBook-compatible documentation under
`docs/src`.

## Motivation

The v2.1.0 crate was intentionally small, but the project had almost no durable
project documentation beyond the README. As API seams for UWP, sandboxed hosts,
and future reliability work appear, the project needs enough structure to keep
design decisions findable without making the repository heavy.

## Goals

* Adopt `RFC 000` as the project RFC lifecycle policy.
* Use the four-folder RFC layout: `proposed`, `done`, `archive`, plus optional
  handoffs later.
* Keep README concise.
* Move detailed user and maintainer documentation under `docs/src`.
* Add a short roadmap and release history update.
* Add a `NOTICE` file.

## Non-goals

* Do not introduce a workspace.
* Do not introduce a separate handoff status lifecycle.
* Do not require mdBook as a build dependency.
* Do not turn documentation into exhaustive API reference that duplicates docs.rs.

## External design

The README becomes a landing page with overview, quick start, and links to
`docs/src`.

The documentation tree is:

```text
docs/src/
  SUMMARY.md
  introduction.md
  quick-start.md
  storage-model.md
  api-guide.md
  platform-behavior.md
  uwp.md
  error-handling.md
  testing.md
  migration-v2.md
  maintainer-notes.md
```

The RFC tree follows RFC 000:

```text
rfcs/
  README.md
  proposed/
  done/
  archive/
```

## Acceptance checklist

* `rfcs/done/000-rfc-lifecycle-policy.md` exists.
* `rfcs/README.md` lists RFCs by lifecycle state.
* Implemented v2.1.0/v2.2.0 RFCs live under `rfcs/done/`.
* README is concise and links to full docs.
* `docs/src/SUMMARY.md` exists and lists user/maintainer pages.
* `ROADMAP.md`, `CHANGELOG.md`, and `NOTICE` exist.
