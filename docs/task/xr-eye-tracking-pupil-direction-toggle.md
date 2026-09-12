# Task: independently disable pupil-direction tracking

Status: proposed

## Goal

Add a builder setting to `XREyeTracking` and `XREyeTrackingHTC` that controls
whether live gaze/pupil direction drives avatar eye bones:

```text
XREyeTracking.on().enable_pupil_direction_tracking(false)
XREyeTrackingHTC.on().enable_pupil_direction_tracking(false)
```

The setting defaults to `true` to preserve existing scene behavior. Disabling direction tracking must not disable
the transport, event emission, retained samples, or eye-open amount tracking.
In particular, mapped left/right blink morphs must continue to follow retained
closure/openness samples.

Although the requested public wording says “pupil direction,” the existing AVC
path actually rotates mapped `left_eye` and `right_eye` bones from the retained
gaze direction. Document that meaning explicitly so this flag is not confused
with HTC's separately reported 2D pupil position or pupil diameter.

## Current behavior and ownership

`XREyeTrackingComponent` and the HTC component retain `gaze_sample` and
`closure_sample` independently. `AvatarControlSystem` independently calls the
gaze/eye-bone and blink/morph paths. This makes AVC consumption the appropriate
gate: source polling and diagnostic events should continue even when gaze pose
application is disabled.

`XREyeTrackingHTCComponent` is currently a Rust type alias for
`HTCEyeTrackingComponent`; the MMS names `XREyeTrackingHTC` and `HTCEyeTracking`
therefore need consistent serialization and builder behavior.

## Required changes

1. Add `enable_pupil_direction_tracking: bool` to
   `XREyeTrackingComponent` and `HTCEyeTrackingComponent`, initialized to
   `true` by `on()`, `listen(...)`, and `Default`.
2. Add Rust builders and MMS method dispatch for
   `.enable_pupil_direction_tracking(bool)` on `XREyeTracking` and
   `XREyeTrackingHTC`. Apply the same behavior to compatibility names that map
   to those concrete types.
3. Include the non-default value in MMS serialization/round-trip output. Follow
   the repository convention for whether an explicit `false` is omitted.
4. In AVC gaze resolution/application, ignore a tracker's gaze sample when its
   flag is false. Do not ignore its closure sample.
5. When the flag changes from true to false or a disabled tracker replaces an
   enabled one, release any AVC-owned eye-bone rotation and restore the immutable
   GLTF rest rotation exactly once. Do not leave the last tracked gaze frozen.
6. Keep `XrEyeTrackingUpdated` and `XrEyeTrackingHtcUpdated` payloads unchanged;
   this flag governs avatar pose consumption, not observation or transport.
7. Update component documentation and eye-tracking examples whose behavior
   depends on live gaze so they opt in explicitly.

## Tests

- Constructors and deserialization default the flag to true.
- MMS round-trip preserves the non-default `false` for both component spellings.
- With the flag false and both gaze and closure samples present, eye bones stay
  at rest while blink morph drivers update.
- With the flag true, current per-eye gaze rotation behavior remains intact.
- Switching true to false restores both eye bones without clearing closure.
- A disabled newer gaze source cannot suppress or take ownership from an
  enabled source unless the existing source-election contract explicitly says
  that the selector-level flag owns the final gate.
- Raw update events still fire in both modes.

## Acceptance criteria

- Pupil/eye-bone direction tracking remains enabled by default and can be disabled per tracker.
- Eye-open amount tracking remains active when direction tracking is off.
- Disabling direction never leaves stale eye-bone rotation behind.
- Generic and HTC tracker APIs behave consistently and round-trip through MMS.

## Related files

- `src/engine/ecs/component/xr_eye_tracking.rs`
- `src/engine/ecs/system/avatar_control_system.rs`
- `src/engine/ecs/system/xr_eye_tracking_system.rs`
- `src/scripting/component_registry.rs`
- `src/scripting/runtime_config.rs`
- `src/scripting/tests.rs`
- `docs/how_to/guide/components.md`
