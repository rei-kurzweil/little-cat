# Anime shading controls and a future Shading component

Status: design only. No engine implementation in this change.

The native Slider is implemented; use [slider.mms](../../examples/slider.mms)
as the working reference for `SliderChanged`, readouts, and silent `sync_value()`.
Use that control to configure the right-hand Bisket in
[shading-models.mms](../../examples/shading-models.mms) through the reusable
[info_panel](../../assets/components/ui/info_panel.mms). Keep the left-hand
model and matching lights as the comparison baseline.

This task includes the engine/MMS update path needed to change anime shader
inputs while the scene is running, as well as the authored panel. Constructor
configuration alone is insufficient: edits must reach the rendered material.

## Phases

1. Live AnimeShading scalar setters and five Slider rows in `info_panel`.
2. Live shade/rim color setters and color controls.
3. Design `Shading` (MMS) / `ShadingComponent` (Rust) for selecting custom
   shading models and owning typed input state, then agree on an implementation
   slice. This phase is a design session, not a prerequisite for phases 1–2.

Phase numbering here is local to this task, independent of the
[Materials v2 epic](epic/materials-v2.md).

## Phase 1: scalar panel composition and settings

Pass a block container as `info_panel({ ..., content = rows })`. Each row
uses `display("flex")`, `flex_direction("row")`, and centered cross-axis
alignment: fixed-width label, growing slider cell, fixed-width value readout.
The enclosing `display("block")` container stacks rows. Explicit row heights
avoid depending on block auto-height behavior for the first example.

`info_panel` already has a flex-column content wrapper; place the block rows
container inside it. Reuse the slider task's authored input wrapper and native
track/thumb mounts for presentation. No custom MMS drag implementation is needed.

Start with these controls. Defaults come from
[bisket_anime_shading.mms](../../assets/components/materials/bisket_anime_shading.mms),
not the engine's generic AnimeShading defaults:

| Control | Initial value | Proposed UI range | Step |
| --- | --- | --- | --- |
| Shade strength | 0.50 | 0–1 | 0.01 |
| Shade threshold | 0.40 | 0–2 | 0.01 |
| Lit threshold | 0.55 | 0–2 | 0.01 |
| Rim strength | 0.38 | 0–1 | 0.01 |
| Rim power | 4.0 | 0.01–16 | 0.01 |

Keep shade and rim colors at their Bisket preset values in phase 1. Color
controls and live color setters belong to phase 2.

These are useful example ranges, not new engine limits: thresholds currently
accept nonnegative values and rim power is clamped to 0.01–128. Keep
`shade_threshold <= lit_threshold`. Match existing builder normalization:
raising shade above lit also raises lit; lowering lit below shade also lowers
shade. Synchronize both affected controls and readouts silently.

Keep settings outside the panel body. Initialize the material and controls
from the same settings, and provide Reset to restore the five scalar defaults.

### Live shader-input update path

Resolve and retain the authored AnimeShading component beneath the right-hand
GLTF. Slider value changes should update its typed material parameters through
the normal live component mutation API. Inspect existing builder/live method
dispatch first, then implement any missing methods or update notifications.
Keep the API consistent with other component updates; final method names are
an implementation decision, not an assumed existing interface.

The required path is:

1. A slider emits its normalized numeric value; the panel changes the associated
   scalar in its retained settings.
2. The live AnimeShading component validates the update using the same rules as
   initial construction, including threshold ordering and non-finite handling.
3. The material update reaches every generated renderable projection associated
   with that authored component, including projections created after the edit.
4. The renderer refreshes the affected shader parameter data so the next
   rendered frame using that update reflects the new values.
5. Controls and readouts synchronize with the effective sanitized values,
   including any coupled threshold change, without emitting recursive edits.

