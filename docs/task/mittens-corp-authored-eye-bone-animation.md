# Task: authored ambient eye-bone animation for `mittens-corp`

Status: proposed / depends on pupil-direction consumption toggle and imported-bone animation access

## Goal

Create `examples/mittens-corp-eye-animations.mms`, based on the corporation
scene, where live pupil-direction tracking is disabled but live eye-open amount
tracking remains enabled. Drive the avatar's mapped left and right eye bones
with a looping, gently tweened authored animation instead.

The loop should contain at least 16 gaze targets; 32 is preferred. Successive
targets should be spaced between 500 ms and 2000 ms apart and should look like
small ambient saccades rather than a regular mechanical sweep.

## Intended scene configuration

```text
XREyeTracking.on()
    .enable_pupil_direction_tracking(false)
```

The tracker remains a direct child of `AVC`, so its closure samples continue to
drive `left_eye_blink` and `right_eye_blink`. Only AVC's live gaze-to-eye-bone
rotation path is disabled. A looping `Animation` owns eye direction for this
example.

## Why this cannot be only an example-file change today

MMS `Animation` keyframes dispatch actions at their timestamps; they do not
interpolate transform values by themselves. Smooth motion currently comes from
a `Transition` attached to the target `Transform`.

The eye bones are transforms created asynchronously by GLTF import and selected
through a `HumanoidBoneMap`. The example does not currently have stable MMS
component references to those resolved semantic targets at declaration time,
nor a supported way to attach authoring-only `Transition` state to them after
import. AVC also owns restoration of tracked eye bones. Directly querying node
names would couple the example to one avatar and risks racing import and
fighting AVC.

## Required engine/API slice

Provide an asynchronous, GLTF-instance-scoped way for MMS to obtain the
resolved `left_eye` and `right_eye` transform components after the humanoid map
is ready. Prefer semantic slots over raw node-name queries. Acceptable shapes
include a readiness callback/query API or a small component that binds an
animation target to a humanoid slot.

The resulting contract must:

- resolve within the intended avatar instance, never another avatar;
- return the actual mutable transform used by skinning;
- expose or preserve each bone's immutable rest translation, rotation, and
  scale so authored gaze offsets are rest-relative rather than absolute;
- permit transitions/tweening on those transforms;
- release and re-resolve safely across GLTF reload or map regeneration; and
- define ownership priority between AVC, animation, IK, and imported animation.

For this example, when pupil-direction tracking is false, the authored eye
animation owns eye-bone rotation while AVC retains ownership only of blink
morphs. Removing the authored animation must restore the bone's prior/rest pose,
not leave its last keyframe applied.

## Tweening design

Attach a `Transition` to each resolved eye transform, or provide equivalent
animation-native interpolation, with these semantics:

- quaternion-safe shortest-path rotation interpolation; do not linearly blend
  Euler angles across wrap boundaries;
- duration shorter than the current keyframe interval, roughly 120–350 ms, so
  each saccade moves quickly and then holds;
- `replace_same_target()` behavior so a new gaze target cleanly supersedes an
  unfinished tween;
- rest-relative offsets with conservative, independently clamped yaw/pitch;
- both eyes normally converge on the same distant target, with only subtle
  per-eye variation if anatomically safe; and
- seamless final-to-first loop interpolation.

If the existing `Transition` system cannot target imported transforms or cannot
interpolate quaternions correctly, extend it or introduce a focused bone-pose
transition mechanism before authoring the example.

## Keyframe schedule

Author 32 deterministic samples so runs and tests are reproducible. The
intervals should vary within `[0.5s, 2.0s]`; “random” here means an irregular,
pre-authored seeded sequence, not runtime nondeterminism.

Each keyframe supplies a small rest-relative yaw/pitch target. Bias most samples
toward center, occasionally look farther left/right/up/down, avoid alternating
extremes, and include repeated or near-repeated targets to create natural
holds. The loop length is the sum of the intervals, and the last pose must
transition naturally back to the first.

Prefer a helper/factory for the repeated two-eye pose action if MMS can preserve
component-reference provenance through keyframe closures. Otherwise keep the
literal keyframes in the example and cover them with an evaluation test.

## Implementation steps

1. Complete `xr-eye-tracking-pupil-direction-toggle.md`.
2. Add the semantic, asynchronous eye-bone target binding described above.
3. Confirm `Transition` can tween rotations on imported/skinned bone transforms
   and add quaternion-safe interpolation if missing.
4. Copy the relevant `mittens-corp` scene setup into
   `examples/mittens-corp-eye-animations.mms`; keep it a focused acceptance
   scene rather than changing the main example immediately.
5. Configure pupil-direction tracking false while leaving tracker/closure input
   active.
6. Add 32 deterministic keyframes with 0.5–2.0 second intervals and conservative
   eye rotations.
7. Add cleanup/reload behavior and automated structural tests.
8. Validate visually on the target avatar with live blinking input.

## Tests and acceptance criteria

- The example parses, evaluates, and resolves exactly one left/right eye pair
  within its avatar.
- It contains at least 16 keyframes (target: 32), and every adjacent interval,
  including loop wrap, satisfies the intended timing contract or documents the
  special wrap duration.
- Eye rotations visibly tween rather than snap and take the shortest quaternion
  path.
- Eye direction continues animating when no gaze packets arrive.
- Live gaze packets do not alter eye-bone direction in this example.
- Live openness/closure packets still animate the mapped blink morphs.
- Stopping/removing the animation or reloading the avatar restores a valid rest
  pose and leaves no stale component references.
- Motion stays within the configured eye rotation limits and does not diverge
  between eyes unnaturally.

## Related work

- `docs/task/xr-eye-tracking-pupil-direction-toggle.md`
- `docs/task/xr-eye-tracking-avc.md`
- `docs/task/unified-two-eye-tracking-normalization-and-avc-routing.md`
- `examples/mittens-corp.mms`
- `examples/transition.mms`
- `src/engine/ecs/system/avatar_control_system.rs`
- `src/engine/ecs/component/transition.rs`
- `src/engine/ecs/system/animation_system.rs`

