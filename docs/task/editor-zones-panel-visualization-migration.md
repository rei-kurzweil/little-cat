# Task: editor Zones panel and spatial-visualization migration

## Status and outcome

Planned, 2026-09-08. Add an optional, minimizable editor panel for controlling
diagnostic visualization of authored zones and zone-like spatial regions. Move
the existing collider and secondary-motion collider controls out of the Editor
Settings panel only after the new panel reaches behavioral and serialization
parity.

Before this panel work, implement the narrower
[Editor Settings generic-zone visualization first slice](editor-settings-generic-zone-visualization-first-slice.md).
That slice adds the underlying `ZoneVisualizationSystem` and a temporary
Settings toggle to unblock vehicle-entry-zone diagnosis. This task later moves
that same request state into the Zones panel; it must not create a parallel
visualizer or a second independently mutable visibility flag.

Generic red zone markers now render, and the temporary Settings control has a
real-panel off/on/materialize/off cleanup regression. `show_zones` means row
inclusion rather than forced initial visibility. Preserve that single
owner-scoped state when moving the control here; do not create a second flag or
request path.

The authored panel selector is `zones`:

```mms
EditorUI {
    panels([
        { panel = "settings" },
        { panel = "zones" },
        { panel = "world" },
    ])
}
```

Use **Zones** as the visible title and `zones_panel` for internal factory/root
names where a suffix is needed. Follow the existing shared panel shell and
accordion minimization contract; do not create a zones-specific window or
minimize state machine.

This task is a UI and visualization-ownership migration. It does not make zones
physical, does not route them through collision response, and does not change
mount eligibility semantics.

## Current controls and runtime paths

The Settings panel currently contains:

- `show zones`, temporarily backed by `ZoneVisualizationSystem`; its off/on
  lifecycle is covered by the focused predecessor's real-panel regression;
- `show all colliders`, backed by `CollisionVisualizationMode::All`;
- `show GLTF colliders`, backed by `CollisionVisualizationMode::GltfOwned`;
- `show spring bones`, backed by one boolean that visualizes both bound spring
  segments/endpoints and their collider spheres.

The temporary and existing configuration is stored in `SettingsPanelConfig` as
`show_zones`, `show_colliders`, `show_gltf_colliders`, and
`show_spring_bones`. Settings-panel click handling mutates `EditorContextState`
and emits owner-scoped `ZoneVisualizationSet`, `CollisionVisualizationSet`, or
`SpringBoneVisualizationSet` intents. The visualization systems union requests
by owner, restrict them to effective editor roots, create runtime-only markers,
and remove those markers when the request or source disappears.

Preserve those useful ownership and cleanup properties. The migration should
change which panel owns the controls, not introduce a second set of marker
systems or independently mutable visibility flags.

## Panel responsibilities

The Zones panel is the editor's spatial-region diagnostic surface. Its first
rows should independently control:

| Control | Spatial sources shown | Initial implementation |
|---|---|---|
| Authored zones | `ZoneComponent` regions | New |
| Collision zones | all physical `CollisionComponent` shapes | Existing `All` mode |
| GLTF collision zones | collision shapes owned by imported GLTF trees | Existing `GltfOwned` mode |
| Spring-bone colliders | secondary-motion collider spheres only | Split from the existing spring overlay |
| Spring-bone chains | simulated segments/endpoints | Split from the existing spring overlay; diagnostic but not itself a zone |

The visible language may use “colliders” where users already recognize it, but
the implementation should treat collision and spring colliders as consumers or
adapters over the common spatial-region vocabulary. Do not label spring-bone
segments as zones: only their exclusion volumes are spatial regions.

Keep the row model extensible for later categories such as attachment probes,
mount eligibility, sockets, interaction rays, navigation volumes, audio
regions, and physics-backend debug geometry. Do not require those categories
for the first panel.

## Typed configuration

Move spatial visualization defaults into a panel-specific config rather than
adding more booleans to `SettingsPanelConfig`:

```rust,ignore
struct ZonesPanelConfig {
    show_zones: bool,
    show_collision_zones: bool,
    show_gltf_collision_zones: bool,
    show_spring_colliders: bool,
    show_spring_chains: bool,
}
```

Add `EditorPanel::Zones` with the authored name `zones`, and add a corresponding
`EditorUIPanelConfig::Zones(ZonesPanelConfig)`. `EditorUI.panels(...)` parsing,
validation, canonical ordering, MMS serialization, and round-tripping must
accept the new typed configuration.

