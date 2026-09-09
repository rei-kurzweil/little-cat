# Collision-system audit follow-up

Date: 2026-07-19, updated 2026-09-08

Status: planned.

`CollisionResponseComponent` and `CollisionResponseSystem` are deprecated. Keep
collision shape/query/event support, but do not extend the response layer. Its
private velocity, gravity integration, push, bounce, friction, and penetration
correction must be migrated or removed in favor of explicit motion authority and
first-class Velocity work. See
[interaction zones on the collision-query foundation](interaction-zone-collision-query-foundation.md)
for the detection/zone boundary and
[retire collision response to static non-penetration](retire-collision-response-to-static-nonpenetration.md)
for the replacement/removal sequence.

- [ ] Inventory and migrate AvatarControl and example uses of
      `CollisionResponse.slide()`/`push()`.
- [ ] Remove the private response velocity accumulator after its consumers have
      an explicit `VelocityComponent`/velocity-driver replacement.
- [x] Retain avatar/static non-penetration temporarily as a narrow pose
      constraint rather than general kinematic/kinetic response.
- [ ] Remove response registration, intents, serialization, system scheduling,
      tests, and the obsolete response spec after migration.
- [ ] Preserve `CollisionShape`, overlap queries, and start/end observations.
- [ ] Extract transform-aware synchronous shape queries shared by collision and
      the new `Zone` component.

- [ ] Decide whether `CollisionMode::Kinematic` should become `Movable`.
- [ ] Specify exact Static, Movable, and Rigged semantics and collision matrices.
- [ ] Move movable-pair response, displacement sharing, continuous collision,
      manifolds, resting stability, stairs/slopes, bounce, mass, and simulation
      authority to the pluggable-physics work; do not retain them in the legacy
      response system.
- [ ] Add bounds-to-shape heuristics for boxes, spheres, capsules, skeletons, selected meshes,
      and accessory exclusion.

Longer-term dynamics and performance work belongs in
[velocity, forces, and pluggable physics](velocity-forces-and-pluggable-physics.md).

This audit must not retroactively broaden the AVC capsule task. Characterize its
current correction and scheduling before migration; then remove obsolete
velocity, friction, gravity, and restitution behavior explicitly rather than
silently changing it in place.