Audit registration, projection ownership, material caches, and GPU parameter
upload/invalidation so an update cannot stop at a CPU-side component field.
Reuse existing update machinery where possible and fill its gaps. This task
targets typed AnimeShading inputs; a general arbitrary shader/uniform API is
outside phases 1–2.

Apply updates during dragging, not only on commit. Do not reload the GLTF,
recompile shaders, recreate geometry, or rebuild panel rows for parameter-only
changes. Preserve the latest value if updates are coalesced within a frame.
Reset must use the same live update path and restore the phase's exposed
parameters coherently.

### Current implementation and gaps (2026-09-07 source audit)

- `src/scripting/runtime_config.rs` registers AnimeShading constructors and
  builders, but no live methods. Add five typed scalar setters and dispatch in
  `src/scripting/component_method_registry.rs`. Proposed names are
  `set_shade_strength`, `set_shade_threshold`, `set_lit_threshold`,
  `set_rim_strength`, and `set_rim_power`.
- Reuse `AnimeShadingComponent` builder normalization. Specify effective-value
  readback or setter results so the panel can synchronize coupled thresholds
  without duplicating engine validation in MMS.
- `Emissive.set_intensity()` is the closest existing live-method precedent:
  validate the target and route an update through the normal intent machinery.
  Anime already has `RegisterAnimeShading`; decide whether setters can mutate
  the source and emit that existing intent, or need a dedicated update intent.
  Keyframe/Transition support is not required for this panel.
- `RenderableSystem::register_anime_shading` already propagates authored source
  values to GLTF primitive projections, including pending renderables.
  `VisualWorld::update_anime_shading` already invalidates draw caching. The GLTF
  system has a regression test for manual source mutation and re-registration;
  extend coverage through MMS rather than rebuilding this propagation path.
- `VulkanoRenderer::get_or_create_material_set` keys cached UBO/descriptor sets
  by all anime parameter values. Previously unseen combinations allocate new
  resources. The observed eviction handles texture replacement, not historical
  slider values. Phase 1 must bound retention during sustained dragging, using
  reclamation or reusable storage that respects in-flight GPU work. Do not
  require the general phase-3 material architecture to solve this.
- Remove the stale “native Slider is not implemented” comment in
  `examples/shading-models.mms` when wiring the panel.

### Panel lifecycle

Minimizing currently removes the accordion body. On
`AccordionRestoreRequested`, build fresh rows from retained settings, wrap
them with the existing `accordion_body` helper, and attach them to the event's
body mount. Reinstall handlers only for the new controls; removed rows must
not retain active subscriptions. Material settings survive minimize/restore.

### Phase 1 implementation and acceptance

1. Verify and complete live AnimeShading mutation, normalization, projection
   propagation, and renderer parameter refresh before wiring the full panel.
2. Build the reusable panel content and instantiate it in `shading-models.mms`.
3. Verify all five scalars visibly update the anime instance while
   the default instance remains unchanged, including under active animation.
4. Verify threshold coupling, effective-value readouts, silent synchronization,
   and Reset against the shared Bisket preset.
5. Verify minimize/restore preserves settings without duplicate handlers, and
   removing controls during a drag releases interaction safely.
6. Add focused regression coverage for a live authored material update reaching
   its generated projections and renderer parameters. Include distinct material
   instances to catch updates leaking to unrelated renderables.
7. Exercise the example to confirm dragging produces live visual feedback
   without model reloads or panel rebuilds.
8. Cover an edit before GLTF projection creation, and verify the latest source
   values reach newly created projections.
9. Exercise sustained changing values and verify GPU material cache retention
   is bounded. Record allocation/cache counts and frame cost, including the
   number of primitive projections, rather than assuming a slider is cheap.

## Phase 2: color controls

Add typed live setters for shade and rim RGB colors through the same source
update path. Reuse component color validation. Choose RGB sliders or a reusable
color control at implementation time; a new color-picker widget is not a
phase-1 dependency.

