# Task: attachment valence and Grabbable unification

Status: planned, 2026-09-12. Follows the `Rider` + `Mountable` first slice.

## Attachment valence model

Use signed attachment valence relative to a user-controlled frame, rather than
making humanoids the root concept:

| Valence | Role | Meaning | Default movement authority |
| --- | --- | --- | --- |
| -1 | carried/equipped item | a held prop, wearable, hat, glasses, tool, or gear rigidly following a controlled frame/socket | none |
| 0 | user-controlled frame | an avatar body, creature rig, camera rig, or another transform basis a user directly controls | owns pedestrian control |
| +1 | mount | a vehicle, broom, mech, platform, or carrier that the controlled frame can enter/follow | may offer the next control layer |

Attachment valence describes an individual edge, not an object's fixed global
type, scene depth, or number of sockets.  A vehicle at `+1` can itself become the controlled participant
for a carrier at `+2`; an equipped item can have its own rigid children.  A
level-`-1` holdable or wearable may attach to any compatible anchor, including
one owned by a level-0 controlled frame, a level-`+1` mount, or another item
when that policy permits it.  A humanoid is one possible level-0 frame, not a
requirement: any user-controlled transform tree can expose named or authored
anchors.

## Existing vocabulary

Keep the present terms narrow:

- `Rider` is the level-0 occupancy participant.  It is an attachment-system
  role and supplies the movement root/placement anchor needed by the
  movement-authority handoff.  It does not mean every attachable object.
- `Mountable` is a level-+1 attachment-system destination.  Its general job is
  accepting a rider relationship; a vehicle is only one implementation.  A
  mount need not itself be humanoid-shaped or user-controlled.
- `Grabbable` remains an interaction affordance: it says a pointer can acquire
  an object.  It is not yet the retained attachment relation.

`Wearable` and a future `Holdable` belong at level -1.  They should share a
common attachment-item foundation, but remain separate semantic policies:

- `Holdable` supplies grab pose, release/throw behavior, pointer eligibility,
  and a temporary attachment to a compatible hand, handle, or other anchor.
- `Wearable` supplies equip/unequip policy, target-anchor eligibility, and a
  persistent rigid attachment.

An item may offer both roles: glasses can be grabbed, then equipped to a head
socket; a tool can be held, then placed in a belt socket; a mounted vehicle can
offer compatible sockets for equipment, dashboard items, handles, or cargo.
Do not use `Rider` for any of those cases, and do not make them transfer
pedestrian movement authority merely because a transform was parented.

## Current implementation and problem

`GrabbableSystem` already creates a temporary parent relationship while held:
it reparents the grabbed target beneath the pointer's nearest transform,
preserves world pose, smooths the held local destination, then restores the
original parent on release.  Its active-edge bookkeeping and lifecycle are
private to `GrabbableSystem`, while `AttachmentSystem` separately owns mounted
Rider-to-Mountable edges.

That duplicates the core concerns of attachment: identity of child/target,
original parent, authoritative lifetime, removal cleanup, cycle protection,
and transform preservation.  The interaction policies are legitimately
different, but their retained attachment edge should not be implemented twice.

## Goal

Create one low-level, directionally generic attachment-edge foundation used by
grab/hold, equip/wear, and ride/mount.  It must describe parent/child or
follow-anchor relations without assuming a humanoid, a vehicle, a pointer, or
movement authority.

Targets must be expressed as generic authored attachment anchors.  `Mountable`
owners may expose such anchors just as a user-controlled frame can; attaching
an item to one does not turn that item into a rider or activate the mount's
movement controls.

Higher-level systems remain responsible for their own policies:

```text
Gesture / interaction system
  chooses a valid Grabbable or Mountable activation
        |
        +--> Holdable / Grabbable policy -> generic attachment edge (level -1)
        +--> Wearable/equip policy       -> generic attachment edge (level -1)
        +--> Rider / Mountable policy    -> generic attachment edge (0 -> +1)
                                                   |
                                                   +--> MovementAuthoritySystem,
                                                        only when that role pair
                                                        requests a handoff
```

The generic edge must not imply that every child is literally ECS-parented.
Rigid parentage is appropriate for the first wearable pass; later constraints,
physics, skin binding, and retargeted bone attachment may use a different
runtime mechanism while preserving the same relationship semantics.

## Implementation slices

1. Inventory the exact invariants shared by `GrabbableSystem` and
   `AttachmentSystem`: atomicity, cycle checks, original-parent restoration,
   world-pose preservation, single-owner policy, removal cleanup, and
   lifecycle events.
2. Introduce a generic active attachment-edge representation and a public
   query/cleanup boundary.  Do not migrate behavior by exposing one system's
   private maps to another.
3. Rebuild current temporary grabs on that edge while retaining their existing
   ray clearance, smoothing, release, and interaction-priority semantics.
4. Define `Holdable` as the policy layer above the retained grab edge.  Decide
   whether `Grabbable` becomes that role, remains the activation marker, or
   composes with it; preserve existing MMS scenes during migration.
5. Add rigid `Wearable`/equip-to-socket behavior.  Support arbitrary named,
   authored anchors first; defer skinning/deformation attachment.
6. Rebuild the Rider-to-Mountable path on the same foundation, then allow the
   movement-authority system to react only to the occupancy capability.
7. Add nesting, ownership, save/load, removal, and conflict rules across held,
   worn, and mounted relationships.

## Acceptance criteria

- A non-humanoid user-controlled frame or a mount can hold or wear an item
  through an authored compatible anchor without special-case humanoid code.
- A prop can be grabbed, released, equipped, unequipped, and grabbed again
  without orphaned parentage, transform jumps, or duplicate ownership.
- A `Grabbable` item and a `Mountable` vehicle still compete deterministically
  for a single interaction activation.
- Held/worn edges do not alter pedestrian movement authority.
- Rider-to-Mountable edges alone can request a movement-authority layer.
- Removal of either endpoint unwinds the edge safely and emits a coherent
  lifecycle result.
- Existing grab clearance, placement smoothing, and current mount alignment
  retain their behavior after migration.

## Related work

- [Separate mounting from movement authority and input routing](attachment-movement-authority-and-input-routing.md)
- [Rider + Mountable attachment-system first slice](rider-mountable-attachment-system-first-slice.md)
- [Interaction zones, sockets, and vehicle mounting](release-zones-sockets-and-vehicle-mounting.md)
- [Grab hand-relative bounds placement](grab-hand-relative-bounds-placement.md)
