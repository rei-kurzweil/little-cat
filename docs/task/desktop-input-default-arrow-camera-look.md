# Task: default desktop Input arrow-key camera look

Status: planned, 2026-09-12.

## Goal

An enabled desktop `Input` / `I {}` should give a desktop scene a usable
keyboard look control by default:

- Left/Right Arrow yaw the active desktop camera.
- Up/Down Arrow pitch the active desktop camera.

This is camera look, not vehicle steering.  Arrow keys must not acquire a
vehicle meaning merely because a Rider is mounted.

The behavior exists only while there is an enabled desktop `Input` in the
world.  A scene that deliberately omits `Input`, or disables its sole active
desktop input, receives no default arrow-key camera movement.

## Current behavior

`InputComponent` supplies built-in WASD movement to its direct transform
child.  Its camera rotation path is right-mouse drag/FPS rotation; the arrow
keys do not currently participate.  The component has only `speed` and
`enabled` configuration, so there is no camera/action binding contract today.

This task must preserve ordinary WASD movement and existing right-mouse look.

## Required semantics

### Input activation

- Default arrow look is enabled only when desktop `InputComponent.enabled` is
  true.
- If no enabled desktop input exists, arrow-key events have no engine-default
  camera effect and remain available to authored/global handlers.
- The implementation must define and test the multiple-enabled-input case;
  do not let every enabled input rotate the same camera.  Prefer an explicit
  active-input selection or require a single active desktop input before
  enabling the default.
- Disabling, removing, or replacing the active input immediately stops its
  default arrow control and clears held arrow state.

### Camera target and pose ownership

Define an explicit target-resolution rule.  The input must rotate the relevant
`Camera3D` pose, not accidentally pitch an avatar locomotion root or a
vehicle.  It must work for the common desktop layouts where the camera is a
descendant of an avatar/head attachment as well as a standalone `Camera3D`.

Yaw and pitch need separate, well-documented ownership so they do not fight:

- avatar/body yaw systems;
- `InputTransformMode.fps_rotation()` mouse look;
- camera/head attachments and avatar-control pose drivers;
- authored camera animations or constraints.

Use delta-time-held key state, not OS key-repeat frequency.  Clamp pitch to a
comfortable finite range; define the initial and reset behavior after focus
loss.  Preserve yaw wrapping without numerical drift.

### Focus and text entry

Arrow look must yield while a text field, editor control, or other focused UI
consumer captures those keys.  Window focus loss, scene reload, and input
disable/removal must clear held arrows so returning to the scene cannot leave
the camera rotating.

MMS `KeyDown`/`KeyUp` remains observable for authored behavior.  The default
camera mapping must have explicit propagation/capture rules rather than
silently consuming all arrow events.

## Implementation outline

1. Record desktop arrow held state at the input boundary with focus-safe reset.
2. Define the active enabled `Input` and its resolved `Camera3D` target.
3. Add a camera-look driver/authority that applies yaw and pitch once per frame
   after resolving conflicts with avatar and camera pose systems.
4. Integrate UI focus/capture routing.
5. Document the MMS topology needed for standalone and avatar-attached desktop
   cameras.
6. Update `mittens-corp-desktop.mms` only after the engine contract is ready;
   it must gain default arrow camera look, not arrow-key vehicle controls.

## Acceptance criteria

- In a minimal desktop scene with one enabled `I {}` and `Camera3D`, held
  arrows yaw/pitch the camera smoothly at a delta-time-scaled rate.
- The same scene with no `Input`, or with that input disabled, does not move
  the camera from arrows.
- WASD movement and right-mouse FPS look retain their current behavior.
- An avatar-attached camera looks correctly without pitch/roll being written
  into the locomotion root or conflicting with AVC/head attachment.
- Text focus, window blur, scene reload, input removal, and input disable do
  not leave stuck rotation.
- MMS keyboard handlers retain documented observation/propagation behavior.

## Related work

- [MMS keyboard events first slice](mms-keyboard-events-first-slice.md)
- [MMS keyboard and regular gamepad events](mms-keyboard-and-gamepad-events.md)
- [Input translation basis source for pose-driven locomotion](input-translation-basis-source-for-pose-driven-locomotion.md)
- [Avatar control desktop vs VR divergence](avatar-control-desktop-vs-vr-divergence.md)
