# Task: global MMS keyboard events first slice

Status: implemented; manual desktop verification pending, 2026-09-07. Child of
[keyboard and gamepad events](mms-keyboard-and-gamepad-events.md).

Implementation: `UserInput` retains ordered physical/logical keyboard records;
`KeyboardInputSystem` dispatches them before movement and `FrameTick`; MMS exposes
the three global signals with the two-field payload described below. See
[`examples/keyboard-events.mms`](../../examples/keyboard-events.mms).

## Outcome and existing path

Expose `KeyDown`, `KeyUp`, and `KeyPress` through the existing
`on_global("KeyDown", fn(event) { ... })` API. No `Keyboard {}` or `I {}`
component is required. The shorter unscoped `on(...)` overload is deferred.
Regular gamepads remain in the parent ticket.

Today, `Windowing::window_event` feeds winit events to `UserInput`.
`InputState` retains logical-key down/pressed/released sets and ordered text
editing events, but no complete ordered keyboard-event queue. Its gameplay
key normalization lowercases ASCII letters. Preserve that existing behavior;
the new event payload must retain original logical-key meaning separately.
MMS already converts typed engine event fields into maps and then transport
tables in `runner::event_arg_value` and `host::event_arg_transport`.

## Proposed event semantics

Use these explicit engine semantics; `KeyPress` is not a text-input event:

| Signal | When emitted |
| --- | --- |
| `KeyDown` | Once on transition from up to down |
| `KeyPress` | After initial `KeyDown`, then for each OS repeat while held |
| `KeyUp` | Once on transition from down to up, including cleanup releases |

Initial press emits `KeyDown`, then `KeyPress`. Repeats emit only `KeyPress`.
Release emits only `KeyUp`. Repeat detection stays internal; no repeat flag
is included in the MMS payload for this slice.
All keys, including arrows and modifiers, can produce an initial `KeyPress`;
repeat availability follows the OS. This proposal must be documented with the
API because the name has different meanings in other event systems. Movement
uses down/up held state plus frame time, not `KeyPress` repeat frequency.
Text editing and IME composition continue through the text-input path.

## Typed engine payload, MMS table

Define a typed Rust `KeyboardEvent` payload shared by the three signal variants.
Expose exactly two fields to MMS; illustrative values for Shift+W:

```text
{
    code: "KeyW",
    key: "W"
}
```

- `code`: stable named physical key (`KeyW`, `ArrowUp`, `ShiftLeft`), independent
  of layout. Use null for an unidentified physical code in this first slice.
- `key`: logical/layout-dependent character string or documented named key
  (`Enter`, `ArrowUp`, `Dead`, `Unidentified`). Preserve case and Unicode;
  it need not be one character. It is not guaranteed committed text.

Defer `location`, `modifiers`, `repeat`, and `synthetic` fields until MMS has a
faster runtime and richer payloads are reconsidered. Keep table construction
and transport lean: no nested modifier table or extra metadata fields in this
slice. Internal input records may retain information needed for correct repeat
and focus handling; those fields are not serialized into the MMS event table.

Use explicit mappings rather than Rust debug formatting or enum ordinals as a
public API. A native numeric scan code is platform-specific and unnecessary
for the first slice. The useful pair is physical code string plus logical key
string. Tables allow future optional fields without positional arguments or a
breaking callback signature change. Keep internal state identity
for unidentified native keys even when the public `code` is null.

## Dispatch and focus

Add a `KeyboardInputSystem` / `keyboard_input_system` that consumes normalized,
ordered records captured by `UserInput`, then emits typed ECS event signals.
Keep winit/platform decoding at the existing input boundary; the ECS system
does not need its own window listener. Preserve down/up/down and repeats between
redraws; reconstructing events from the existing sets loses order and detail.
Capture the logical key at event receipt, preserving case/layout meaning rather
than recomputing it at callback delivery. No modifier snapshot API is required.
Consume each record once before end-of-frame clearing, with callbacks delivered
in order before scripted movement's frame update. Verify the actual signal
queue drain order rather than assuming enqueue time is callback time.

For this first slice, global means scene-wide gameplay subscription. Suppress
new presses/repeats while editor text input owns the keyboard. On capture or
window-focus loss, synthesize `KeyUp` for previously delivered down keys and
clear held state; a later physical release must not emit a duplicate. Require
a fresh physical press after focus returns; do not activate gameplay from
synthetic focus-restoration presses or already-held repeats. Preserve legitimate
releases across modifier/layout changes by tracking physical identity.
Scene teardown clears state/subscriptions; no callbacks enter a disposed scene.

Windowing currently exits immediately on Escape press. Keep that shortcut
unchanged in this slice and document that Escape delivery before exit is not
guaranteed. Use another key for the demonstration.

## Delivery and verification

Register the three kinds in the typed signal enums, both MMS signal-name
resolvers/catalogs, and shared payload conversion. Add a small MMS example
that displays the signal name, `code`, and `key` and tracks a held movement key
without `I {}`. No velocity implementation is required to verify dispatch.

Focused tests must cover ordered press/repeat/release, several transitions
within one frame, logical/physical identity differences (including Shift),
exactly the two table fields in global callbacks, focus/capture cleanup, and
scene reload.
Manually verify character and arrow keys, Shift, repeat, text focus, and
window blur. Existing automatic input and text editing must still work.
