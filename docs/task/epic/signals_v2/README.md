# Epic: Signals v2

## Status

Planning, started 2026-09-09. This epic collects naming, performance, routing,
and ownership improvements for the engine's unified signal machinery. It does
not authorize one large rewrite: each workstream should become a focused task
with measurements and compatibility boundaries before implementation.

## Purpose

Mittens currently represents two importantly different lanes with `Signal`:

- **events** are observed facts. They are delivered to global handlers and to
  scoped handlers along the event source's ancestor chain;
- **intents** are directed requests for engine work. They have one recipient,
  may be scheduled, pass through recipient-owned pipeline operators, and are
  interpreted or applied at explicit drain points.

`CommandQueue` is a temporary `Vec<Signal>` staging buffer needed where Rust
borrowing prevents direct access to the signal runtime. MMS handlers register
event routes that enqueue callback invocations only when a matching handler is
actually dispatched.

Signals v2 should make these roles obvious in names, make the unobserved-event
path close to free, and review operators according to the lane they actually
serve.

## Current performance model

An event producer currently constructs and enqueues an `EventSignal` whether or
not that event kind has observers. At the next drain point, `RxWorld`:

1. removes the event from the ready queue;
2. looks for global `Any` and kind-specific handlers;
3. computes `source -> parent -> ... -> root`;
4. looks for scoped `Any` and kind-specific handlers at each node.

If there is no MMS handler, the event does **not** cross the script boundary:
payload conversion, callback-invocation allocation, callback queueing, and MMS
execution occur only inside an installed handler closure. The remaining cost is
native signal construction, queueing, draining, hash lookups, and the ancestor
walk.

Eye tracking is a useful high-frequency example. With no registered
eye-tracking components, the system iterates empty source sets and emits no eye
events. With `XREyeTracking` in the world, source polling and component-sample
updates occur for AVC regardless of script observers. New source data currently
emits eye events even when nobody handles them. AVC reads the retained component
samples directly; it does not require those observation events.

The standard OSC eye source already bounds event production to the newest
sample per frame/source. The HTC source currently emits for each decoded packet
and should be included in the coalescing review.

## Workstream A: make unobserved events cheap

Start with the common case: no handler anywhere for an event kind.

### First slice

Add an O(1) interest query maintained by handler registration and removal. It
must account for both the concrete `SignalKind` and `SignalKind::Any`, and for
both engine and MMS handlers. Candidate API:

```rust,ignore
signals.has_event_interest(SignalKind::XrEyeTrackingUpdated)
```

High-frequency producers may use that query to avoid constructing and
enqueueing optional observation events. State updates and other system behavior
must continue even when event interest is zero. Never use handler interest to
skip an intent: intents request effects and are not optional observations.

An initial implementation can count handlers anywhere in the world by kind.
That safely eliminates production only when the count is zero. It does not need
to solve scope ancestry yet.

Also add a dispatch fast path: if there are no global or scoped handlers for
the concrete kind or `Any`, return before computing the source ancestor chain.
This protects producers that cannot or should not consult the signal runtime.

### Later refinement

A world-wide count cannot reject an event when handlers of the same kind exist
only in unrelated trees. A scope-aware interest query would need to respect
ancestor bubbling and topology changes. Possible approaches include:

- maintain subtree/ancestor subscription summaries and update them on handler
  or topology changes;
- cache `(kind, source) -> has reachable observer` with explicit invalidation;
- accept queueing but avoid the scope walk when the relevant handler maps prove
  no delivery is possible.

Do not introduce a topology-maintenance structure until measurements show that
the simple zero-subscriber fast path is insufficient.

### Event-production and coalescing policy

Classify event kinds before optimizing them:

- **edge/lifecycle events** such as press, release, mount, and dismount normally
  must preserve every occurrence and ordering;
- **latest-state events** such as gaze, pointer motion, or continuous telemetry
  may often retain only the newest value per source and frame;
- **dirty notifications** may often coalesce to one event per affected owner at
  a declared drain boundary.

Coalescing must be explicit per event kind. A generic queue must not silently
drop repeated lifecycle events.

### Measurement

Add opt-in counters or tracing for:

- events produced by kind;
- events dropped by the no-interest fast path;
- events dispatched with zero matching handlers;
- ancestor nodes visited;
- native handlers invoked;
- MMS payload conversions and callback invocations;
- coalesced event count and retained count.

Use eye tracking, pointer movement, editor topology changes, and a quiet scene
as representative workloads. Record CPU time and allocation counts before and
after; do not assume queue traffic is a meaningful bottleneck without evidence.

## Workstream B: rename `Rx` to `Signal`

Follow the dedicated
[Rx-to-Signal naming migration](rx-to-signal-naming-migration.md). `Rx` is
historical and no longer communicates what these types own. The public/internal
vocabulary should consistently say `Signal`.

The audit includes at least:

- `RxWorld`;
- `RxIntentExecutor`;
- `RxMutationExecutor`;
- variables and fields named `rx` when they refer to the signal runtime;
- module names, imports, comments, tests, examples, and current documentation;
- old analysis documents that should retain historical names only when
  describing an earlier revision.

Likely names are `SignalWorld`, `SignalIntentExecutor`, and
`SignalMutationExecutor`, but reconsider whether `SignalWorld` is truthful.
The object is a queue, handler registry, timed-intent store, and pipeline index,
not an ECS world. `SignalRuntime` or `SignalBus` may be clearer. Settle that name
in the focused task before performing the mechanical rename.

