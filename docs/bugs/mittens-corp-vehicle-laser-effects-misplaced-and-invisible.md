# `mittens-corp` vehicle laser flash is invisible and the beam is misplaced

Status: open / needs runtime visual diagnosis

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

The beam is made from unit squares scaled by `laser_half_length`, rotated about
X, while the beam root is translated by `-laser_half_length` on local Z. That
placement assumes the rotated square spans two half-lengths from the muzzle and
that the car's bounds-derived local `-Z` is its visible forward direction.

The scripting regression test only checks emitted transform intents and the
bounds-derived arithmetic. It does not render the textures, validate alpha or
facing, measure the beam's final world-space endpoints, or establish the car
model's semantic forward axis. It can therefore pass while the effect is
invisible or visually detached.

## Likely failure areas

- The flash quads may be edge-on, back-face culled, behind the car, too small,
  or missing the transparent-material/cutout configuration required by the PNG
  alpha channel.
- Zero-scale-at-rest transforms may interact poorly with transition capture or
  renderable bounds/culling when first made visible.
- The imported car's visual front may not be `model_box.min.z`; an axis-aligned
  model bound is not a semantic muzzle marker.
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
   and at `car_laser_origin`; confirm which direction is the car's front.
2. Render each flash continuously at a large scale, without emissive or
   animation, and verify texture loading, alpha blending/cutout, UV orientation,
   face culling, and local facing from the expected camera positions.
3. Replace zero-scale hiding temporarily with opacity or a small offscreen
   placement to determine whether first-frame bounds/culling is involved.
4. Measure the beam's final world-space near and far endpoints after all parent
   transforms. Assert that the near endpoint equals the muzzle position and the
   far endpoint is five times the original distance away.
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

