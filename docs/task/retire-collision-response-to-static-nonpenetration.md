# Task: retire collision response to a static non-penetration constraint

## Status and outcome

Planned, 2026-09-08. Replace the general
`CollisionResponseComponent`/`CollisionResponseSystem` with the one behavior we
still need in the current engine: static collision geometry prevents an
explicitly pose-driven movable object, especially an avatar locomotion root,
from ending a frame inside it.

Remove kinetic response behavior. The retained constraint has no mass, forces,
gravity, friction, restitution, bounce, momentum, or private velocity. It does
not simulate movable bodies and does not resolve movable-versus-movable contact.
It is an intentionally narrow transition architecture that may later be
replaced by a pluggable physics backend.

This task does not remove collision shapes, overlap detection, collision events,
zone queries, IK, animation, transform streams, or secondary motion.

## Current dependency and problem

The current response component combines unrelated responsibilities:

- static penetration correction (`slide`);
- acceleration away from non-static overlaps (`push`);
- gravity integration;
- friction and speed limiting;
- side-wall restitution/bounce;
- a private runtime velocity accumulator;
- transform movement-target resolution.

AvatarControl currently generates a kinematic capsule with
`CollisionResponse.slide()` and routes its correction to the actual locomotion
root. Several examples also instantiate slide or push responses. Removing the
system without migrating those consumers would allow avatars to pass through
floors and walls and would break the examples.

The response system should not become the foundation for zones, attachments,
broom flight, or future dynamics. Those uses need spatial queries, explicit
motion ownership, and first-class velocity instead.

## Retained contract

Introduce a narrowly named component and system; provisional name:

```mms
T {
    Collision.movable() {
        CollisionShape.capsule_y(0.28, 0.62)
        StaticCollisionConstraint {
            movement_target("#avatar_movement_root")
        }
    }
}
```

The exact nesting and builder syntax remain subject to the component pass. The
semantic contract is fixed:

- The collider supplies the proposed pose after input, animation, attachment,
  or another pose driver has run.
- Only overlaps against collision geometry designated static are considered.
- The constraint calculates the minimum world-space correction needed to leave
  static geometry and applies that displacement to its resolved movement target.
- It stores no velocity and performs no free integration.
- It never pushes the static object or another movable object.
- It does not infer bounce, sliding velocity, gravity, friction, or momentum.
- An unresolved movement target results in no correction and a diagnostic; it
  must not move an arbitrary nearby transform.
- Multiple corrections in one frame have deterministic ordering and a bounded
  iteration count. Failure to converge is observable.

The initial implementation may preserve the current discrete MTV correction
and capsule/box/sphere geometry to keep AvatarControl working. This is
containment cleanup after a pose, not a robust continuous character controller.
Tunneling, stairs, slopes, moving platforms, step offsets, and swept collision
remain explicit limitations.

## Motion and authority

Call the retained object **pose-driven** or **movable**, not kinetic. Its pose is
owned by input, animation, attachment, a velocity driver, or some other named
source. Static contact is a constraint on that proposed pose, not a second
integrator.

If a first-class `VelocityComponent` is present, this transitional constraint
does not integrate it. A later policy may project or zero velocity along a
contact normal, but that belongs to the velocity/physics contract and must be
explicit. The first migration can leave commanded velocity unchanged and only
correct the pose, provided the limitation is documented so a driver does not
silently acquire fake bounce behavior.

Exactly one system owns the movable transform update at a time. Applying the
constraint through the existing world-displacement/movement-target path is
acceptable as an intermediate implementation, but the correction must occur
after the proposed pose and before cameras and dependent interaction queries
consume the final world transform.

## Migration plan

1. Inventory every authored and generated `CollisionResponse` consumer.
   AvatarControl's generated capsule is the required compatibility case.
2. Characterize existing floor and wall non-penetration with focused tests,
   including the movement-target routing used by desktop and XR avatars.
3. Add `StaticCollisionConstraintComponent` and migrate only required slide
   users. Preserve generated-runtime cleanup and serialization behavior.
4. Remove `CollisionResponse.push()`, non-static repulsion, gravity integration,
   friction, restitution, speed limiting, and the private velocity accumulator.
5. Remove or rewrite the collision-perimeter/gravity examples that exist only
   to demonstrate the retired solver. Do not preserve obsolete behavior merely
   to keep a demo unchanged.
6. Remove `CollisionResponseComponent`, its registration/removal intents, system
   scheduling, MMS constructors/methods, serialization tests, and old spec.
7. Rename/revisit `CollisionMode::Kinematic` and `Rigged`. Detection roles and
   movement authority should not be encoded in one ambiguous enum.

## Performance and scheduling

The transitional constraint should query only registered pose-driven
participants against static broadphase candidates. Do not rebuild or scan all
collision pairs solely to constrain one avatar. Track dirty transforms and
avoid work for unchanged participants where correctness allows it.

Keep shape resolution and narrow-phase math shared with ordinary collision and
zone queries. Do not fork capsule/box/sphere intersection implementations.

Record at least candidate-pair count, narrow-phase test count, correction
iterations, and non-convergence count so the retained path can be compared with
a future backend.

## Acceptance criteria

- Desktop and XR AvatarControl capsules remain outside static floors and walls.
- Pose-driven objects are not automatically pushed by other movable objects.
- No retained component contains velocity, gravity, friction, restitution,
  bounce, force, or mass state.
- `CollisionResponse.push()` and its private velocity accumulator are removed.
- Collision detection/events and `Zone` queries continue working without a
  response component.
- System ordering exposes one final corrected pose to cameras and interaction
  consumers.
- Tests document discrete-correction limitations rather than implying robust
  rigid-body or character-controller behavior.

## Related work

- [Spatial, collision, and physics naming](spatial-collision-and-physics-naming.md)
- [Velocity, forces, and pluggable physics](velocity-forces-and-pluggable-physics.md)
- [Interaction zones on the collision-query foundation](interaction-zone-collision-query-foundation.md)
- [AVC auto-calibrated upright capsule](avc-upright-character-capsule.md)
- [Velocity / AngularVelocity components WIP](wip/velocity-components.md)
