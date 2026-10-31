# Installation

## Requirements

- Rust 1.95 or later, the version Mok's `Cargo.toml` names.
- macOS, Windows, or Linux (all tested in CI).
- No full Xcode: GPUI compiles its Metal shaders at run time.

## Add the dependencies

Mok is not on crates.io. Depend on it by git, with GPUI at the snapshot Mok pins. `gpui-pre` republishes Zed's GPUI on crates.io:

```toml
[dependencies]
mok-gpui = { git = "https://github.com/amitverma-cf/mok-gpui" }
gpui = { package = "gpui-pre", version = "=0.3.9" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.9", features = ["font-kit"] }
```

| Line | Why |
| --- | --- |
| `mok-gpui` | The library. `Cargo.lock` records the commit; add `rev = "<commit>"` to hold it in `Cargo.toml` as well. |
| `gpui` | The same snapshot as Mok's, under the name `gpui`. Another builds a second GPUI, and its types do not match the ones Mok takes. |
| `gpui_platform` | Opens windows on each platform. Without `font-kit`, macOS falls back to GPUI's no-op text system and draws no text. |

## Shaders

Mok's default feature, `runtime_shaders`, turns on the same feature in `gpui_platform`, so GPUI compiles its Metal shaders at run time. With `default-features = false` the build runs `xcrun metal` instead, which needs Xcode's Metal compiler.

## The `cargo mok` tool

`cargo mok` sets a project up and copies components into it as source. From a clone of the repository:

```sh
cargo install --path cli
cd your-project
cargo mok init
cargo mok add button
```

`init` writes `mok-gpui.toml` and `.cargo/config.toml`, and downloads Inter, JetBrains Mono, IBM Plex Sans and the Lucide icons into `assets/`, each checked against a pinned checksum. See [the CLI guide](/docs/#the-cli) for every command.

## Project configuration

`mok-gpui.toml` at the project root defines design tokens, paths, media and icon pruning. The build reads it through `MOK_CONFIG_PATH`, which `init` sets in `.cargo/config.toml`:

```toml
[paths]
scan_sources = ["src/**/*.rs"]

[icons]
mode = "auto"
```

## Check the install

Copy the [first window](/docs/#a-first-window) into `src/main.rs` and run it:

```sh
cargo run
```

A window opens with a counter and a Press button. From a clone of the repository, `cargo run --example gallery` opens every chapter.

## When it fails

| Symptom | Cause and fix |
| --- | --- |
| Build error: `mok: MOK_CONFIG_PATH is not set` | The project has no `mok-gpui.toml` for the build to read. Run `cargo mok init`. |
| `init` error: `mok: the asset source lacks N of Mok's files, first <path>: <reason>` | The app's source cannot load every file Mok bundles. Pass `Assets`, or `Assets::before(yours)` when the app has a source of its own. |
| `init` error: `mok: embedded fonts failed to register` | GPUI's text system refused Mok's fonts. File a bug with the log. |
| Windows open with no text on macOS | `gpui_platform` lacks the `font-kit` feature. |
| `mismatched types`, with the note ``there are multiple different versions of crate `gpui` in the dependency graph`` | `gpui` names another snapshot or a git source. Use `gpui-pre` at `=0.3.9`. |
| `metal shader compilation failed` | Default features are off and Xcode's Metal compiler is missing. Keep `runtime_shaders`. |
