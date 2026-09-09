# Task: Rider + Mountable attachment-system first slice

## Status and outcome

Planned, 2026-09-08. Implement the first native mounting path in
`mittens-engine` using authored `Rider` and `Mountable` components and a runtime
`AttachmentSystem` relationship. Keep the existing `Grabbable` component and
`GrabbableSystem` operational; eligible mounting receives grip priority and an
ineligible mount attempt falls back to ordinary grabbing when the pointed owner
is also grabbable.

Validate the slice in [mittens-corp](../../examples/mittens-corp.mms): while
the Bisket rider is inside the independent mech/car's front entry zone,
pressing the XR grip while pointing at the car mounts Bisket at the authored
cockpit target. Outside that zone, gripping the car does not mount. The mount
transaction suppresses Bisket's automatic pedestrian locomotion without
disabling `InputXR` tracking or raw XR controller events.

This task is the narrow bridge between the implemented
[zone query foundation](interaction-zone-collision-query-foundation.md) and the
broader [zones, sockets, and vehicle mounting design](release-zones-sockets-and-vehicle-mounting.md).

## Why three authored concepts remain distinct

The public vocabulary has three useful concepts even though only two enter the
new attachment runtime in this slice:

- `Rider` describes the participant that can be mounted: its movement root,
  alignment anchor, and automatic movement mapping.
- `Mountable` describes a destination: its entry zone, destination anchor,
  activation policy, occupancy, and later its vehicle movement layer.
- `Grabbable` describes an object that can be held by a pointer.

Do not replace or reinterpret `Grabbable` while proving the first mount path.
The existing grab system already owns hand-relative placement, clearance,
release, and parent restoration. A held object is conceptually another
attachment relationship, but migrating it immediately would combine the new
mount transaction with a rewrite of established grabbing behavior.

The first implementation therefore has one interaction arbiter feeding two
consumers:

```text
grip activation + pointed hit
             |
             v
      interaction arbiter
        /             \
eligible Mountable   eligible Grabbable
        |                  |
        v                  v
 AttachmentSystem    GrabbableSystem
```

Exactly one consumer wins a grip activation. The arbiter must not emit two
independent commands and rely on system tick order to resolve the conflict.

## Proposed MMS contract

The exact builder spellings can change during registration, but the first
contract should stay close to:

```mms
// Authored within the pointer-associated Bisket/player tree.
Rider
    .anchor("[name='bisket_rider_cxr_anchor']")
    .movement_root("[name='bisket_locomotion_root']")
    .input("[name='bisket_pedestrian_locomotion']") {}

// Authored on the independent mech/car owner.
Mountable
    .entry_zone("[name='left_display_car_front_zone']")
    .mount_anchor("[name='left_display_car_cxr_mount']")
    .on_grip() {}
```

Every component-reference field accepts either a live MMS component object or
a durable string query, following the existing `ComponentRef` contract. A live
object becomes a GUID reference. Vehicle-side unprefixed queries resolve in the
`Mountable` owner's containing scope. Rider-side queries resolve within the
pointer-associated rider tree; they must not accidentally select another
avatar from a world-global search.

`Rider` is explicit rather than inferred from `AvatarControl`, `CXR`, humanoid
bones, or a particular input topology. This permits camera-only riders,
non-humanoid occupants, nested vehicles, and test fixtures without embedding
Bisket-specific conventions in the attachment system.

An object may carry both roles:

```mms
T {
    Rider { /* this mech can itself enter an outer carrier */ }
    Mountable { /* an inner rider can enter this mech */ }
}
```

The broom can independently carry both `Grabbable` and `Mountable`; its
release-to-mount activation policy is a follow-up after grip-to-enter is proven.

## Grip activation and eligibility

The XR grip and desktop grab gesture represent an interaction attempt, not a
precommitted grab. For the car path, mounting wins only when all of these are
true at activation time:

1. the pointed raycast hit resolves to an enabled, unoccupied `Mountable` owner;
2. the initiating pointer resolves to exactly one enabled `Rider` association;
3. the rider's configured probe is inside or on the boundary of the referenced
   entry zone;
4. the rider movement root, rider anchor, zone, and destination anchor are live;
5. the proposed relationship introduces no structural or effective-transform
   cycle;
6. the required transform bases are finite and non-singular.

For the first car slice, the rider anchor's current world origin is also the
entry probe. This corresponds to the tracked HMD/CXR position and prevents a
long controller ray from mounting the player from outside the vehicle's entry
area. Add a separate `probe(...)` field later only when a rider needs an entry
point different from its alignment anchor.

If the eligible mount check fails and the same pointed owner is enabled
`Grabbable`, dispatch the ordinary grab path. If it is only `Mountable`, do
nothing. Merely entering the zone never mounts, and merely pointing at or
gripping the car from outside the zone never mounts.

Eligibility is a synchronous current-transform query. Cached zone enter/exit
state may eventually drive preview feedback but cannot authorize the commit.

## AttachmentSystem runtime relationship

Authored components describe capabilities and rules. Mutable mounted state
belongs to `AttachmentSystem`, not to `ZoneComponent` and not to example-local
MMS handlers. Each active mount edge records at least:

```rust,ignore
struct ActiveMount {
    rider: ComponentId,
    mountable: ComponentId,
    rider_root: ComponentId,
    rider_anchor: ComponentId,
    mount_anchor: ComponentId,
    suspended_input: Option<ComponentId>,
    suspended_input_previous_state: Option<bool>,
    original_parent: Option<ComponentId>,
    dismount_world_pose: TransformTrs,
}
```

Store resolved identities at commit time. Do not repeat a world-global query
each frame and silently switch riders, anchors, or input mappings after topology
changes. Validate cached IDs before use and unwind the edge if a required
participant disappears.

