# Focus and keyboard

Every Mok control that takes a press is a Tab stop. Focus moves with Tab and Shift-Tab, shows as a 1px ring, and comes back where it was when an overlay closes.

## What `init` binds

`init` binds `tab` to the `FocusNext` action and `shift-tab` to `FocusPrev`, under `MokFocus`, the key context every `FocusScope` sets. A `FocusScope` turns the two actions into focus moves. A context nested deeper keeps its own Tab, as a code editor that indents does, whichever library's `init` ran last.

## The root scope

Wrap each window's content in `FocusScope::new(&handle).root()` and focus `handle` when the window opens.

| Behavior | Detail |
| --- | --- |
| Tab and Shift-Tab | Move to the next or previous Tab stop in the window. |
| A focused element leaves the tree | The root takes focus, and logs `focus: its element left the tree; the root takes it`. Keys keep reaching the window. |
| `.trap()` | Keeps Tab inside the scope. A trap with no Tab stop inside logs a warning and leaves focus where it is. |

## Make a control a Tab stop

Four parts make a box a Tab stop that shows Mok's ring:

| Part | Why |
| --- | --- |
| `cx.focus_handle().tab_stop(true)` | GPUI tabs only to handles built with `tab_stop(true)`. Keep the handle in the view. |
| `.track_focus(&handle)` | Registers the handle on the box each frame. |
| `.border_1()` with a transparent color | Room for the ring, so the box does not shift when it shows. |
| `.focus_ring(cx)` | From `FocusRing`: the border takes the theme's `focus` color while focused. |

With an `on_click`, GPUI presses the focused box when Enter or Space is released.

```rust example=focus
```

Tab from the window moves through First, the tile and Last. Enter or Space on the tile turns it on and off.

## Buttons

A `Button` stays out of focus when clicked: it stops GPUI's focus on mouse down. Tab still reaches it, unless it is disabled; a loading button stays a Tab stop, so focus holds through the wait. A button whose words and action change with a mode keeps one handle through `Button::focus_handle(&handle)`, built with `tab_stop(true)`, so focus holds across the change.

## Overlays

Mok's dialogs, popovers, menus, palettes, sheets and lightbox take focus when they open and hand it back to what held it before, on every way they close. While open, they take focus back when the focused element leaves the tree. An app writes no focus code for them.
