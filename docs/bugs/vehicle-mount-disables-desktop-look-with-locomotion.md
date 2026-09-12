# Vehicle mounting disables desktop look together with locomotion

Status: interim fix implemented on 2026-09-12; manual desktop validation pending.

## Problem

Mounting a vehicle previously disabled the Rider's entire desktop `Input`
component.  That prevented pedestrian WASD translation from moving the
player independently of the vehicle, but it also removed the player's ordinary
mouse and arrow-key rotation.  The mounted player consequently loses desktop
look even though look ownership has not transferred to the vehicle.

These are separate control capabilities:

- pedestrian translation belongs to the vehicle while mounted;
- player/head look remains on the Rider's desktop pose driver;
- authored vehicle controls receive their own keyboard/gamepad events and must
  not acquire player-look semantics as a side effect of mounting.

`InputComponent` is a pose driver: it can update its controlled Transform and
therefore dirty that Transform's descendant pose.  Its master enabled state is
too coarse to express a mount handoff that transfers only translation.

## Previous implementation evidence

`AttachmentSystem::snapshot_input` recorded `InputComponent.enabled` and
`AttachmentSystem::suspend_input` set the field to `false`.  On dismount,
`restore_input` restored the captured value.  `InputSystem` gated all built-in
desktop behavior on that single field, including:

- WASD/R/F translation;
- right-mouse rotation;
- arrow-key yaw and pitch;
- Q/E rotation on the configured roll axis.

The XR path already had the required conceptual split.  Mounting sets
`InputXRGamepadComponent.locomotion = false` while leaving the component and
raw XR input observation enabled.  Desktop input should have equivalent
capability-level ownership instead of disabling the whole pose driver.

This is observable in `examples/mittens-corp-desktop.mms`, whose Rider names
`bisket_desktop_input` as its pedestrian input.  The same `Input` owns both
WASD movement and FPS look on `bisket_desktop_driver`.

## Required component contract

Desktop `Input` needs independent translation and rotation gates in addition
to its master lifecycle gate:

```text
Input.enabled                 master pose-driver gate
Input.translation_enabled     built-in translation authority
Input.rotation_enabled        built-in rotation/look authority
```

All three default to `true`.  Effective authority is:

```text
translation active = enabled && translation_enabled
rotation active    = enabled && rotation_enabled
```

The component/live API should expose explicit operations such as:

```text
set_translation_enabled(bool)
set_rotation_enabled(bool)
```

Builder/getter spelling should follow the normal component API conventions.
Existing `enable()` / `disable()` remains the master pose-driver operation; it
must not be used as shorthand for a translation-only ownership transfer.

`InputTransformMode.rotation_enabled` and `rotation_disabled()` already model
part of the rotation gate.  For this bounded compatibility slice they remain an
additional authored restriction: effective rotation requires both the new
`Input.rotation_enabled` gate and the mode gate.  Consolidating that older mode
setting into one canonical representation is deferred to the subsequent input
and attachment rewrite.

## Implemented interim behavior

- `InputComponent` now has master, translation, and rotation gates, all defaulting
  to `true` and preserved by MMS serialization.
- `InputSystem` evaluates translation and rotation independently.
- Live MMS exposes `set_translation_enabled(bool)` and
  `set_rotation_enabled(bool)`.
- Desktop mount suspension snapshots and disables only
  `Input.translation_enabled`, then restores that captured value on unwind.
- `InputXRGamepad` retains its previous locomotion-only suspend/restore behavior.

This slice intentionally does not change mount-point yaw alignment or redesign
attachment authority. It exists so mounted desktop look can be exercised while
those later changes are investigated.

## Mount handoff semantics

On a successful desktop mount:

1. Snapshot the Rider input's translation-enabled state.
2. Set only translation authority to `false`.
3. Leave the master input and rotation authority unchanged.
4. Keep right-mouse and arrow look flowing through the same controlled
   Transform and FPS retained yaw/pitch state as before mounting.
5. Route vehicle locomotion through its own mounted-layer controls; do not map
   arrow look onto vehicle steering.