Keep this rename separate from delivery or pipeline semantic changes. A
reviewable rename should preserve behavior and make stale terminology easy to
find with one repository-wide search.

## Workstream C: review intent pipeline operators

The current `SignalPipeline` is an **intent-recipient pipeline** despite its
general name. It runs before intent execution and currently contains one
operator:

```text
SignalRouteUpward(intent_kind, parent_type)
```

That operator rewrites the recipient of matching intents to an ancestor of a
specified component type. It was useful for transform/gizmo proxy routing and
does not process observed events.

The review should answer:

1. Which current components still rely on recipient rewriting?
2. Are those uses declarative scene behavior or workarounds for missing direct
   target references?
3. Does each operator have unambiguous ownership, cleanup, ordering, and cycle
   behavior?
4. Should the honest names be `IntentPipeline`, `IntentPipelineOperator`, and
   `IntentRouteUpward`?
5. Can operators remain recipient-local without scanning or cloning large
   signal payloads?
6. Which proposed transform operators still belong here after the transform,
   attachment, and velocity APIs mature?

Do not add event semantics to this pipeline merely because both lanes share a
`Signal` envelope.

## Workstream D: review event observers and `SignalObserverRouter`

`SignalObserverRouterComponent` is event-side machinery. During scoped event
dispatch it reads a whitelist/blacklist of **handler names** attached to the
current scope and filters which observers run. It does not rewrite intents and
is not currently part of `SignalPipelineProcessor`.

Review:

- whether handler-name filtering remains a useful authored capability;
- whether `SignalObserverRouter` should be renamed to
  `EventObserverRouter`, `EventObserverFilter`, or another event-specific name;
- the cost of cloning whitelist/blacklist vectors during every matching scoped
  dispatch;
- whether filters should index handler identities at registration time;
- how filters compose when multiple scopes in the bubble chain have routers;
- removal, serialization, and debugging visibility;
- whether consumption/stop-propagation belongs here or should remain a
  separate event-delivery feature.

There is an existing exploratory note about event filtering, transformation,
coalescing, scheduling, and event-to-intent bridging. Treat an eventual event
operator layer as a sibling to the intent pipeline unless a concrete use case
demonstrates a genuinely shared abstraction.

## Workstream E: clarify staging and drain ownership

Review `CommandQueue` alongside the naming work. It does not execute commands;
it stages both event and intent signals until they can be drained into the
signal runtime. `SignalBuffer` is a more accurate candidate name.

Preserve explicit drain points and their ordering guarantees. Moving or
removing the buffer must account for the Rust borrowing boundary between
`World`, `SystemWorld`, and signal emitters. Avoid replacing a confusing name
with hidden re-entrant processing.

Document and test:

- which events are ready now versus deferred to the next drain/frame;
- which intents emitted by event handlers execute in the same drain;
- timed-intent promotion;
- the signal processing cap and leftover behavior;
- callback servicing relative to engine intent execution.

## Proposed task order

1. [ ] Measure event production, zero-handler dispatch, ancestor traversal, and
   MMS callback conversion in representative workloads.
2. [ ] Add a zero-subscriber interest count/query plus a dispatch fast path,
   without changing observable behavior for registered handlers.
3. [ ] Coalesce only explicitly classified latest-state events, beginning with
   the HTC eye-tracking packet path if measurements justify it.
4. [ ] Audit current intent-pipeline operators and decide whether to rename the
   intent-only types.
5. [ ] Audit `SignalObserverRouter` and decide its event-specific name and
   future scope.
6. [ ] Perform the behavior-preserving repository-wide
   [Rx-to-Signal naming migration](rx-to-signal-naming-migration.md).
7. [ ] Review `CommandQueue` naming and drain ownership after the central signal
   type name is settled.
8. [ ] Design an event-operator/coalescing layer only from proven consumers,
   not as a prerequisite for the earlier performance work.

## Acceptance criteria for the epic

- A scene with no handlers for a high-frequency event kind does not construct
  or enqueue that optional observation event when the producer can query
  interest.
- Producers without interest-query access still reach an O(1) no-handler
  dispatch exit without an ancestor walk.
- Adding or removing engine or MMS handlers updates interest accurately;
  `Any`, scoped bubbling, handler cleanup, and session shutdown are covered.
- No intent is skipped because it lacks an observer.
- Lifecycle events retain ordering and multiplicity; coalescing occurs only for
  documented latest-state or dirty-notification events.
- MMS payload conversion and callback execution occur only for handlers that
  will receive the event.
- Intent-recipient operators and event-observer filters have distinct,
  descriptive names and documented execution phases.
- No active engine type, module, field, or ordinary variable retains the
  historical `Rx` name after the rename task.
- Before/after measurements demonstrate the effect of each performance change.

## Related documents

- [Event signal operators and coalescing](../../../analysis/event-signal-operators-and-coalescing.md)
- [CommandQueue audit](../../../analysis/command-queue-audit.md)
- [Intent versus mutation signal shape](../../../analysis/intent-vs-mutation-signal-shape.md)
- [Signal guide](../../../how_to/guide/signals.md)
- [Editor input routing](../../editor-input-routing.md)
- [Legacy transform pipeline and CommandQueue cleanup](../../legacy-transform-pipeline-and-command-queue-cleanup.md)
