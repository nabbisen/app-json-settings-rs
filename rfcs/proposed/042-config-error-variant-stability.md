# RFC 042 — `ConfigError` variant stability

**Status.** Proposed
**Tracks.** Public API stability, and the ledger of work blocked on it.
**Touches.** `src/core/error.rs`, `docs/src/error-handling.md`, `docs/src/migration-v2.md`, `docs/src/maintainer-notes.md`, `CHANGELOG.md` — at a major version only.
**Relates to.** [RFC 038](../done/038-fail-closed-constructor.md), whose design was constrained by this; [RFC 041](../done/041-permission-policy-for-pre-existing-modes.md), which deferred a slice here.

## Summary

`ConfigError` is not `#[non_exhaustive]`, so adding a variant breaks any
exhaustive `match` downstream. Two pieces of design work have now been
constrained by this — one shipped with a workaround, one deferred — and there is
no single place recording what is waiting.

This RFC is that place. It proposes adding `#[non_exhaustive]` to `ConfigError`
at the next major version, and serves as the standing ledger of variants that
cannot be added until then.

**It does not propose scheduling a major version.** That is the owner's
decision and is deliberately out of scope.

## Motivation

### The project has already broken this rule twice

`docs/src/maintainer-notes.md` records it: `ConfigError::Platform` (2.1.0) and
`ConfigError::InvalidPathComponent` (2.2.0) both shipped as *minor* releases,
and both break an exhaustive `match` downstream. `docs/src/migration-v2.md`
carries the disclosure.

That history is why the rule now exists. The consequence of honouring it is that
genuine design needs go unmet rather than being met by another silent break —
which is correct, but it means the unmet needs accumulate somewhere, and until
now that somewhere was nowhere.

### Why a ledger rather than an annotation

RFC 000 defines four statuses and makes the folder the source of truth. An RFC
whose work is partly shipped and partly waiting on an unscheduled major cannot
be honestly filed in either `proposed/` or `done/`. Recording a deferral *inside*
the RFC that spawned it therefore blocks that RFC from ever closing cleanly, and
produces exactly the "Status field that lies" state RFC 000 names as the
anti-pattern.

Deferred items must therefore move here, so the originating RFC can close on the
work it actually delivered.

### Consumer position

The orbok team has raised the missing attribute twice, in their first consumer
report and again on 2026-08-12, and stated they would support the change
whenever a major happens. They work around it today by never matching on the
type. That is one consumer, not a mandate, but it is the only downstream
position recorded.

## The ledger

Variants wanted and currently unbuildable. **Add to this table rather than
recording the deferral elsewhere.**

| Wanted variant | Wanted by | Shipped instead | Status |
|---|---|---|---|
| "Executable name unusable" | [RFC 038](../done/038-fail-closed-constructor.md), `try_new()`, 2.6.0 | Reports through `ConfigError::Platform` with a message | Shipped with workaround |
| "Insecure file permissions" | [RFC 041](../done/041-permission-policy-for-pre-existing-modes.md), Alternatives — "Report the condition rather than repair it silently" | Nothing — since 2.8.0 the condition is repaired silently and never surfaced | Deferred, unreported |

Both cases share a shape: a distinct failure the caller cannot distinguish
programmatically, only by string-matching a message, which is not an API.

## Goals

* Make `ConfigError` extensible without a further breaking change.
* Give every blocked item one visible home, so nothing is lost to an RFC that
  closed.
* Add the accumulated variants in the same major, so consumers absorb one
  disruption rather than several.

## Non-goals

* **No scheduling of a major version.** Owner's decision.
* No change to any existing variant's name, meaning, or payload.
* No change before a major. This RFC ships nothing on its own.
* No new dependency.

## Design

At the next major version:

1. Add `#[non_exhaustive]` to `ConfigError`.
2. Add the variants in the ledger above, each with the RFC that wanted it.
3. Document in `docs/src/migration-v2.md`'s successor that exhaustive matches
   need a wildcard arm, and that this is the last time such a break is needed.

Adding `#[non_exhaustive]` is itself a breaking change — a downstream exhaustive
`match` stops compiling — which is precisely why it cannot be slipped into a
minor and why it must be batched with the variants rather than done first.

## Compatibility

* Breaking. Major version only.
* After it lands, further variants are additive and this class of blockage ends.

## Testing and verification

* Existing error tests stay green.
* A compile test or documented example showing that a wildcard arm is required.
* Each added variant is constructible and reachable by the condition that wanted
  it — a variant nothing can produce is worse than no variant.

## Risks and unresolved questions

* **The major is unscheduled**, so items may sit here for a long time. That is
  the honest state and is better represented here than as a stalled RFC.
* **The ledger only works if it is used.** `docs/src/maintainer-notes.md`'s enum
  stability rule must point here, or the next blocked item is recorded somewhere
  ad hoc and lost again.
* **Batching argues for waiting; unreported conditions argue for going sooner.**
  RFC 041 slice 2 means an insecure-permission condition is repaired silently and
  never surfaced. That is a real if minor cost of deferral, not a free wait.

## Alternatives considered

* **Add variants in a minor, as 2.1.0 and 2.2.0 did.** Rejected. The project
  documented that as a mistake; repeating it knowingly is worse than having done
  it unknowingly.
* **Report through existing general variants with messages.** Already done once
  for `try_new()`. Rejected as a pattern: it makes string-matching the only
  discrimination mechanism, and each repetition entrenches it further.
* **Record deferrals in the RFC that spawned them.** Rejected — blocks that RFC
  from closing, per RFC 000. This is the reason this RFC exists as a ledger.
* **`#[non_exhaustive]` now, variants later.** Rejected: two breaking releases
  where one suffices.

## Acceptance criteria

Nothing ships from this RFC before a major version. Until then:

* `docs/src/maintainer-notes.md`'s enum stability rule names this RFC as where a
  blocked variant is recorded.
* Any RFC that defers work on these grounds adds a row to the ledger and does not
  retain the deferred slice itself.

At the major:

* `ConfigError` is `#[non_exhaustive]`.
* Every ledger row is either implemented or explicitly withdrawn with a reason.
* Migration documentation covers the wildcard-arm requirement.
