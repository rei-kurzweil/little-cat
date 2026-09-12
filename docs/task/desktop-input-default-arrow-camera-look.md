# Task: default desktop Input arrow-key rotation

Status: implemented; manual desktop verification pending, 2026-09-12.

## Goal

An enabled desktop `Input` / `I {}` should rotate its ordinary controlled
Transform from held arrow keys by default:

- Left/Right Arrow apply yaw.
- Up/Down Arrow apply pitch.

When a `Camera3D` follows that controlled Transform, this provides keyboard
camera look.  This is still ordinary `Input` rotation, not a separate global
camera controller and not vehicle steering.  Arrow keys must not acquire a
vehicle meaning merely because a Rider is mounted.

The behavior exists only through enabled desktop `Input` components.  A scene
that deliberately omits `Input`, or an input disabled by MMS, the component
API, attachment handoff, or another runtime authority, receives no default
arrow-key rotation from that component.

## Current behavior

`InputComponent` supplies built-in WASD movement to its direct transform
child.  When rotation is enabled, right-mouse drag enters either the default
local/unrestricted rotation path or the FPS yaw/pitch path selected by
`InputTransformMode.fps_rotation()`.  Arrow keys do not currently participate.

This task must preserve ordinary WASD movement and right-mouse rotation while
feeding arrow-key deltas through those same two mode-specific paths.

## Required semantics

### Input activation

- Default arrow rotation is enabled only when `InputComponent.enabled` is
  true.
- If no enabled desktop input exists, arrow-key events have no engine-default
  Transform effect and remain available to authored/global handlers.
- Each enabled `Input` follows the same existing per-component rule as WASD:
  it applies input to its own direct controlled Transform.  Do not add a
  world-global active-camera lookup or a second camera-specific routing path.
- `InputTransformMode.rotation_disabled()` disables arrow rotation along with
  mouse rotation while leaving the component's translation behavior intact.
- Disabling, removing, or replacing an input immediately stops its
  default arrow control and clears held arrow state.

### Rotation modes and target ownership

Use the same controlled Transform already resolved by `InputSystem` for WASD
and right-mouse rotation.  A camera may be its descendant, including through
an avatar/head attachment, but `Input` must not search the world for a
`Camera3D` or directly mutate an unrelated camera.

Arrow input must honor the selected `InputTransformMode`:

- In the default/unrestricted mode, apply arrow yaw and pitch with the same
  local incremental rotation semantics used by the existing unrestricted
  mouse path.
- In FPS mode, add arrow yaw/pitch to the same retained FPS yaw/pitch state used
  by mouse look, including the existing pitch clamp and world-yaw/local-pitch
  reconstruction.
- Mouse and arrow deltas in one frame are additive; neither path may overwrite
  the other or maintain a second stale yaw cache.

Yaw and pitch ownership must remain compatible with:

- avatar/body yaw systems;
- `InputTransformMode.fps_rotation()` mouse look;
- camera/head attachments and avatar-control pose drivers;
- authored camera animations or constraints.

Use delta-time-held key state, not OS key-repeat frequency.  Clamp pitch to a
comfortable finite range; define the initial and reset behavior after focus
loss.  Preserve yaw wrapping without numerical drift.

### Focus and text entry

Arrow rotation must yield while a text field, editor control, or other focused
UI consumer captures those keys.  Window focus loss, scene reload, and input
disable/removal must clear held arrows so returning to the scene cannot leave
the camera rotating.

MMS `KeyDown`/`KeyUp` remains observable for authored behavior.  The default
arrow mapping must have explicit propagation/capture rules rather than
silently consuming all arrow events.

## Implementation outline

1. Read held `NamedKey::ArrowLeft/Right/Up/Down` state in `InputSystem` with
   focus-safe reset behavior.
2. Convert held arrows to delta-time-scaled yaw/pitch deltas.
3. Feed those deltas into both existing rotation implementations: default
   unrestricted rotation and retained-state FPS rotation.
4. Keep the existing `InputComponent.enabled` and
   `InputTransformMode.rotation_enabled` gates authoritative regardless of who
   changed them.
5. Integrate UI focus/capture routing and add mode-specific tests.
6. Exercise the behavior in `mittens-corp-desktop.mms`; it must gain ordinary
   arrow look through its `Input`, not arrow-key vehicle controls.

## Acceptance criteria

- In a minimal desktop scene with enabled `I {}`, held arrows yaw/pitch its
  controlled Transform smoothly at a delta-time-scaled rate.
- Default/unrestricted mode preserves its existing local incremental rotation
  semantics.
- FPS mode updates its one retained yaw/pitch state, respects the pitch clamp,
  and continues smoothly when alternating mouse and arrow input.
- The same scene with no `Input`, with that input disabled, or with
  `rotation_disabled()` does not rotate its controlled Transform from arrows.
- WASD movement and right-mouse FPS look retain their current behavior.
- Multiple enabled inputs affect only their own existing controlled transforms,
  consistently with current WASD semantics.
- An avatar-attached camera looks correctly without introducing an unrelated
  world-global camera writer or conflicting with AVC/head attachment.
- Text focus, window blur, scene reload, input removal, and input disable do
  not leave stuck rotation.
- MMS keyboard handlers retain documented observation/propagation behavior.

## Related work

- [Vehicle mounting disables desktop look together with locomotion](../bugs/vehicle-mount-disables-desktop-look-with-locomotion.md)
- [MMS keyboard events first slice](mms-keyboard-events-first-slice.md)
- [MMS keyboard and regular gamepad events](mms-keyboard-and-gamepad-events.md)
- [Input translation basis source for pose-driven locomotion](input-translation-basis-source-for-pose-driven-locomotion.md)
- [Avatar control desktop vs VR divergence](avatar-control-desktop-vs-vr-divergence.md)
- [Desktop mounted facing is reversed](../bugs/mittens-corp-desktop-mounted-facing-is-reversed.md)
