# Native Slider / SliderComponent and standalone example

Status: implemented first slice.

Implement `SliderComponent` in the mittens engine, expose it as `Slider` in
MMS, and create `examples/slider.mms` to demonstrate it independently of any
material or model. The [anime shading panel task](anime-shading-panel-and-live-shader-inputs.md)
will consume this control after its interaction and mounting contracts work.

## Native control, authored visuals

Add a native `SliderComponent` primitive exposed as `Slider`. The engine owns the numeric value, range,
step, pointer capture, focus, keyboard input, and thumb positioning. MMS
owns graphics and composition: a reusable slider input wrapper adds the
label, value readout, and theme around the native control.

Expose two stable engine-owned mounts: `track` and `thumb` (the moving head).
Each accepts an arbitrary component subtree through a builder method. Accept
a component tree value or a resolved live component reference, using the
ordinary component attachment machinery. These are real children, so theme
authors can mount geometry, text, or composite graphics without implementing
drag behavior themselves.

Proposed API sketch, **not currently executable MMS**; names are provisional:

```mms
Slider.range(0.0, 1.0).step(0.01).value(0.5)
    .track(track_visuals)
    .thumb(thumb_visuals)
```

The builder attaches the supplied root beneath the corresponding mount.
For a live reference, it moves the same subtree rather than silently cloning
it. Reject cycles, stale references, and cross-world references. A theme
factory must produce fresh trees for each slider; one live tree cannot belong
to several controls. Replacing a part detaches its previous content using
normal lifecycle rules and must not leave orphaned renderables or handlers.
Keep the engine mount itself stable across replacements.

Expose resolved track/thumb mount references for subsequent ordinary subtree
attachment. Avoid requiring callers to query private generated node names.
Do not introduce a separate slider-only tree representation or mounting system.

The track mount spans the usable track rectangle. The thumb mount uses a
centered origin at the current value; movement belongs to the engine mount,
preserving transforms inside the supplied visual tree. Slider layout must
reserve the thumb extent at both ends so its center travels inside the control.
Visual children must not feed thumb translation back into intrinsic sizing.
Use explicit control dimensions for the first version; custom graphics may
overflow, but do not redefine the numeric range or hit area.

Provide a basic default track and thumb when visuals are omitted. Theme
geometry routes interaction to its owning slider; it must not need its own
drag handler or prevent clicks reaching the control. Expose hover, focus,
dragging, and disabled state for wrappers to style later.

## Value and interaction contract

- First slice: horizontal, one thumb, finite `min < max`, optional positive
  step. Omitted step means continuous input; invalid configuration is an error.
- Clamp values to the range and snap to steps anchored at `min`, with both
  endpoints reachable. Apply the same normalization to initial values,
  pointer input, keyboard input, and programmatic updates.
- Track click moves to the clicked value. Thumb drag preserves its initial
  grab offset. Capture continues outside the control; release ends the gesture.
- Compute value from the laid-out local track axis, including parent transforms
  and panel unit scale. Desktop and XR pointers share this mapping.
- Focused arrows move by one step (1% of the range for continuous controls);
  Home/End select endpoints. Disabled controls ignore input.
- Proposed `ValueChanged` event carries the normalized numeric value on an
  actual change, allowing live material updates. `ValueCommitted` marks a
  completed user edit. Programmatic synchronization can be silent to avoid
  feedback loops. Final event names/payload conventions should follow existing
  engine controls during implementation.
- Removal, panel minimization, or loss of pointer capture ends interaction and
  releases focus/capture safely. Cancellation keeps the last applied value
  without emitting a successful commit.

## Standalone example

Create `examples/slider.mms`, runnable through the normal MMS example loader:

```sh
cargo run --release -- load examples/slider.mms
```

Show a default slider, a stepped slider, and a custom-themed slider with
arbitrary component trees mounted on both track and thumb. Include numeric
readouts driven by value events, plus a programmatic reset to demonstrate
silent synchronization. Demonstrate a live-reference mount and separate theme
instances so attachment ownership is visible in a small, inspectable scene.

Use block-stacked flex rows for labels, controls, and readouts. A thin reusable
`assets/components/ui/slider_input.mms` wrapper may own that presentation;
dragging, range normalization, and thumb movement remain native behavior.
Keep this example independent of anime materials and GLTF assets.

## Implementation follow-up and acceptance

1. Implement native slider state, layout, input, mounting, and MMS bindings.
2. Add the thin authored slider input wrapper and verify arbitrary visuals on
   both mounts, including live-reference attachment and independent instances.
3. Create `examples/slider.mms` with default/custom visuals, stepped values,
   numeric readouts, and programmatic updates.
4. Verify endpoints, snapping, keyboard input, pointer capture outside bounds,
   scaled/moved panels, desktop/XR mapping, and cleanup during an active drag.
5. Verify independent slider instances, live-reference ownership, part
   replacement, disabled behavior, and silent updates without feedback loops.
6. Add focused engine tests for normalization and mount lifecycle, and exercise
   the runnable example for layout and input behavior.

The first slice now exists in `SliderComponent`, `SliderSystem`, the MMS
registry, and `examples/slider.mms`. Layout-native sizing, richer exposed
visual states, and replacement of mounted parts after initialization remain
follow-up work.
