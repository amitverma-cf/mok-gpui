# The CLI

`cargo mok` copies components into your project as source, so they are yours to edit, and fetches the fonts and icons they draw with. Install it from a clone of the repository with `cargo install --path cli`.

## Commands

| Command | Does |
| --- | --- |
| `cargo mok init` | Writes `mok-gpui.toml` and `.cargo/config.toml`, and downloads fonts and Lucide into `assets/`. `--no-assets` skips the download; `--force` rewrites the config. |
| `cargo mok list` | Every chapter. `list <chapter>`, `list search <word>` and `list --all` narrow or widen it. |
| `cargo mok add <component>...` | Copies each component and the files it needs into `src/ui`. `--dir`, `--from <checkout>` and `--force` change where from and how. |
| `cargo mok update` | Brings the copied files up to date with the source and keeps your edits. `--dry-run` shows the plan; `--from <checkout>` and `--force` as for `add`. |
| `cargo mok add icons <pack>` | Fetches `lucide`, `heroicons` or `hugeicons` into `assets/icons`. |
| `cargo mok doctor` | Checks the config's contrast, fonts and icon packs. |
| `cargo mok registry [--check]` | In the Mok repository: rebuilds `registry.json`, or says whether it is current. |

## What add does

1. Finds the component's file, then every file it needs, by following its `use crate::...` paths through each module's own declarations and re-exports.
2. Copies those files into `src/ui/<chapter>/`, with each chapter's `mod.rs` cut down to the files you have.
3. Rewrites imports: the modules that stay in the `mok-gpui` crate (theme, motion, primitives, i18n, layout, typography) point at `mok_gpui::`, the copied chapters at `crate::ui::`.
4. Runs `cargo add` for the crates the files use, and puts any data file a component loads into `assets/` and `[media.files]`.
5. Records the copied files in `mok.lock`, and the text it wrote for each in `.mok/base`. Run `add` again and files you edited stay as you left them.

Add `mod ui;` to your crate, and call `ui::init(cx)` after `mok_gpui::init(cx)` to bind the keys the copied chapters use.

## Updating

`cargo mok update` renders the copied components again from the source and compares each file with what mok wrote last time (`.mok/base`, which belongs in git):

| Your file | The source | Result |
| --- | --- | --- |
| untouched | changed | updated |
| edited | unchanged | yours, left alone |
| edited | changed elsewhere | merged with your edits |
| edited | changed the same lines | conflict markers (`<<<<<<< yours`) to fix by hand; update reports them until they are gone |

Merging uses `git merge-file`. New files or crates that a changed component now needs arrive with it, and a file whose unit left the source stays as it is. Commit before you update, so a merge you dislike is one `git checkout` away.

## Which entries copy

Each registry entry has a status. `component` and `feature` copy; a feature is part of another component, and `add` copies that one. `core` lives in the `mok-gpui` crate: import it from there. `native` is gpui itself. `blocked` and `host` need something gpui lacks or your app provides. `unmapped` entries are in the plan but not code yet.

## Fonts and icons

`init` downloads Inter 4.1, JetBrains Mono 2.304, IBM Plex Sans 1.1 and Lucide 1.48, each checked against a SHA-256 pinned in the CLI. Nothing is bundled in the library. At build time the icon list is pruned to the icons your code and the library's core name.
