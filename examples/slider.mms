// Native SliderComponent examples: default, stepped, and authored visuals.
//
// Run with:
//   cargo run --release -- load examples/slider.mms

RendererSettings { window_size(1100, 720) }
BGC.rgba(0.035, 0.04, 0.06, 1.0)
AL.rgb(0.45, 0.45, 0.48)

I.speed(1.5) {
    InputTransformMode.forward_z() { fps_rotation() roll_axis_y() }
    T.position(0.0, 0.4, 8.5) { C3D { Pointer {} } }
}

fn track_visual(color) {
    return T.scale(2.5, 0.055, 0.055) {
        R.cube() { C.rgba(color[0], color[1], color[2], color[3]) }
    }
}

fn thumb_visual(color) {
    return T.scale(0.19, 0.30, 0.11) {
        R.sphere() { C.rgba(color[0], color[1], color[2], color[3]) }
    }
}

fn label(text) {
    return T.position(-4.1, 0.10, 0.0).scale(0.055, 0.055, 1.0) {
        Text { text }
    }
}

fn readout(initial) {
    return T.position(3.1, 0.10, 0.0).scale(0.055, 0.055, 1.0) {
        Text { initial }
    }
}

let default_slider = Slider.range(0.0, 1.0).value(0.25).width(5.0)
let default_value = readout("0.25")
T.position(0.0, 1.8, 0.0) {
    label("Default / continuous")
    default_slider
    default_value
}

let stepped_slider = Slider.range(-1.0, 1.0).step(0.25).value(0.0).width(5.0)
let stepped_value = readout("0")
T.position(0.0, 0.4, 0.0) {
    label("Stepped by 0.25")
    stepped_slider
    stepped_value
}

// This let-bound component is a live subtree reference. Slider.track reparents
// the same tree beneath its stable engine-owned track mount.
let live_track = track_visual([0.22, 0.62, 0.92, 1.0])
let themed_slider = Slider.range(0.0, 100.0).step(5.0).value(65.0).width(5.0)
    .track(live_track)
    .thumb(thumb_visual([1.0, 0.35, 0.62, 1.0]))
let themed_value = readout("65")
T.position(0.0, -1.0, 0.0) {
    label("Authored track + thumb")
    themed_slider
    themed_value
}

// A second themed instance receives fresh visual trees; mounted live trees have
// one owner and are moved rather than cloned.
let second_themed_slider = Slider.range(0.0, 10.0).step(1.0).value(3.0).width(5.0)
    .track(track_visual([0.42, 0.30, 0.72, 1.0]))
    .thumb(thumb_visual([0.45, 1.0, 0.55, 1.0]))
let second_themed_value = readout("3")
T.position(0.0, -2.4, 0.0) {
    label("Independent theme instance")
    second_themed_slider
    second_themed_value
}

on(default_slider, "SliderChanged", fn(event) {
    default_value.query("Text").set_text("" + event["value"])
})
on(stepped_slider, "SliderChanged", fn(event) {
    stepped_value.query("Text").set_text("" + event["value"])
})
on(themed_slider, "SliderChanged", fn(event) {
    themed_value.query("Text").set_text("" + event["value"])
})
on(second_themed_slider, "SliderChanged", fn(event) {
    second_themed_value.query("Text").set_text("" + event["value"])
})

let reset = T.position(0.0, -3.65, 0.0).scale(0.75, 0.28, 0.12) {
    R.cube() {
        C.rgba(0.82, 0.28, 0.34, 1.0)
        Raycastable.click_only()
    }
    T.position(-0.42, 0.13, 0.7).scale(0.10, 0.10, 1.0) { Text { "Reset" } }
}

on(reset, "Click", fn(event) {
    // sync_value is the programmatic, non-emitting synchronization path.
    default_slider.sync_value(0.25)
    stepped_slider.sync_value(0.0)
    themed_slider.sync_value(65.0)
    second_themed_slider.sync_value(3.0)
    default_value.query("Text").set_text("0.25")
    stepped_value.query("Text").set_text("0")
    themed_value.query("Text").set_text("65")
    second_themed_value.query("Text").set_text("3")
})
