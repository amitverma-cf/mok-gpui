# Icons, fonts and assets

`Assets` serves the icons, fonts and data files the build baked in from your `mok-gpui.toml`; none of them live in the library. `cargo mok init` downloads Inter, JetBrains Mono, IBM Plex Sans and Lucide into your `assets/`, and only what you declare or use reaches the binary. Pass it to `with_assets`, or `Assets::before(other)` to serve Mok's files first and every other path from another source; `init` checks that each of Mok's files loads through the app's source, then registers the fonts.

## Icons

Mok has typed icon enums (`LucideIcon`, `HeroIcon`, `HugeIcon`), each implementing `IconAsset`: `LucideIcon::Check` is `icons/check.svg`, and `name()` returns `check`. `HeroIcon` and `HugeIcon` have variants once their pack is on disk. `build.rs` scans your sources and the library's core modules, and embeds only the icons they name.

| Call | Does |
| --- | --- |
| `Icon::new(icon)` | Draws the icon (e.g. `LucideIcon::Check`) at `IconSize::Md` in `fg`. |
| `Icon::from_path(path)` | Draws an SVG from the application's asset source with the same defaults. |
| `.size(IconSize::Xl)` | `Xs` to `Xxl`, from the theme's icon sizes. |
| `.color(color)` | Any color; components pass theme colors. |
| `.rotate(Radians(..))` | Turns the glyph inside its box. |
| `.scale(factor)` | Draws the glyph larger or smaller in the same box. |

GPUI's `svg()` paints with its own text color alone, so `Icon` always sets one.

## Adding icon packs

A pack in `assets/icons` is what makes its enum exist:

```sh
cargo mok add icons heroicons
cargo mok add icons hugeicons
```

The build embeds only the SVGs your code names; the rest of the pack stays on disk.

## File icons

`FileIcon::file(name)` shows the icon a file's name suggests, and `FileIcon::folder(open)` a folder's. Mok maps extensions to icons: `rs` and `ts` to `FileCode`, `toml` to `FileCog`, `json` to `FileJson`, images, audio, video, archives and more. An app lays its own rules over that map:

| Call | Does |
| --- | --- |
| `IconTheme::default().name("Cargo.toml", LucideIcon::Package)` | A file of that exact name, in any case. |
| `.extension("md", LucideIcon::BookOpen)` | Files with that extension, given without its dot. |
| `.apply(cx)` | Serves every window from then on, and repaints. |

A name rule beats an extension rule, and both beat Mok's map.

## Fonts

`init` registers the seven fonts `cargo mok init` downloaded, served from `Assets`: Inter Regular, Medium, SemiBold and Italic; JetBrains Mono Regular and Medium; IBM Plex Sans Regular. The theme draws text in Inter and code in JetBrains Mono, set by its `font_family` and `mono_family` fields. On the web, GPUI answers the system UI font with IBM Plex Sans, and text inside SVGs falls back to it.

## An app's own assets

GPUI takes one asset source. An app with files of its own, or another library's, passes `Assets::before(theirs)`: Mok's paths load from Mok, every other path from `theirs`, whose errors pass through. `init` returns an error naming the first of Mok's files the source cannot load.

Use `Icon::from_path("app-icons/mark.svg")` to draw one of those SVG assets with Mok's themed sizing and color. It also accepts an owned `String` or `SharedString`. Paths are asset keys resolved by `AssetSource`; a path it lacks logs an error on first draw and shows a red broken-picture icon. SVGs render as single-color masks; `LucideIcon` and `IconPicker` continue to list the bundled icons.

Compatible single-color SVGs can come from [Lucide](https://lucide.dev/), [Hugeicons](https://github.com/hugeicons/hugeicons), [Phosphor](https://github.com/phosphor-icons/core), [Remix Icon](https://github.com/Remix-Design/RemixIcon), or [Tabler](https://github.com/tabler/tabler-icons). The app supplies its chosen files and preserves their licenses; Mok does not download or register those catalogs.

The example below includes its asset source. Copy [`examples/assets/mark.svg`](https://github.com/amitverma-cf/mok-gpui/blob/main/examples/assets/mark.svg) with it, or supply your own SVG.

## A window of icons

```rust example=icons
```
