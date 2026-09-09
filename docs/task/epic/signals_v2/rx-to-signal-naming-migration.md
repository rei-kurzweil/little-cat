# Task: rename `Rx` signal machinery to `Signal`

## Status

Planned under the [Signals v2 epic](README.md), 2026-09-09.

## Outcome

Replace the historical `Rx` vocabulary throughout active engine code with
explicit `Signal` vocabulary without changing queueing, routing, handler,
pipeline, timing, or execution behavior.

Expected primary mappings:

| Current | Target |
| --- | --- |
| `RxWorld` | `SignalWorld` |
| `RxIntentExecutor` | `SignalIntentExecutor` |
| `RxMutationExecutor` | `SignalMutationExecutor` |
| `rx_world.rs` | `signal_world.rs` |
| local/field name `rx` | `signals` or `signal_world`, according to what it stores |

Use `SignalWorld` for the first mechanical pass so the rename has a definite
target. If a separate Signals v2 design task concludes that `SignalRuntime` or
`SignalBus` is materially more accurate, make that a later explicit rename;
do not combine terminology debate with unrelated behavior changes.

## Scope

- Type declarations, constructors, imports, re-exports, fields, parameters,
  locals, module paths, debug labels, tests, and comments in active Rust code.
- Current how-to, spec, task, and architecture documentation where `Rx` names
  describe the present implementation.
- MMS host/runtime-boundary descriptions and examples.
- Any scripts or development commands that name the Rust types or modules.

Historical analysis may retain an old name when describing a past revision,
but should explicitly identify it as historical. Do not mechanically alter
external quotations or unrelated uses of “rx” that genuinely mean receive-side
data rather than this signal subsystem.

## Boundaries

This task must not:

- add handler-interest counting;
- change event bubbling or delivery order;
- change ready/deferred/timed queue semantics;
- change intent interpretation or mutation behavior;
- rename intent-only pipeline types before their architecture review;
- remove or inline `CommandQueue`;
- add event-pipeline operators.

Those changes belong to separate Signals v2 workstreams. Keeping this rename
mechanical makes compiler errors and review evidence trustworthy.

## Implementation sequence

1. [ ] Record a repository-wide `Rx` inventory and classify false positives.
2. [ ] Rename the three core types and the `rx_world` module/file.
3. [ ] Rename ownership fields and parameters from `rx` to `signals` or
   `signal_world`.
4. [ ] Update imports, public re-exports, tests, debug output, and code comments.
5. [ ] Update current documentation and links while preserving intentional
   historical references.
6. [ ] Run formatting, compilation, focused signal tests, scripting callback
   tests, and the available library suite.
7. [ ] Repeat the inventory and account for every remaining `Rx`/`rx` match.

## Acceptance criteria

- Active code contains no `RxWorld`, `RxIntentExecutor`, or
  `RxMutationExecutor` identifier.
- The corresponding module and ordinary ownership variables use `Signal`
  terminology.
- Remaining lowercase `rx` matches are reviewed and are either renamed or
  documented as genuine receive-side terminology.
- Signal queue ordering, event bubbling, handler cleanup, timed intents,
  pipeline routing, and MMS callback delivery retain their existing focused
  test behavior.
- The change contains no intentional semantic modification.
- `cargo fmt --all`, `cargo check`, and focused signal/scripting tests pass.
