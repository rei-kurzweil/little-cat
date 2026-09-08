// Content for info_panel: edits one retained Shading.anime() source.
// The caller owns panel chrome, restore, and the original Reset values.
fn fixed_2(value) {
    let scaled = Math.round(value * 100.0)
    let whole = Math.floor(scaled / 100.0)
    let fraction = scaled - whole * 100.0
    let padding = ""
    if fraction < 10.0 { padding = "0" }
    return "" + whole + "." + padding + fraction
}

export fn anime_shading_controls(target, reset_values) {
    let initial = target.get_shade_strength()
    let track = T.scale(8.0, 0.10, 0.10) { R.cube() { C.rgba(0.24, 0.45, 0.65, 1.0) Raycastable.enabled() { interaction_priority(120.0) } } }
    let thumb = T.scale(0.45, 0.90, 0.30) { R.sphere() { C.rgba(1.0, 0.46, 0.64, 1.0) Raycastable.enabled() { interaction_priority(120.0) } } }
    let slider = Slider.range(0.0, 1.0).step(0.01).value(initial).width(16.0)
        .track(track)
        .thumb(thumb) {
            name = "anime_shade_strength_slider"
        }
    let readout = Text { name = "anime_shade_strength_readout" fixed_2(initial) }
    let reset = T {
        name = "anime_shading_reset"
        Raycastable.enabled() { interaction_priority(120.0) }
        Style {
            display("flex") width(8.0) height(2.0)
            align_items("center") justify_content("center")
            background_color([0.35, 0.20, 0.30, 1.0]) background_z(-0.02)
        }
        T.position(0.0, 0.0, 0.03) { Text { "Reset" } }
    }
    let content = T {
        name = "anime_shading_controls"
        Style {
            display("block") width(100%) padding(0.65)
            font_size(0.8) color([0.96, 0.96, 0.98, 1.0])
            background_color([0.075, 0.075, 0.09, 0.96]) background_z(-0.01)
        }
        T {
            Style {
                display("flex") flex_direction("row") width(100%) height(3.0)
                align_items("center") gap(0.5)
            }
            T {
                Style { display("flex") width(11.0) height(3.0) align_items("center") }
                T.position(0.0, 0.0, 0.03) { Text { "Shade strength" } }
            }
            T {
                Style {
                    display("flex") width(18.0) height(3.0) flex_grow(1.0)
                    align_items("center") justify_content("center")
                }
                slider
            }
            T {
                Style {
                    display("flex") width(4.0) height(3.0)
                    align_items("center") justify_content("center")
                }
                T.position(0.0, 0.0, 0.03) { readout }
            }
        }
        T {
            Style {
                display("flex") width(100%) height(2.5)
                align_items("center") justify_content("flex-end")
            }
            reset
        }
    }
    // Resolve live controls from the materialized subtree before capturing them
    // in callbacks; local component expressions are otherwise reusable templates.
    let live_slider = content.query("#anime_shade_strength_slider")
    let live_readout = content.query("#anime_shade_strength_readout")
    let live_reset = content.query("#anime_shading_reset")
    on(live_slider, "SliderChanged", fn(event) {
        target.set_shade_strength(event["value"])
        let effective = target.get_shade_strength()
        live_slider.sync_value(effective)
        live_readout.set_text(fixed_2(effective))
    })
    on(live_reset, "Click", fn(event) {
        target.set_shade_strength(reset_values.shade_strength)
        let effective = target.get_shade_strength()
        live_slider.sync_value(effective)
        live_readout.set_text(fixed_2(effective))
    })
    return content
}