Omitting `{ panel = "zones" }` must avoid materializing its panel and avoid
installing a visualization request owned only by that panel. A bare
`EditorUI {}` currently means all supported panels; if that behavior is
retained, adding Zones to the supported `ALL` set will show it by default. If
the product intent is “available but absent from the bare default,” split the
concepts into `SUPPORTED` and `DEFAULT` rather than making `ALL` lie. Settle
that default explicitly during implementation.

Suggested authored configuration:

```mms
{
    panel = "zones"
    config = {
        show_zones = true
        show_collision_zones = false
        show_gltf_collision_zones = false
        show_spring_colliders = true
        show_spring_chains = false
    }
}
```

Defaults should favor useful diagnostics without covering the scene in every
available overlay. Preserve explicitly authored old Settings defaults during
migration rather than silently changing an example's initial presentation.

## Shared visualization state

Use one authoritative spatial-visualization state per editor workspace. Both
panel initialization and row clicks update that state, and the visualization
systems consume it through owner-scoped requests. Avoid keeping one copy in
`SettingsPanelConfig`, another in Zones-panel toggles, and a third in
`EditorContextState` after the migration.

A suitable first typed request is conceptually:

```rust,ignore
struct SpatialVisualizationRequest {
    scope_roots: Vec<ComponentId>,
    categories: SpatialVisualizationCategories,
}

struct SpatialVisualizationCategories {
    zones: bool,
    collision_all: bool,
    collision_gltf_owned: bool,
    spring_colliders: bool,
    spring_chains: bool,
}
```

It is acceptable to retain separate collision, zone, and secondary-motion
visualization systems internally, provided they receive one coordinated panel
state and do not produce duplicate markers. Preserve request union semantics
when multiple editor workspaces or diagnostic owners are present.

`CollisionVisualizationMode::All` and `GltfOwned` are currently mutually
exclusive. The panel may preserve that behavior with a selection control or
replace it with explicitly documented filter semantics. Do not render a GLTF
collider twice when both rows are enabled.

## Zone visualization system

Add visualization for `ZoneComponent` without adding a renderable to authored
zone trees. Runtime markers must:

- resolve the same effective transform and normalized `CollisionShape` used by
  synchronous zone queries;
- support cubes, spheres, and Y capsules with translation, rotation, and
  non-uniform scale;
- remain non-raycastable so diagnostics cannot intercept selection, grabbing,
  or mounting;
- use `Serialize.off()` and never appear in saved authored MMS;
- be removed when the source zone, request owner, or scope root disappears;
- follow enable, shape, reference, role, and transform changes;
- visually distinguish disabled or unresolved zones if those diagnostic modes
  are intentionally supported, otherwise omit them consistently;
- avoid registering with collision detection or collision response.

Reuse the existing collision-marker mesh/material helpers where practical.
Do not create a second cube/sphere/capsule tessellation vocabulary merely
because the control moved to a new panel.

Use stable category colors and include a compact legend or row swatch so users
can distinguish authored zones, physical colliders, spring colliders, and
attachment probes. Exact colors are a UI choice, but opacity and depth behavior
must remain readable in both the desktop view and XR.

## Splitting secondary-motion diagnostics

The current `SpringBoneVisualizationSystem` derives both collider markers and
chain segment/endpoint markers from one request. Refactor its request to carry
at least `show_colliders` and `show_chains` independently. Both categories may
reuse the same bound-chain snapshot, but disabling chains must actually remove
or stop reconciling segment/end-point marker trees while leaving requested
collider spheres visible, and vice versa.

This split must not affect secondary-motion simulation. Visualization remains a
read-only consumer of bound solver state.

## Optional panel and minimization behavior

Build the panel through the generic panel factory and `PanelKind`/`PanelBodySpec`
path documented by
[editor panel minimize and render suspension](editor-panel-minimize-and-render-suspension.md).
The expanded body contains the category rows; the minimized state retains only
the common title-bar shell.

Minimizing the Zones panel must not change visualization choices. Minimization
is presentation state, so active overlays remain active while the body is
suspended. Toggle state must restore correctly when the body is rematerialized,
without adding duplicate handlers or requests.

Removing or omitting the Zones panel is different from minimizing it. When the
panel is the sole owner of a visualization request, removing its panel instance
must remove that request and its runtime markers. Late refreshes after removal
must not recreate the body or overlays.