On dismount, cancellation, mount failure, target removal, or scene teardown,
restore the exact captured translation state.  Do not blindly enable
translation if it was already disabled by MMS, the component API, or another
runtime authority before mounting.

If another authority disables the master driver or rotation while mounted,
that remains authoritative.  Attachment owns only the translation lease it
acquired.  Nested or overlapping authority transfers must not restore stale
state over a newer owner; if those transfers are supported, use a scoped lease
or equivalent ownership token rather than stacked booleans.

## Input-state and pose continuity

- Disabling translation immediately stops WASD/R/F effects without stopping
  mouse, arrow, or Q/E rotation.
- Disabling rotation immediately stops all built-in rotation without stopping
  translation.
- Disabling the master pose driver stops both and clears its retained held-key
  activation state as defined by desktop input.
- Translation handoff must not discard or fork the FPS yaw/pitch cache.  Mouse
  and arrow look must remain continuous across mount and dismount.
- Restoring translation currently resumes from the ordinary physical key-down
  state. Requiring a fresh movement-key press remains follow-up authority/input
  routing work and should be made explicit before vehicle keyboard controls are
  finalized.
- UI keyboard capture remains authoritative for arrow look while mounted, just
  as it is while walking.

This capability split does not by itself solve mount-point yaw alignment.  A
mount transition may still need to synchronize the retained FPS orientation
with an intentional authored yaw snap; that is tracked separately.

## Implementation checklist

1. [x] Add translation/rotation capability state to desktop `Input` and
   expose builder, serialization, registry, and live component methods.
2. [x] Split `InputSystem`'s master early gate into independent translation and
   rotation gates while continuing to resolve one direct controlled Transform.
3. [x] Change `AttachmentSystem::SuspendedInput::Desktop` to snapshot and suspend
   translation only.  Preserve exact restore behavior on every exit path.
4. [x] Keep the component registered and its FPS/look state live throughout the
   mounted interval.
5. [x] Update API documentation and tests that currently describe desktop
   `disable()` as relinquishing automatic locomotion authority; it disables the
   entire desktop pose driver, while translation-specific relinquishment uses
   the new capability API.

## Acceptance criteria

- While walking, enabled desktop `Input` retains existing WASD, mouse, arrow,
  and Q/E behavior.
- While mounted, WASD/R/F cannot translate the Rider's controlled Transform,
  but mouse and arrow look continue smoothly on that Transform.
- Mounted arrow keys remain player look and do not become implicit vehicle
  steering controls.
- A deliberately disabled master input remains fully disabled while mounted.
- A deliberately disabled rotation capability remains disabled while mounted,
  while a deliberately disabled translation capability is restored as disabled
  after dismount.
- Dismount restores the exact pre-mount translation authority. Fresh-press
  behavior for a movement key held across dismount remains deferred as noted
  above.
- Mouse and arrow look share the existing FPS state before, during, and after
  the handoff, including pitch clamp and yaw wrapping.
- Multiple Inputs retain their current per-component target ownership; mounting
  one Rider does not alter unrelated Inputs.
- Text fields and focused controls continue to capture arrow keys, and MMS
  keyboard observation retains its documented propagation behavior.
- XR behavior remains unchanged: tracked pose and raw gamepad observation stay
  enabled while only XR locomotion is suspended.

## Related work

- [Desktop occupancy mount pose and input-authority first slice](../task/desktop-occupancy-mount-pose-and-input-authority-first-slice.md)
- [Default desktop Input arrow-key camera look](../task/desktop-input-default-arrow-camera-look.md)
- [`mittens-corp-desktop` mounted facing is reversed](mittens-corp-desktop-mounted-facing-is-reversed.md)
- [Rider + Mountable attachment-system first slice](../task/rider-mountable-attachment-system-first-slice.md)
- [`mittens-corp` mounted vehicle controls and laser first slice](../task/mittens-corp-mounted-vehicle-controls-and-laser-first-slice.md)
- [Input translation basis source for pose-driven locomotion](../task/input-translation-basis-source-for-pose-driven-locomotion.md)