The relationship must be inspectable by rider and mountable, enforce one rider
per single-seat mount, and support deterministic cleanup. Runtime occupancy is
derived from active edges; avoid a second independently mutable `occupied`
boolean on `MountableComponent`.

## Atomic mount transaction

Commit mounting as one coordinated operation:

1. Revalidate eligibility from current world transforms.
2. Snapshot the rider root's parent/world pose and the movement input's prior
   automatic-mapping state.
3. Reject transform cycles before changing either tree.
4. Attach the rider movement root under the vehicle mount basis.
5. Align the rider anchor's full position and orientation with the mount anchor;
   do not snap arbitrary root origins together.
6. Disable only the rider's automatic locomotion mapping.
7. Publish the active edge and a `MountStarted` observation.

If any step fails, roll back parentage, world pose, input state, and occupancy.
The failure result must be observationally equivalent to no mount having
occurred, after which arbitration may use the permitted grab fallback.

Do not disable `InputXR`: tracked head translation and rotation must continue
while mounted. `InputXRGamepad.disable()` relinquishes its built-in pedestrian
locomotion while preserving canonical axis/button events for the vehicle
controller. Desktop `Input.disable()` gates its built-in transform mapping.

## Dismount and nested authority

Provide a repeatable first-slice dismount action. Until a dedicated binding is
selected, pressing grip on the occupied mountable again from its current rider
may toggle that one mount edge. Dismount must:

1. detach the rider root while preserving its current world pose;
2. apply an authored or temporary safe exit offset outside the entry zone;
3. restore the exact previous automatic-input state;
4. clear occupancy and publish `MountEnded`;
5. leave both rider and mountable eligible for another complete cycle.

Represent nesting as a stack/chain of active edges rather than a global
"mounted" flag:

```text
Bisket -> mech -> carrier -> construction vehicle -> station
```

Each new outer edge suspends only the immediately inner movement authority.
Removing an edge restores only the state captured by that edge. Never enable
all descendant inputs during dismount. Full nested mounting is not required for
the first car test, but the stored relationship must not make it impossible.

## Engine integration boundary

Add the native behavior under `engine/ecs`:

- `RiderComponent` and `MountableComponent` own authored configuration and MMS
  serialization;
- an interaction-arbitration path consumes pointer activation plus ordered ray
  hits and chooses one action;
- `AttachmentSystem` owns eligibility, mount/dismount transactions, occupancy,
  active relationships, and lifecycle cleanup;
- `ZoneComponent` remains a detection-only geometry primitive;
- `GrabbableSystem` continues to own existing grabs in this slice.

Prefer a neutral interaction-attempt command/event from gesture recognition
over calling `AttachmentSystem` directly from `GestureSystem`. Native mount
outcome events can later be exposed to MMS without requiring scripts to perform
the transaction themselves.

Do not run mounting through the collision-response worker. Entry eligibility
uses the synchronous zone query against current authoritative transforms.

## First implementation sequence

1. Add and round-trip `RiderComponent` with anchor, movement-root, input, and
   enabled configuration.
2. Add and round-trip `MountableComponent` with entry-zone, mount-anchor,
   activation policy, and enabled configuration.
3. Add pointer-to-rider resolution bounded to the initiating pointer's player
   tree; cover missing and ambiguous associations.
4. Add pure mount eligibility and cycle-check helpers with focused tests.
5. Introduce grip arbitration that chooses eligible mount before grab fallback.
6. Implement atomic mount, same-rider dismount, input restoration, and removal
   cleanup in `AttachmentSystem`.
7. Author `Rider` and `Mountable` in `mittens-corp`, keeping the existing car
   zone and rider/car anchors.
8. Validate repeated XR mount/dismount cycles and ordinary grabbing regressions.

## Acceptance criteria

- Walking Bisket into the car's front zone and gripping while pointing at the
  car mounts exactly once.
- The same grip from outside the entry zone does not mount.
- Entering the zone without grip does not mount.
- A long pointer ray cannot mount from outside merely because it hits the car.
- The rider anchor aligns to the cockpit mount in position and orientation.
- `InputXR` tracking and raw XR gamepad events continue while pedestrian
  locomotion is suppressed.
- The mounted layer receives canonical control events without the inner rider
  also moving independently.
- Dismount preserves a valid world pose, exits the zone, restores the captured
  input state, and permits remounting.
- Mount failure or removal of a required component leaves no partial parent,
  occupancy, or disabled-input state.
- An ineligible `Mountable + Grabbable` target follows the declared grab
  fallback; an eligible one cannot both mount and grab from one activation.
- Existing grabbable examples and tests behave unchanged.
- No mounting code depends on `CollisionResponseComponent` or asynchronous
  collision-worker timing.

## Deferred work

- migrating `Grabbable` onto the shared attachment relationship backend;
- broom release-to-mount and held-object handoff;
- multiple seats, multiple riders, reservation, and mount priority;
- continuous zone enter/exit authoring and scripted eligibility callbacks;
- vehicle velocity/force integration and physical collision ownership;
- polished entry/exit animation and pose transitions;
- full nested-vehicle UI, diagnostics, and recovery policies.

## Related work

- [Interaction zones on the collision-query foundation](interaction-zone-collision-query-foundation.md)
- [Interaction zones, sockets, and vehicle mounting](release-zones-sockets-and-vehicle-mounting.md)
- [E2 broom attachment first slice](e2-broom-mounting-first-slice.md)
- [Scriptable Velocity pose driver](scriptable-velocity-pose-driver.md)
- [Velocity, forces, and pluggable physics](velocity-forces-and-pluggable-physics.md)
