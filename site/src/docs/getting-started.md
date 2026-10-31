# Getting started

An Mok app is a GPUI app with four more parts: Mok's assets, one call to `init`, a root `FocusScope` around each window's content, and colors from the theme.

## Four parts

| Part | Why |
| --- | --- |
| `gpui_platform::application().with_assets(Assets)` | Mok embeds its icons and fonts. `init` returns an error when the source cannot load one of them. |
| `mok_gpui::init(cx)?` | Call once, before any window. It returns an error if the asset source fails. It registers the fonts, sets the theme, and binds Tab, Shift-Tab and the keys of text fields, the code editor, documents and, natively, the terminal. |
| A root `FocusScope` | `FocusScope::new(&handle).root()` owns Tab for the window and takes focus back when the focused element leaves the tree. Focus its handle when the window opens. |
| `cx.theme()` | Colors, sizes and radii come from the theme. Components read it; an app's own boxes read it too. |

## A window that follows the theme

The example opens a titled window in dark mode. Its button cross-fades every component to the other mode.

```rust example=start
```

`Theme::set_mode_now` before the window opens makes the first frame dark. `Theme::set_mode` fades the palette and repaints every window as it goes, so the button needs no `notify`. `Heading` and `Paragraph` take their size, weight and color from the theme.

## Changing state from a press

`Button::on_click` takes a closure over `&ClickEvent`, `&mut Window` and `&mut App`. To change a view's own state, capture the view with `cx.entity()` in `render`, update it in the closure, and call `cx.notify()`. The [first window](/docs/#a-first-window) counts presses that way. A change made outside `render` without `cx.notify()` draws nothing.

## Desktop shell layout

For full desktop applications with structured window regions, use `AppLayout`:

```rust
use mok_gpui::layout::AppLayout;

AppLayout::new()
    .header(title_bar)
    .sidebar(navigation_sidebar)
    .sidebar_right(inspector_panel)
    .footer(status_bar)
    .child(main_content_view)
```
