# `mittens-corp-desktop` startup appears slower than the XR example

Status: open, reported by user on 2026-09-12; timing data not yet captured.

## Observed behavior

`examples/mittens-corp-desktop.mms` takes a noticeably long time to become
usable.  On the same development machine it appears to start more slowly than
`examples/mittens-corp.mms`, even though the latter initializes the XR version
of the scene.

This is currently a subjective comparison.  We have not yet established:

- wall-clock time for either example;
- whether the delay occurs before the first window/frame or during the first
  few frames;
- whether the runs used equivalent build, asset-cache, shader-cache, editor,
  and runtime conditions;
- whether the apparent delay is CPU work, GPU/shader preparation, synchronous
  asset loading, cascading ECS/Rx work, or a wait elsewhere in startup.

## Expected behavior

The desktop example should reach its first visible, responsive frame without a
large unexplained delay.  Under controlled same-machine conditions, any
material difference from the XR example should be measured and attributable to
intentional work in one path rather than an accidental desktop-only startup
regression.

## Measurement boundary

Treat startup as the interval from launching an already-built executable and
requesting the MMS scene through the first presented frame that accepts input.
Record compilation time separately; a Cargo rebuild is not scene startup.

Measure at least these milestones:

1. process/runtime initialization begins;
2. MMS parse/evaluation begins and ends;
3. editor and shared asset-module setup begins and ends;
4. glTF requests are issued and all required scene instances finish spawning;
5. the first `SystemWorld::tick` begins and ends, including queue/Rx drain
   loops;
6. camera/render preparation completes;
7. first frame is submitted/presented;
8. the next few frames complete, to distinguish a single slow first frame from
   startup work spread across several frames.

Capture peak resident memory and note whether the window is absent, visible but
unresponsive, or rendering while the delay is perceived.

## Controlled comparison

Compare `mittens-corp-desktop.mms` and `mittens-corp.mms` using:

- the same commit and release/debug profile;
- an already-built binary for both runs;
- both cold-cache and warm-cache runs, labeled separately;
- the same editor visibility/configuration where applicable;
- several repetitions rather than a single run;
- XR runtime/headset state recorded explicitly, since the XR path may skip,
  block, or defer work depending on session availability.

Report median and individual startup times.  Do not conclude that a subsystem
is responsible solely from the total process time.

## Investigation plan

1. Reproduce and capture a baseline video or timestamps for both examples.
2. Use the planned `SystemWorld::tick` startup timing breakdown for the first
   frame and, separately, the first small frame window.
3. Add coarse timestamps outside `SystemWorld::tick` for MMS evaluation, asset
   import, renderer readiness, and first presentation so pre-tick work is not
   invisible.
4. Compare the evaluated scene/component counts and glTF import/cache events to
   find desktop-only repeated work or different initialization ordering.
5. Narrow the dominant phase before adding finer instrumentation or changing
   behavior.

## Acceptance criteria

- The delay is reproducible with an already-built binary, or the report is
  closed as compilation/cache noise with measurements showing that result.
- Desktop and XR startup measurements use documented equivalent conditions.
- The dominant startup phase is identified with timing evidence.
- Any fix includes before/after measurements from multiple runs.
- `mittens-corp-desktop.mms` reaches a visible, responsive first frame without
  the reported unexplained delay, and XR startup does not regress.

## Related work

- [SystemWorld tick timing breakdown and terminal graph](../task/startup-timing-breakdown-terminal-graph.md)
- [Armature visualization startup follow-up](../task/armature-visualization-startup-followup.md)
- [Desktop occupancy mount pose and input-authority first slice](../task/desktop-occupancy-mount-pose-and-input-authority-first-slice.md)
