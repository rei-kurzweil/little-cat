# Anime shading panel and live shader-input updates

Status: design only. No engine implementation in this change.

Depends on [native Slider / SliderComponent and standalone example](native-slider-component-and-example.md).
Use that control to configure the right-hand Bisket in
[shading-models.mms](../../examples/shading-models.mms) through the reusable
[info_panel](../../assets/components/ui/info_panel.mms). Keep the left-hand
model and matching lights as the comparison baseline.

This task includes the engine/MMS update path needed to change anime shader
inputs while the scene is running, as well as the authored panel. Constructor
configuration alone is insufficient: edits must reach the rendered material.

## Panel composition and settings

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
| Shade red / green / blue | 0.40 / 0.40 / 0.65 | 0–1 each | 0.01 |
| Rim red / green / blue | 1.0 / 1.0 / 1.0 | 0–1 each | 0.01 |

These are useful example ranges, not new engine limits: thresholds currently
accept nonnegative values and rim power is clamped to 0.01–128. Keep
`shade_threshold <= lit_threshold`. Match existing builder normalization:
raising shade above lit also raises lit; lowering lit below shade also lowers
shade. Synchronize both affected controls and readouts silently.

Keep settings outside the panel body. Initialize the material and controls
from the same settings, and provide Reset to restore the preset.

## Live shader-input update path

Resolve and retain the authored AnimeShading component beneath the right-hand
GLTF. Slider value changes should update its typed material parameters through
the normal live component mutation API. Inspect existing builder/live method
dispatch first, then implement any missing methods or update notifications.
Keep the API consistent with other component updates; final method names are
an implementation decision, not an assumed existing interface.

The required path is:

1. A slider emits its normalized numeric value; the panel changes the associated
   scalar or RGB channel in its retained settings.
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
outside its scope.

Apply updates during dragging, not only on commit. Do not reload the GLTF,
recompile shaders, recreate geometry, or rebuild panel rows for parameter-only
changes. Preserve the latest value if updates are coalesced within a frame.
Reset must use the same live update path and restore all parameters coherently.

## Panel lifecycle

Minimizing currently removes the accordion body. On
`AccordionRestoreRequested`, build fresh rows from retained settings, wrap
them with the existing `accordion_body` helper, and attach them to the event's
body mount. Reinstall handlers only for the new controls; removed rows must
not retain active subscriptions. Material settings survive minimize/restore.

## Implementation follow-up and acceptance

1. Verify and complete live AnimeShading mutation, normalization, projection
   propagation, and renderer parameter refresh before wiring the full panel.
2. Build the reusable panel content and instantiate it in `shading-models.mms`.
3. Verify every scalar and RGB channel visibly updates the anime instance while
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

This document plans the work only. Runnable examples should not reference
unimplemented slider or shader-update APIs until those implementations land.
