# Task: Editor Settings generic-zone visualization first slice

## Status and outcome

Planned, 2026-09-09. Add one temporary **show zones** toggle to the existing
Editor Settings panel and a `ZoneVisualizationSystem` that draws authored
`ZoneComponent` regions. This is the smallest diagnostic slice needed to tune
and validate the `mittens-corp` vehicle-entry zone before building the dedicated
Zones panel.

This slice visualizes only actual `ZoneComponent`s. Existing
`CollisionComponent` shapes and secondary-motion colliders keep their existing
visualization paths and colors. They are expected to become zone consumers or
adapters later, but that migration is deliberately out of scope here.

## Why this comes first

Mounting currently performs a synchronous point-in-zone query when grip begins.
The authored entry volume is otherwise invisible, which makes these different
failure modes indistinguishable in-headset:

- the rider anchor is outside the entry zone;
- the zone frame or component reference resolves somewhere unexpected;
- the car-local zone translation, rotation, or size needs tuning;
- the ray hit does not resolve to the `Mountable` owner;
- the grip activation is not reaching mount arbitration.

Showing the exact region used by `zone_query` makes the first three cases
directly observable without adding temporary renderables to the authored scene.

## First-slice UI contract

Extend the existing Settings panel configuration with one temporary field:

```rust,ignore
pub struct SettingsPanelConfig {
    // existing fields...
    pub show_zones: bool,
}
```

Add a **show zones** boolean row beside the existing collider diagnostics. The
MMS configuration is:

```mms
EditorUI {
    panels([
        {
            panel = "settings"
            config = { show_zones = true }
        },
    ])
}
```

The setting should default to `false` globally so adding this feature does not
cover existing scenes in overlays. Author `show_zones = true` in
`mittens-corp` while the mounting fixture is being tuned.

Use `zones_visible: bool` in `EditorContextState`. A Settings-row click toggles
that state and emits an owner-scoped request:

```rust,ignore
IntentValue::ZoneVisualizationSet {
    component_id: editor_ui,
    scope_roots: effective_editor_roots,
    visible,
}
```

Use the owning `EditorUIComponent` as the request owner, matching collision and
spring visualization. Removing that owner must eventually remove its request
and markers.

This Settings field and row are intentionally transitional. The later Zones
panel migration moves the state rather than creating a second independent
toggle.

## Zone visualization system

Add a `ZoneVisualizationSystem` alongside `CollisionVisualizationSystem`. It
maintains retained runtime-only markers keyed by source `ZoneComponent` and
union-combines requests from all live editor owners.

The system must use the same resolved frame and normalized `CollisionShape` as
the synchronous query implementation. Extract or expose shared zone-frame
resolution rather than recreating subtly different `.at(...)` component-ref
semantics in the visualization system.

For every enabled, resolved zone under a requested scope:

- cube: draw a unit cube scaled to twice its local half-extents;
- sphere: draw a unit sphere scaled to twice its local radius;
- Y capsule: reuse the cached capsule mesh vocabulary already used by collider
  visualization;
- apply the resolved frame's full world translation, rotation, and scale;
- use a translucent red diagnostic style, initially
  `Color.rgba(1.0, 0.15, 0.15, 1.0)` with opacity near `0.22`;
- remain non-selectable and non-raycastable;
- carry `Serialize.off()` and never enter saved MMS;
- never create `Collision`, `CollisionShape`, or collision-response components.

The red style deliberately differs from the current cyan physical-collider
overlay. Exact opacity may be tuned in desktop and XR, but generic zones and
physical colliders must remain visually distinguishable when both toggles are
enabled.

Disabled or unresolved zones should be omitted for this slice. Diagnostic
rendering must not invent a fallback frame or shape that disagrees with
`classify_zone_point`.

## Transform correctness

The current collision visualizer places markers from a world position and does
not represent parent rotation or scale. Do not copy that limitation into zone
visualization. A zone query transforms the probe through the inverse of the
resolved frame's complete world matrix, so its marker must represent the same
matrix.

For a shape-local marker matrix:

```text
marker_world = resolved_zone_frame_world * shape_local_scale
```

