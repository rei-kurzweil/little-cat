# Task: separate mounting from movement authority and input routing

Status: planned, 2026-09-12. Follows the first `Rider` + `Mountable` mount
slice and its `mittens-corp` vehicle demonstration.

## Problem

`AttachmentSystem` currently performs two unrelated jobs:

1. It validates and commits the Rider-to-Mountable relationship, keeps the
   transform relationship alive, and unwinds it safely.
2. It snapshots the Rider's pedestrian input setting, disables that automatic
   movement mapping on mount, and restores it on dismount.

The second job is movement authority and input routing, not attachment.  In
the current car example, raw controller events remain available and MMS keeps
its own `vehicle_state.mounted` flag from `MountStarted`/`MountEnded` to decide
whether it should drive the car.  That is a useful first slice, but it is not
a general control-context model.

Do not add desktop arrow-key vehicle controls as a workaround.  Vehicle
controls need a deliberate control-layer contract, not another global handler
whose behavior happens to be gated by an MMS boolean.

## Goal

Make attachment responsible only for relationship eligibility, atomic
mount/dismount, transform alignment/following, occupancy, and lifecycle
observations.  Move automatic-movement suppression and restoration into a
separate generic movement-authority/input-routing system.

The system must be generic: a car, broom, mech, or outer carrier may supply a
movement layer.  It must not be a car-specific rule embedded in
`AttachmentSystem`.

```text
interaction activation
        |
        v
AttachmentSystem
  validate + commit/unwind Rider <-> Mountable
  publish MountStarted / MountEnded
        |
        v
MovementAuthority / InputRoutingSystem
  maintain the active control-layer stack
  route input to its top layer
  suspend and restore lower automatic movement mappings
        |
        v
pedestrian / vehicle / carrier controller
  consumes its selected actions and owns its motion
```

## Intended contract

### AttachmentSystem

It must:

- resolve the rider, mountable, anchors, entry eligibility, and cycles;
- atomically establish or remove the transform-follow relationship;
- retain the authoritative active attachment edge for validation and cleanup;
- emit `MountStarted` only after commit and `MountEnded` after every normal or
  forced unwind.

It must not inspect, mutate, snapshot, or restore `Input`,
`InputXRGamepad`, or another control component.

### Movement authority and routing

Introduce an engine-owned, stack-based authority model.  A mounted controller
may push a layer above the rider's pedestrian layer; dismount removes exactly
that layer and restores the captured lower layer state.  Nested mounts must
work transactionally: rider -> mech -> carrier is three distinct layers, and
unwinding one must not blindly enable every lower mapping.

The layer must distinguish:

- raw device observation (keyboard/controller events remain available to the
  input backend);
- automatic transform locomotion, such as `I {}` WASD or
  `InputXRGamepad.locomotion()`;
- the active consumer of movement actions.

The routing owner must receive authoritative mount state directly from the
attachment relationship, rather than infer it from entry-zone membership,
disabled input, tree shape, or a scene-local boolean.  `MountStarted` and
`MountEnded` remain useful MMS observations, but event timing must not be the
only internal authority mechanism.

Vehicle motion itself stays out of this system.  A vehicle controller consumes
the routed action values and drives its own transform/velocity; a later physics
controller can make the same choice without changing mounting.

## Suggested implementation slices

1. Remove `snapshot_input`, `suspend_input`, and `restore_input` from
   `AttachmentSystem`, while preserving current mount/dismount safety tests.
2. Define a durable active-attachment query/notification boundary suitable for
   a sibling system; do not make a vehicle system reach into private attachment
   maps.
3. Add a generic movement-authority stack with exact prior-state restoration.
4. Migrate pedestrian `Input` and `InputXRGamepad.locomotion` to registered
   automatic movement layers.
5. Add an authored or engine-owned vehicle-controller layer that becomes active
   only for its matching `Mountable` edge.
6. Migrate `mittens-corp` from manual mounted-state gating to that layer, while
   retaining its existing XR stick/laser bindings as the first controller
   consumer.

## Acceptance criteria

- Mounting and dismounting succeed with no input components in the world.
- Attachment code has no dependency on desktop or XR input component types.
- Mounting a vehicle stops only the relevant pedestrian automatic movement;
  tracking and raw device observation remain live.
- A mounted vehicle alone receives its routed movement actions; an unmounted
  vehicle never receives them.
- Nested mounted contexts restore the immediately underlying movement layer,
  including when a vehicle/rider is removed unexpectedly.
- Focus loss, input-device loss, removal, failed commit, and scene teardown
  leave no stuck layer or stale held action.
- Existing `MountStarted` / `MountEnded` MMS behavior remains observable and
  occurs after the corresponding relationship transition.

## Related work

- [Rider + Mountable attachment-system first slice](rider-mountable-attachment-system-first-slice.md)
- [Interaction zones, sockets, and vehicle mounting](release-zones-sockets-and-vehicle-mounting.md)
- [Mittens-corp mounted vehicle controls and laser first slice](mittens-corp-mounted-vehicle-controls-and-laser-first-slice.md)
- [MMS keyboard and regular gamepad events](mms-keyboard-and-gamepad-events.md)