| Color | Preset RGB | Proposed channel range / step |
| --- | --- | --- |
| Shade | 0.40, 0.40, 0.65 | 0–1 / 0.01 |
| Rim | 1.0, 1.0, 1.0 | 0–1 / 0.01 |

Extend retained settings, effective-value synchronization, Reset, restore,
instance-isolation tests, and cache checks to these fields.

## Phase 3: design Shading / ShadingComponent

The requested public shape is `Shading` in MMS and `ShadingComponent` in Rust.
It selects a shading model, including custom models, and owns or references
the typed input state applied to its renderables. Reconcile this name with the
illustrative `Material` API in [Materials v2](epic/materials-v2.md) before
implementation; do not introduce two competing public authoring APIs.

### Input terminology

Descriptors and push constants are likely the two Vulkan mechanisms intended
by “Descriptors and positional arguments.” Push constants are named fields
packed at validated byte offsets, not positional function arguments.
Descriptors bind resources such as uniform/storage buffers, images, and
samplers; multiple scalar fields normally live together inside one bound
buffer. Vertex inputs and specialization constants are additional mechanisms
with different purposes, not ordinary editable material values.

Keep the MMS parameter vocabulary separate from GPU binding layout. Authors
should update named typed fields; the validated shader schema determines their
storage and packing. A schema authored in MMS must match the shader interface:
declaring a field cannot add a corresponding input to an already compiled shader.

### Boring starting design to discuss

- An immutable, registered shading definition contains program identity,
  compatible vertex interface, render state, and a typed parameter schema.
  Validate and resolve names, bindings, offsets, alignment, and defaults once
  when registering/loading it.
- A ShadingComponent references that definition and holds mutable parameter
  values with stable instance identity and dirty tracking. Decide explicitly
  how multiple renderables share that instance and how independent copies are
  requested; preserve authored-source ownership across GLTF projections.
- Start with a packed material uniform block for floats/vectors/colors.
  Descriptors bind that block; sliders update its contents. Add image/sampler
  inputs and push-constant fields only when concrete use cases justify them.
- Prefer automatic static/deformed vertex selection for a compatible fragment
  program, following Materials v2. Define what “custom” initially permits:
  registered programs or user-supplied shader files. The latter additionally
  needs compilation/loading, interface validation, and useful errors.
- Allow MMS schema construction at definition time if needed, but resolve it
  to engine metadata once. Avoid interpreting schemas or rebuilding layouts
  for every setter or frame. Consider shader reflection as validation rather
  than making arbitrary reflection output the public MMS API.

### Lean update requirements

- Ordinary value edits must not change pipeline identity, rebuild geometry,
  or compile shaders. Switching shading models can require pipeline selection
  or creation and must have a separate cost/lifecycle contract.
- Resolve field names to stable slots; coalesce dirty writes per material per
  frame. Upload only changed data through renderer-owned storage safe for
  frames in flight. Do not allocate persistent descriptor/UBO entries for every
  historical value combination.
- Share immutable schemas/programs and deliberately shared material state.
  Define batch keys using stable identities and relevant render state; measure
  the batching consequences of independent material instances. Per-renderable
  values may need instance/storage data if separate UBOs would split batches.
- Benchmark one shared material across many primitives and many independently
  changing materials. Record CPU update/upload time, GPU time, allocations,
  descriptor writes, cache size, and draw count. Set numerical budgets after
  collecting a baseline; “lean” needs measured acceptance criteria.

### Design-session deliverable

Agree on definition versus instance ownership; MMS schema and setter syntax;
initial field types and storage; shader/interface validation; model switching;
serialization/reload behavior; and a small measured prototype. Start with one
custom compatible shading program and one live scalar. Plan animation support
with [animated shader inputs](animated-shader-material-inputs-mms-animation-system.md)
as a separate integration step. Do not expand phase 1 into this architecture.

This document plans the work only. Runnable examples should not reference
unimplemented shader-update APIs until those implementations land.