The implementation may update a marker transform directly or use a runtime
transform-parent operator, provided rotation and non-uniform scale agree with
the query result. Capsule dimensions must not be applied twice if the cached
mesh is already authored at the requested radius and half-segment.

## Reconciliation and performance

The first implementation may enumerate world components while at least one
zone-visualization request is active, matching the current collision
visualizer. When no request is active it should only perform request cleanup and
marker removal; it must not scan or rebuild the scene indefinitely.

Retain one marker per visible source zone. Rebuild its geometry only when the
normalized shape changes. Update placement when its resolved frame changes,
and remove the marker when:

- the source zone is removed or disabled;
- its frame no longer resolves;
- it leaves every requested scope;
- the last request is disabled or its owner is removed.

Do not emit a generic signal/transform dependency flush per unchanged marker if
the marker can be updated through the visualization-owned retained path. This
is especially important in XR and follows the lessons from spring-bone
visualization performance work.

## `mittens-corp` validation

With `show_zones = true`, the independent car should display one translucent red
front-entry box derived from:

```mms
let car_front_zone_frame = T.position(0.0, 0.15, 3.5) {
    name = "left_display_car_front_zone_frame"
}
car_front_zone_frame
Zone.cube([4.6, 3.8, 0.8])
    .at(car_front_zone_frame)
    .role("vehicle_entry")
```

The rendered box must move and rotate with the car and agree with mount
eligibility at its visible boundary. The relevant probe is the world position
of `bisket_rider_cxr_anchor`, not either hand or ray-hit position.

If Bisket's anchor is visibly inside the red region and gripping the car still
does not mount, continue diagnosis in pointer-to-Rider association, ray-hit to
Mountable resolution, and grip arbitration rather than expanding the zone.

## Implementation sequence

1. [ ] Add `show_zones` to `SettingsPanelConfig`, MMS parsing/serialization,
   builders, and round-trip tests; default it off.
2. [ ] Add `zones_visible` to `EditorContextState` and the Settings panel's
   initial-state synchronization.
3. [ ] Add the **show zones** Settings row and click handling.
4. [ ] Add `ZoneVisualizationSet` signal routing and request ownership.
5. [ ] Implement retained `ZoneVisualizationSystem` markers using shared zone
   frame/shape resolution.
6. [ ] Register and tick the system in `SystemWorld`, including owner/source
   cleanup.
7. [ ] Enable the setting in `mittens-corp` and validate the entry box in VR.
8. [ ] Tune the car entry-zone frame/shape based on the visible rider anchor and
   verify inside/outside mount behavior.
9. [ ] Later, migrate the toggle and request state into the dedicated Zones
   panel without changing marker ownership or visual semantics.

## Acceptance criteria

- Editor Settings contains one synchronized **show zones** toggle.
- The toggle shows all enabled `ZoneComponent`s under the effective editor
  roots and no `CollisionComponent` or spring-only collider shapes.
- Cube, sphere, and capsule markers agree with point classification after
  translated, rotated, and non-uniformly scaled frames.
- `.at(...)` query and GUID references resolve identically for queries and
  visualization.
- Markers are red/translucent, runtime-only, non-selectable, non-raycastable,
  and non-physical.
- Toggling repeatedly does not duplicate or leak markers or handlers.
- Removing/disabling a zone or removing the request owner cleans up its marker.
- Visualization does not change zone queries, collision behavior, grabbing, or
  mounting.
- `mittens-corp` exposes the car entry region clearly enough to determine why a
  grip is or is not mount-eligible.

## Deferred to the broader Zones-panel task

- adding `EditorPanel::Zones` or `zones_panel` UI factories;
- moving/removing the existing collider and spring rows;
- representing `CollisionComponent` shapes as zones;
- representing spring-bone colliders as zones;
- splitting spring-chain visualization from spring-collider visualization;
- legends, category filters, role filters, attachment probes, and socket
  diagnostics.

## Related work

- [Editor Zones panel and spatial-visualization migration](editor-zones-panel-visualization-migration.md)
- [Interaction zones on the collision-query foundation](interaction-zone-collision-query-foundation.md)
- [Rider + Mountable attachment-system first slice](rider-mountable-attachment-system-first-slice.md)
- [Spring-bone visualization command-flush performance](spring-bone-visualization-command-flush-performance.md)