## Staged Settings migration

Do not delete the Settings rows first. Use this order:

1. Add `EditorPanel::Zones`, typed configuration, generic panel shell/body, and
   minimization support.
2. Connect its rows to the existing collision and secondary-motion requests.
3. Add authored `ZoneComponent` visualization and split spring colliders from
   spring chains.
4. Verify initial state, toggle synchronization, owner cleanup, and
   minimize/restore behavior.
5. Migrate example `EditorUI` panel specs and their authored defaults from
   Settings config to Zones config.
6. Remove `show_colliders`, `show_gltf_colliders`, and `show_spring_bones` from
   `SettingsPanelConfig`, its MMS parser/serializer, panel factory arguments,
   row construction, click handling, and tests.
7. Remove compatibility parsing only after repository-authored MMS and any
   documented migration window are handled.

There must never be two simultaneously visible control surfaces that can drift
while mutating the same underlying state. During the transition, either route
both surfaces through the same state and synchronization path or keep the new
panel behind a development-only selection until the cutover.

The Settings panel retains interaction mode, armature, bounds, and camera
controls unless a separate diagnostics-panel cleanup explicitly moves them.
This task moves spatial-region and secondary-motion spatial diagnostics only.

## First implementation sequence

1. Decide whether bare `EditorUI {}` includes Zones by default; encode separate
   supported/default lists if necessary.
2. Add `EditorPanel::Zones`, `ZonesPanelConfig`, MMS validation, serialization,
   canonical ordering, and round-trip tests.
3. Add `zones_panel` factory/body content to the shared internal panel asset and
   materialize it through `PanelSystem`.
4. Add click handling and toggle synchronization using one workspace-owned
   visualization state.
5. Implement `ZoneComponent` runtime marker reconciliation using shared shape
   geometry and effective transforms.
6. Split spring collider and spring chain visualization requests.
7. Exercise minimize/restore/removal lifecycle and verify stable component,
   handler, request, and marker counts.
8. Migrate examples and remove the old Settings rows/config fields.

## Acceptance criteria

- `{ panel = "zones" }` materializes one panel through the standard editor
  shell and omission does not materialize it.
- The panel minimizes and restores with the same behavior as every other panel;
  repeated cycles do not leak nodes, handlers, requests, or markers.
- Authored zones, all collision zones, GLTF collision zones, spring colliders,
  and spring chains can be controlled according to their documented independent
  or filtered semantics.
- Zone markers use the exact query transform/shape interpretation and follow
  live edits without becoming collidable, raycastable, or serializable.
- Spring colliders can be shown without spring segments, and spring segments can
  be shown without collider spheres.
- Minimizing preserves active visualization state; removing the owning panel
  cleans up its requests and overlays.
- Multiple request owners union predictably and never duplicate a source
  marker.
- The Settings panel no longer displays or serializes its old collider and
  spring-bone visualization controls after migration.
- Existing authored initial visibility choices survive example migration.
- Collision detection, zone queries, secondary-motion simulation, grabbing,
  and mounting behave identically with every visualization category off or on.

## Performance and validation

Record expanded/minimized panel node and handler counts, active marker counts by
category, and representative frame cost with all categories off and on. With
all categories disabled, visualization systems should perform cleanup and
cheap request checks rather than rebuild geometry. Reuse cached source identity
and update transforms/materials incrementally where possible; broadphase or
collision-worker participation is not required for diagnostic rendering.

Manual validation should include:

- desktop and XR readability of overlapping zone/collider categories;
- a transformed, rotated, and non-uniformly scaled zone;
- live creation, disable, reference change, and removal of a zone;
- GLTF reload/removal while its collision visualization is active;
- secondary-motion chain rebind/removal with only colliders visible and with
  only chains visible;
- at least 100 minimize/restore cycles followed by panel removal.

## Related work

- [Editor Settings generic-zone visualization first slice](editor-settings-generic-zone-visualization-first-slice.md)
- [Interaction zones on the collision-query foundation](interaction-zone-collision-query-foundation.md)
- [Rider + Mountable attachment-system first slice](rider-mountable-attachment-system-first-slice.md)
- [Editor panel minimize and render suspension](editor-panel-minimize-and-render-suspension.md)
- [Live panel factory and stopgap adapter seams](live-panel-factory-spawn-and-stopgap-adapter-seams.md)
- [Editor UI live MMS panel validation slices](editor-ui-live-mms-panel-validation-slices.md)
