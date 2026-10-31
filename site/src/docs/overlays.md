# Overlays

Dialogs, popovers, menus, tooltips and toasts draw over the page. Dialogs, popovers and menus take focus while open and hand it back when they close; tooltips and toasts leave focus alone.

## Drawing over the page

`primitives::raise(id, child)` draws `child` over the page. At the top of the tree it draws after everything else. Inside something raised, it lays out on its own and draws last there, so a list opened in a dialog lies over the dialog. Mok's overlays raise themselves; an app calls `raise` only for an overlay of its own.

## Dialogs

`Dialog::new(id, title, on_close)` is a card over a scrim. Render it while it is open, and set the open flag false in `on_close`.

| Call | Does |
| --- | --- |
| `.detail(text)` | A line under the title. |
| `.child(..)` | The owner's content. |
| `.action(\|close\| ..)` | A button in the row at the bottom, in the order given. The closure gets the dialog's close; a button that ends the dialog calls it. |
| `.fullscreen()` | Fills the window below its title bar, under a bar with the title and a close button. |

Escape, a press on the scrim and a fullscreen dialog's close button all run `on_close`. Tab stays inside the dialog, and focus returns to what held it before, on every way out. The card fits the window; its body scrolls and its actions stay. `AlertDialog`, `ConfirmDialog` and `PromptDialog` build on `Dialog`.

## Command palette

`CommandPalette::new(id, on_close)` opens a modal quick-search and action launcher over a backdrop scrim.

| Call | Does |
| --- | --- |
| `.placeholder(text)` | Search field placeholder text. |
| `.query(text)` | Sets the active search filter query string. |
| `.group(CommandGroup::new(heading, items))` | Adds a section of grouped items with a category heading. |
| `.on_query_change(handler)` | Callback triggered whenever search text changes. |

Each `CommandItem::new(id, title)` supports `.description(text)`, `.shortcut("⌘D")`, `.icon(LucideIcon::Variant)`, and `.on_select(move |window, cx| ..)`. <kbd>Up</kbd> and <kbd>Down</kbd> navigate items, <kbd>Enter</kbd> runs the selected action, and <kbd>Escape</kbd> closes the palette.

## Popovers

`Popover::new(id, label, content)` is a button that opens a panel under it, or over it when only above has room. A press outside or Escape closes it, and Tab stays inside. `content` runs only while the panel is open. `Popover::with_opener` opens the panel from an element of the app's own.

## Toasts

A window keeps one `Toaster`, an entity, and renders one `ToastViewport::new(id, &toaster)`. Anything that holds the entity pushes to it.

| Call | Does |
| --- | --- |
| `toaster.push(toast, cx)` | Shows the toast and returns its id. Past four toasts, the oldest leaves. |
| `toaster.dismiss(id, cx)` | Sends a toast away early. |
| `Toast::new(title)` | Stays 5 seconds. |
| `.body(text)`, `.severity(Severity::Success)` | A line of detail; a color and icon for info, success, warning or danger. |
| `.action(label, handler)` | A button that runs `handler`, then sends the toast away. |
| `.undo(handler)` | An Undo button and a line that runs out with the time left. |
| `.stay(None)` | Stays until closed. `Some(Duration::ZERO)` panics. |

## A popover, a dialog and a toast

The example opens a popover of details and a dialog that asks before it deletes. Its Delete button closes the dialog and pushes a toast.

```rust example=overlays
```
