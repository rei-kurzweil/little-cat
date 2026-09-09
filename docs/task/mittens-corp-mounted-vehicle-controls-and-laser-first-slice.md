# Task: `mittens-corp` mounted vehicle controls and laser first slice

## Status and outcome

Implemented for XR validation on 2026-09-09. The runtime now publishes mounted
lifecycle observations, and the example gates planar driving and the visual
laser on its active car mount. Direction, speed, steering sign, muzzle
placement, beam orientation, and timing still need in-headset tuning.

This starts with a planar transform-driven car. It does not pretend to be the
future velocity/force or pluggable-physics implementation.

## Intended interaction

While Bisket is mounted in `left_display_car`:

- the left stick moves the car on its local ground-plane heading;
- pedestrian `InputXRGamepad` locomotion remains disabled, but the same
  component continues emitting canonical XR events;
- holding right grip and pressing right trigger calls `fire_laser()` once per
  trigger-down edge;
- dismount restores pedestrian locomotion and deactivates vehicle control.

The temporary “any grip ejects” behavior conflicts with the firing chord.
Until the guarded eject control exists, reserve left-grip press for temporary
dismount and leave right grip available as the weapon modifier.

## What MMS supports now

### XR controls

`InputXRGamepad` emits `XrAxisChanged`, `XrButtonDown`, `XrButtonUp`, and
`XrButtonChanged`. Payloads expose `hand`, `control`, and `value`;
`LeftStick`, `RightGrip`, and `RightTrigger` are canonical names. `disable()`
relinquishes only built-in locomotion and intentionally keeps events live.

MMS table literals are heap-backed, so callbacks can share mutable script
state through fields on one retained table. Captured primitive bindings are
snapshots and must not be reassigned as if they were shared. The axis handler
stores the latest stick value on `vehicle_state`, while
`on_global("FrameTick", ...)` reads that state and its `dt_sec` to integrate
continuous movement without relying on input repeat frequency.

### Ground-plane transform motion

MMS supports scalar arithmetic, array indexing, `Math.sin/cos/sqrt/clamp`,
`Transform.translation()`, and `Transform.update_transform(position, euler,
scale)`. It does not support overloaded element-wise array operators; construct
result arrays explicitly.

`Transform.trs()` and `Transform.world.trs()` return an intentionally opaque
snapshot. MMS cannot inspect quaternion channels, multiply a matrix by a
direction, or ask a transform for local forward today. A reusable vehicle or
flying implementation should eventually gain a high-level API such as:

```mms parse-only
let forward = vehicle.world.direction([0, 0, -1])
```

Prefer that to exposing raw 4x4 matrices as the ordinary movement API.

The first car need not wait for it. Its initial yaw is authored here and the
controller is its sole rotation writer, so MMS can retain `vehicle_state.yaw`
and compute planar forward explicitly. Keep `vehicle_state.position[1]`
unchanged.
Ramps, gravity, contact, velocity inheritance, and forces are follow-ups.

### One-shot visual animation

`Animation.playing()` is non-looping and becomes paused after its length.
Calling `animation.play()` resets its local clock and fired keyframes, so it is
replayable.

Build a hidden-at-rest muzzle flash from a zero-scale sphere with
`Emissive.off()`. Build `laser_beam_glow` from three nested thin quads (or
crossed ribbons if a flat beam disappears at oblique angles), with a narrow,
brighter core and wider, more translucent outer layers. A short animation
expands and illuminates them at beat zero, then collapses them and turns
emissive off. This is a visual beam, not yet a raycast, projectile, or damage
event; `laser_beam_glow` is clearer than `laser_trajectory_glow`.

## Mounted-state seam

`AttachmentSystem` owns the active Rider-to-Mountable edge and now publishes
`MountStarted` and `MountEnded` observations, giving MMS an authoritative way
to gate this car's handlers.

Both events carry stable rider and mountable identities. The system emits
`MountStarted` only after the transaction commits and `MountEnded` after
ordinary dismount or cleanup/unwind. The example retains mounted state from
these events in its shared `vehicle_state` table and checks it in movement and
firing handlers.

Do not infer mounting from zone membership, disabled input, proximity, or
transform topology in MMS. Those are consequences or eligibility, not the
authoritative relationship.

## First implementation sequence

1. [x] Verify canonical XR events remain live after
   `InputXRGamepad.disable()`.
2. [x] Verify persistent handler state, `FrameTick.dt_sec`, scalar math, array
   indexing, transform mutation, and replayable one-shot animation.
3. [x] Publish and expose `MountStarted` / `MountEnded`, with focused commit,
   dismount, and cleanup tests.
4. [x] Reserve left grip for temporary mounted dismount so right grip can be a
   weapon modifier.
5. [x] Retain left-stick state and integrate planar car-local motion on
   `FrameTick`, with deadzone and speed constants.
6. [x] Author the hidden muzzle flash and layered `laser_beam_glow` under a
   vehicle muzzle transform.
7. [x] Define `fire_laser()` and a paused one-shot animation; trigger it only
   on right-trigger down while right grip is held and this car is mounted.
8. [ ] Validate mount, drive, stop, fire, dismount, and remount in XR.
9. [ ] Add a general transform-direction live API before flight or reusable
   arbitrary-basis controllers.

## Acceptance criteria

- The car never moves from these controls while unmounted.
- Left-stick motion follows car yaw and never changes Y in provisional ground
  mode.
- Pedestrian locomotion does not also move Bisket while mounted.
- Right-trigger down fires only while right grip is held and the car is mounted.
- Flash and beam return fully to rest and replay on the next valid press.
- Right grip does not eject; temporary left-grip escape remains available.
- Dismount restores the exact captured pedestrian locomotion state.

## Related work

- [Rider + Mountable attachment-system first slice](rider-mountable-attachment-system-first-slice.md)
- [Interaction zones, sockets, and vehicle mounting](release-zones-sockets-and-vehicle-mounting.md)
- [Velocity, forces, and pluggable physics](velocity-forces-and-pluggable-physics.md)
- [Spatial, collision, and physics naming](spatial-collision-and-physics-naming.md)
