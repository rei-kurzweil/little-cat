# `mittens-corp` vehicle laser flash is invisible and the beam is misplaced

Status: partially fixed in source / needs desktop and XR visual acceptance

## Confirmed findings (2026-09-11)

- `R.square()` is a unit quad in the local XY plane, with a local `+Z`
  normal. The beam's `-90°` X rotation maps its scaled local `+Y` direction
  onto local `-Z`.
- Consequently a Y scale of `40.0` produces a **40-unit total beam length**,
  centered on its parent. The former `-40.0` parent translation put its
  endpoints at local Z `-20` and `-60`, leaving a 20-unit muzzle gap. The
  parent now sits at `-20.0`, producing endpoints at `0` and `-40` relative
  to the shared muzzle frame.
- The flash parent is now rotated 180° about Y so the square's `+Z` normal
  faces the firing direction (`-Z`). The active keyframe retains that
  rotation. Renderer defaults currently do not explicitly enable back-face
  culling, but matching face/normal orientation remains correct for lighting
  and makes the effect robust if culling is enabled later.
- The flash PNGs are RGBA. They now use `Opacity.opacity(0.99)` to enter the
  alpha-blended pass; without an opacity/color-alpha hint, the renderer routes
  them through its opaque pass because it does not inspect texture alpha when
  building draw lists.
- The imported car's measured aggregate `local_bounds()` are
  `min = [-4.3999033, 0.04635185, -1.7789365]` and
  `max = [4.3999033, 5.566735, 2.745441]`. The present AABB-derived placement
  therefore evaluates to approximately `[0.0, 3.137, -1.879]`. This is
  telemetry for all renderable car geometry, not evidence of a real barrel or
  muzzle point.

The current script retains the `min.z` placement only as a calibrated interim
assumption: the existing cockpit, entry zone, and initial driving convention
all use local `-Z` as forward. It must be replaced with an authored muzzle
marker or verified car-local offset once the model is inspected visually.

## Observed behavior

In `examples/mittens-corp.mms`, firing the mounted vehicle laser does not show
either muzzle-flash image. The beam appears to begin well in front of the
vehicle instead of at the muzzle and, despite its authored five-fold length
increase, does not look as though it extends very far.

The regression was observed after replacing the procedural sphere flash with
`assets/images/flash_red_0.png` and `assets/images/flash_red_1.png`, sequencing
the two images, and changing `laser_half_length` from `8.0` to `40.0`.

## Current implementation

The muzzle origin is placed once the imported car has renderable bounds:

```text
front_z = model_box.min.z - muzzle_clearance
```

Both flashes are textured `R.square()` children whose transforms start at zero
scale. The shot animation shows `_0` at `0.00s`, replaces it with `_1` at
`0.05s`, then hides `_1` and the beam at `0.10s`.

The beam is made from unit squares whose Y scale is `laser_length`, rotated
about X, while the beam root is translated by `-laser_length * 0.5` on local Z.
This centers the 40-unit quad-derived beam over the muzzle-to-tip segment. The
car's bounds-derived local `-Z` is still an interim forward-direction
assumption, not a semantic muzzle definition.

The scripting regression test only checks emitted transform intents and the
bounds-derived arithmetic. It does not render the textures, validate alpha or
facing, measure the beam's final world-space endpoints, or establish the car
model's semantic forward axis. It can therefore pass while the effect is
invisible or visually detached.

## Likely failure areas

- The flash quads may still be behind the car, too small, or affected by
  zero-scale-at-rest bounds/culling when first made visible.
- The shared flash parent now faces local `-Z`, and the images enter the
  alpha-blended pass. Confirm their appearance from the firing-side camera and
  from oblique angles rather than assuming a successful script test proves it.
- The imported car's visual front may not be `model_box.min.z`; an axis-aligned
  model bound is not a semantic muzzle marker.
- `Transform.local_bounds()` does union descendant renderable bounds in the
  queried transform's local frame, including nested transforms, and deliberately
  returns `null` until the GLTF and cached renderable bounds are ready. The scene
  is using that API as designed, but the resulting AABB describes all car
  geometry, not the cannon barrel or a semantic front point. A correct aggregate
  box can therefore still produce an incorrect muzzle location.
- The beam's square primitive axis, post-rotation axis, root translation, and
  scale convention may not produce the assumed muzzle-to-tip segment.
- A very long transparent quad may be clipped, culled, depth-sorted poorly, or
  mostly embedded behind the camera/vehicle because its midpoint is wrong.

## Desired behavior

- `flash_red_0.png` is clearly visible at the vehicle muzzle first.
- `flash_red_1.png` replaces it, then both flashes are fully hidden.
- The beam begins at the exact same muzzle point with no visible gap.
- The beam extends five times farther than the original visible beam in the
  vehicle's firing direction.
- The flash and beam replay consistently on every valid trigger press.

## Investigation plan

1. Add temporary visible axis markers at the measured bounds minimum/maximum Z
   and at `car_laser_origin`; print the returned `local_bounds()` values and
   confirm which direction is the car's front.
2. Render each flash continuously at a large scale, without emissive or
   animation, and verify texture loading, alpha blending/cutout, UV orientation,
   face culling, and local facing from the expected camera positions.
3. Replace zero-scale hiding temporarily with opacity or a small offscreen
   placement to determine whether first-frame bounds/culling is involved.
4. Measure the beam's final world-space near and far endpoints after all parent
   transforms. The local geometry regression now establishes `[0, -40]` on
   local Z; extend it or add a render test to establish the equivalent
   world-space result under translated and rotated cars.
5. Prefer an authored muzzle marker in the car asset or a calibrated
   car-local muzzle offset over an AABB extremum once the correct position and
   direction are known.
6. Test from desktop and XR viewpoints, including oblique angles. If one flat
   ribbon disappears, use crossed ribbons or a camera-facing beam presentation.
7. Add a render/screenshot acceptance test or a geometry-level endpoint test;
   do not rely solely on animation intent presence.

## Acceptance criteria

- A captured firing sequence visibly contains frame 0, then frame 1, then no
  flash, in that order.
- Neither flash is visible at rest.
- Beam near-end and muzzle world positions agree within a small tolerance.
- Beam world-space length is demonstrably five times the pre-change length.
- No part of the effect appears detached when the car is translated or rotated.

## Related files

- `examples/mittens-corp.mms`
- `assets/images/flash_red_0.png`
- `assets/images/flash_red_1.png`
- `src/scripting/tests.rs`
