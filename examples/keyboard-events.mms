// Global gameplay keyboard events. No Input/I component is required.
RendererSettings { window_size(760, 360) }
BGC.rgba(0.80, 0.80, 0.80, 1.0) {}

T.position(0.0, 0.0, 2.5) {
    C3D {
        Pointer {}
    }
}

let status = Text {
    "Keyboard events\nPress W, Shift, or an arrow key\nWaiting for input…"
    C.rgba(0.22, 0.23, 0.25, 1.0)
}
T.position(-2, 1, 0.0).scale(0.14, 0.14, 0.14) {
    Draggable {}
    status
}

let keyboard = { w_held = false }

fn show_keyboard_event(signal_name, event) {
    let code = "null"
    if event.code != null {
        code = event.code
    }
    status.set_text(signal_name + "\ncode: " + code + "    key: " + event.key + "\nW held: " + keyboard.w_held)
}

on_global("KeyDown", fn(event) {
    if event.code == "KeyW" {
        keyboard.w_held = true
    }
    show_keyboard_event("KeyDown", event)
})

on_global("KeyPress", fn(event) {
    show_keyboard_event("KeyPress", event)
})

on_global("KeyUp", fn(event) {
    if event.code == "KeyW" {
        keyboard.w_held = false
    }
    show_keyboard_event("KeyUp", event)
})
