# Mok GPUI (`mok-gpui`)

![Mok GPUI: components for GPUI](frontend/public/og.png)

Mok GPUI began as a fork of [Ely GPUI Components](https://github.com/ELY-Open-Source/Ely-GPUI-Components) (MIT) and is now a separate project with its own CLI and site.

A component library for [GPUI](https://www.gpui.rs), the Rust UI framework behind Zed.
Every component ships in light and dark. The palette is warm and quiet. Color carries meaning, not decoration.

Every component runs in the native gallery (`cargo run --example gallery`), and `frontend/` is a website that browses all of them with a real capture of each.

Status: early. `TASKS.md` tracks every component, chapter by chapter. To help, see `CONTRIBUTING.md`.

## Run the gallery

```sh
git clone https://github.com/amitverma-cf/mok-gpui
cd mok-gpui
cargo run --manifest-path cli/Cargo.toml -- init   # once: fetches the fonts and Lucide icons into assets/
cargo run --example gallery
```

The fonts and icons are not in git. Without the `init` step the build stops and tells you to run it. The website is `cd frontend && pnpm install && pnpm dev`.

## Use

Mok works like shadcn/ui: the `mok-gpui` crate holds the foundation (theme, motion, primitives, i18n, layout, typography), and `cargo mok` copies each component into your project as source you own. Fonts and icons are not bundled: `init` downloads them into your `assets/`, and the build embeds only what you use.

```sh
cargo install --path cli      # from a clone of this repository: installs `cargo mok`
cd your-project
cargo mok init                # mok-gpui.toml, .cargo/config.toml, fonts and Lucide into assets/
cargo mok add button dialog   # source into src/ui, with the crates and files it needs
```

Then add `mod ui;` to your crate and start it:

```rust
mod ui;

use mok_gpui::{Assets, theme::{Mode, Theme}};
use gpui::App;

fn main() {
    gpui_platform::application().with_assets(Assets).run(|cx: &mut App| {
        mok_gpui::init(cx).expect("Mok failed to start");
        ui::init(cx);
        Theme::set_mode(Mode::Dark, cx);
    });
}
```

`cargo mok add` also runs `cargo add` for `mok-gpui`, `gpui` and the other crates the copied files use. `init` registers the fonts and the theme, and returns an error if the application's asset source cannot load one of Mok's files.

## Configuration (`mok-gpui.toml`)

`mok-gpui.toml` sits at your project root, and `.cargo/config.toml` points the build at it (`MOK_CONFIG_PATH`; `cargo mok init` writes both). It drives tokens, palettes, layout metrics, fonts, media, data files and icon pruning with **no runtime deserialization** (constants and static slices are baked at compile time into `$OUT_DIR`). A build without it fails and says so.

```toml
[paths]
assets_dir = "assets"
icons_dir = "assets/icons"
scan_sources = ["src/**/*.rs"]

[theme]
preset = "mok"

[theme.palette.light]
bg = 0xfcfaf7
fg = 0x181613
# ... WCAG 2.1 AA (4.5:1) checked at build time

[theme.tokens]
base_rem = 16.0
dialog_width = 440.0

[icons]
mode = "auto"     # embeds only the icons your code names
minify = true

[media.images]
logo = { path = "logo.png", compress = "optipng" }

[media.files]     # data files a component loads; `cargo mok add` fills it in
countries = { path = "maps/countries.geojson" }
```

## The CLI (`cargo mok`)

```sh
cargo mok init                  # config, fonts and Lucide into assets/ (checksummed)
cargo mok list                  # every component, with what it needs
cargo mok list search chart     # find one
cargo mok add button            # copy it and its dependencies into src/ui
cargo mok add icons heroicons   # another icon pack into assets/icons
cargo mok update --dry-run      # what the source changed since you copied
cargo mok update                # bring copied files up to date, keeping your edits
cargo mok doctor                # contrast, fonts, packs
cargo mok registry --check      # in this repo: is registry.json current?
```

`add` follows the real `use` graph of the source, rewrites imports (`crate::theme` becomes `mok_gpui::theme`, `crate::forms` becomes `crate::ui::forms`), and records each file's hash in `mok.lock`, so running it again keeps your edits (`--force` replaces them). Some entries are not copyable: they live in the `mok-gpui` crate, belong to another component, or are native to gpui; `add` says which.

## Built-in Theme Presets

Mok ships with carefully tuned, WCAG AA compliant light and dark presets:
- `mok` (warm paper & ink, default)
- `zinc` (neutral modern technical gray)
- `slate` (cool bluish slate)
- `nord` (arctic oceanic frost)
- `catppuccin` (Latte & Macchiato pastel hues)
- `tokyo-night` (Day & Storm neon indigo)

Choose a preset in `mok-gpui.toml`:
```toml
[theme]
preset = "nord"
```

Or switch dynamically at runtime:
```rust
Theme::set_preset("tokyo-night", cx);
```

## High-Level Layout & Command Overlays

- **`AppLayout`**: Desktop shell layout organizing titlebar header, left primary sidebar, right inspector panel, scrollable body, and status bar.
- **`CommandPalette`**: Fast modal quick-search and command execution overlay (<kbd>⌘K</kbd> / <kbd>Ctrl+K</kbd>) with keyboard navigation, grouped items, and shortcut badges.

## Typed Provider Icons & Dead Asset Eviction

Icons are referenced through provider-scoped enums rather than loose string paths:

```rust
use mok_gpui::primitives::{Icon, LucideIcon};

Icon::new(LucideIcon::ArrowRight)
```

`HeroIcon` and `HugeIcon` exist once `cargo mok add icons heroicons` (or `hugeicons`) has fetched the pack; before that they have no variants. At compile time `build.rs` scans your sources and the library's own core modules, and bakes only the SVGs they name into `EMBEDDED_ICONS`. The rest of each pack adds nothing to the binary.
