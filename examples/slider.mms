// Native SliderComponent examples: default, stepped, and authored visuals.
//
// Run with:
//   cargo run --release -- load examples/slider.mms

RendererSettings { window_size(1100, 720) }
BGC.rgba(0.05, 0.05, 0.05, 1.0)
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

let PANEL_WIDTH = 11.4
let ROW_HEIGHT = 1.15
let LABEL_WIDTH = 3.8
let SLIDER_CELL_WIDTH = 5.8
let READOUT_WIDTH = 1.2
let CONTROL_FONT_SIZE = 0.18

fn label(text) {
    return T {
        Style {
            display("flex")
            width(LABEL_WIDTH)
            height(ROW_HEIGHT)
            align_items("center")
            padding_xy(0.35, 0.15)
            font_size(CONTROL_FONT_SIZE)
        }
        T.position(0.0, 0.0, 0.03) { Text { text } }
    }
}

fn readout(initial) {
    return T {
        Style {
            display("flex")
            width(READOUT_WIDTH)
            height(ROW_HEIGHT)
            align_items("center")
            justify_content("center")
            font_size(CONTROL_FONT_SIZE)
        }
        T.position(0.0, 0.0, 0.03) { Text { initial } }
    }
}

fn slider_cell(slider) {
    return T {
        Style {
            display("flex")
            width(SLIDER_CELL_WIDTH)
            height(ROW_HEIGHT)
            align_items("center")
            justify_content("center")
        }
        slider
    }
}

fn slider_row(label_text, slider, value_readout) {
    return T {
        Style {
            display("flex")
            flex_direction("row")
            width(100%)
            height(ROW_HEIGHT)
            align_items("center")
            gap(0.2)
            padding_xy(0.15, 0.0)
            background_color([0.09, 0.10, 0.13, 0.92])
            background_z(-0.03)
        }
        label(label_text)
        slider_cell(slider)
        value_readout
    }
}

let default_slider = Slider.range(0.0, 1.0).value(0.25).width(5.0)
let default_value = readout("0.25")

let stepped_slider = Slider.range(-1.0, 1.0).step(0.25).value(0.0).width(5.0)
let stepped_value = readout("0")

// This let-bound component is a live subtree reference. Slider.track reparents
// the same tree beneath its stable engine-owned track mount.
let live_track = track_visual([0.22, 0.62, 0.92, 1.0])
let themed_slider = Slider.range(0.0, 100.0).step(5.0).value(65.0).width(5.0)
    .track(live_track)
    .thumb(thumb_visual([1.0, 0.35, 0.62, 1.0]))
let themed_value = readout("65")

// A second themed instance receives fresh visual trees; mounted live trees have
// one owner and are moved rather than cloned.
let second_themed_slider = Slider.range(0.0, 10.0).step(1.0).value(3.0).width(5.0)
    .track(track_visual([0.42, 0.30, 0.72, 1.0]))
    .thumb(thumb_visual([0.45, 1.0, 0.55, 1.0]))
let second_themed_value = readout("3")

let reset = T {
    name = "reset_sliders"
    Raycastable.enabled()
    Style {
        display("inline-block")
        padding_xy(0.55, 0.25)
        font_size(CONTROL_FONT_SIZE)
        text_align("center")
        vertical_align("middle")
        color([1.0, 1.0, 1.0, 1.0])
        background_color([0.82, 0.28, 0.34, 1.0])
        background_z(-0.02)
    }
    Text { "Reset values" }
}

T.position(-5.7, 3.0, 0.0) {
    LayoutRoot {
        available_width(PANEL_WIDTH)
        available_height(7.5)
        unit_scale(1.0)

        T {
            Style {
                display("flex")
                flex_direction("column")
                width(100%)
                row_gap(0.25)
            }

            slider_row("Default / continuous", default_slider, default_value)
            slider_row("Stepped by 0.25", stepped_slider, stepped_value)
            slider_row("Authored track + thumb", themed_slider, themed_value)
            slider_row("Independent theme instance", second_themed_slider, second_themed_value)

            T {
                Style {
                    display("flex")
                    width(100%)
                    height(0.9)
                    align_items("center")
                    justify_content("center")
                }
                reset
            }
        }
    }
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
