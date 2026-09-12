# Task: live microphone mouth calibration for `mittens-corp`

Status: proposed

## Goal

Create `examples/mittens-corporation-microphone-calibration.mms`, a focused
live microphone calibration scene with an info panel and exactly two primary
calibration sliders:

1. **Full-open loudness** — the average RMS loudness that should produce a fully
   open mouth.
2. **Gradient width** — how broad the quieter-input ramp is below that loudness,
   controlling whether mouth response is sharp or gradual.

Do not expose the current implementation vocabulary of “floor” and “ceiling”
to the user. Do not label the controls “start” and “end.” The panel should speak
in terms of the result a performer is tuning.

## Calibration model

Retain a monotonic mouth response: louder-than-target audio must remain fully
open rather than closing again. Define:

```text
full_open_rms = user-selected loudness for mouth_open = 1
gradient_width_rms = size of the ramp below full_open_rms

ramp_start_rms = max(0, full_open_rms - gradient_width_rms)
mouth_open = smoothstep(ramp_start_rms, full_open_rms, measured_rms)
```

This is the user-facing reparameterization of the current AVC range:

```text
mouth_open_rms_ceiling = full_open_rms
mouth_open_rms_floor = max(0, full_open_rms - gradient_width_rms)
```

Use a small positive epsilon when the component requires `floor < ceiling`.
Clamp `gradient_width_rms` to a useful finite range and never permit invalid or
negative RMS settings.

The existing temporal `mouth_open_smoothing` remains enabled with a sensible
fixed value. It is not a third primary calibration parameter in this task;
gradient width controls amplitude response, while smoothing controls response
over time and should not be conflated with it.

## Example scene

Build `examples/mittens-corporation-microphone-calibration.mms` from the
microphone/avatar parts of the existing corporation and microphone-speaking
examples. It should include:

- `AudioInput` using the default device, with the existing selectable device
  list if it can be reused without obscuring the calibration workflow;
- `Amplitude.rolling_window(...)` feeding AVC mouth-open fallback;
- the corporation avatar and its `viseme_aa` mapping;
- `environment_anime_shading()` for walls, vehicles, and other broad scene
  surfaces while retaining the Bisket-specific preset on the avatar;
- an `info_panel` placed so it is usable in both desktop and XR inspection;
- the two live sliders and numeric readouts;
- a live RMS meter/readout and a mouth-open output meter so the performer can
  see why a setting behaves as it does; and
- concise instructions: speak normally, set full-open loudness near the upper
  end of ordinary speech, then widen/narrow the gradient for the desired
  response.

The meters are diagnostic information, not additional calibration controls.

## Runtime control surface

The current builders `mouth_open_rms_floor(...)` and
`mouth_open_rms_ceiling(...)` are validated construction-time setters. The
panel needs live component methods or intents that update both derived values
atomically. Avoid independently setting floor and ceiling from two slider
callbacks because an intermediate update can violate their ordering.

Preferred API:

```text
avatar_control.set_mouth_open_calibration(full_open_rms, gradient_width_rms)
avatar_control.get_mouth_open_full_open_rms()
avatar_control.get_mouth_open_gradient_width_rms()
```

An equivalent dedicated calibration component is acceptable if it keeps one
authoritative state and feeds AVC. The scripting API should expose the same
performer-facing concepts as the panel. Serialization may continue storing the
derived floor/ceiling for compatibility, but new authoring should prefer:

```text
mouth_open_calibration(full_open_rms, gradient_width_rms)
```

Define precedence and reject ambiguous scenes that mix the new builder with
explicit legacy floor/ceiling calls, or deterministically apply the final call
while documenting that rule.

## Panel behavior

- Slider updates apply immediately without rebuilding AVC or the avatar.
- Show stable decimal RMS values; use a nonlinear/log-friendly slider mapping
  if linear control packs the useful microphone range into too few pixels.
- The full-open slider changing downward must clamp width if necessary rather
  than producing a negative ramp start.
- The width slider should range from near-step response to a broad ramp whose
  start reaches silence.
- Readouts, slider thumbs, derived ramp start, RMS meter, and output meter stay
  synchronized after every update and panel restore.
- Include a reset action using documented defaults.
- Device loss or absent microphone data should show a status message and zero
  the meter without corrupting calibration values.

## Reusable UI

Factor the calibration content into an asset such as:

```text
assets/components/ui/microphone_mouth_calibration_controls.mms
```

The factory should accept the target AVC and amplitude components rather than
querying global names, allowing multiple independent avatars/panels. Reuse
`assets/components/ui/info_panel.mms` for the shell.

## Tests

- Conversion from `(full_open, width)` produces the expected derived range,
  including width greater than full-open and near-zero values.
- The live setter is atomic and cannot leave `floor >= ceiling`.
- RMS below the ramp start maps to closed, at full-open maps to one, and louder
  RMS remains one.
- Increasing gradient width makes an intermediate quiet sample produce a
  larger mouth value without moving the full-open point.
- Both sliders update AVC and all readouts/meters in a strict MMS evaluation
  test.
- The example contains exactly one amplitude source for the calibrated AVC and
  retains its mapped mouth morph.
- Panel restoration does not duplicate handlers or reset live calibration.

## Acceptance criteria

- A performer can tune natural mouth response while speaking, without knowing
  what floor and ceiling mean.
- Only full-open loudness and gradient width are presented as calibration
  parameters.
- Changes are visible immediately in both the meter and avatar mouth.
- The example works with the default microphone and supports clear device/error
  status.
- The reusable controls can calibrate another AVC instance without editing the
  asset factory.

## Related files

- `examples/mittens-corp.mms`
- `examples/vtuber-microphone-speaking-xr-eye-tracking.mms`
- `assets/components/ui/info_panel.mms`
- `assets/components/materials/environment_anime_shading.mms`
- `src/engine/ecs/component/avatar_control.rs`
- `src/engine/ecs/system/avatar_control_system.rs`
- `src/engine/ecs/component/amplitude.rs`
- `src/scripting/component_registry.rs`
- `src/scripting/component_method_registry.rs`
- `src/scripting/tests.rs`
