# Plan: mok-gpui as a shadcn-style toolkit for GPUI apps

Status: approved by Amit (section 10). Phase 1 (cleanup) is implemented in this PR; the rest is not.

## 1. Where the repo is today

I read the whole tree. The important finding is that **the shadcn model already exists in a first form**, so this is a reshaping, not a rewrite.

What exists and works:

- `cli/` is `cargo-mok` (about 3,300 lines, its own workspace). Commands: `init`, `list`, `add <component>`, `add icons <pack>`, `update [--dry-run]`, `doctor`, `registry [--check]`.
- `add` parses the library with `syn` (`graph.rs`), computes the files a component needs, rewrites `crate::` imports to the project's `ui` module (`rewrite.rs`), copies the source, and runs `cargo add` for the crates it uses.
- `mok.lock` records what was copied and the text mok last wrote, so `update` does a three-way merge and keeps the user's edits (`sync.rs`, `lock.rs`).
- `registry.json` is generated from the source (`cargo mok registry`): 1,294 entries in 44 chapters, plus a per-file `units` dependency graph (files, core modules, crates, assets).
- `mok-gpui.toml` drives tokens, palettes, fonts, media and icon pruning through `build.rs` and `build_support/` (baked at compile time, no runtime parsing).
- Fonts and Lucide are fetched with pinned SHA-256 (`cli/src/fetch.rs`), not stored in git.
- The `examples/gallery` app runs natively and in the browser (`scripts/web.sh`: wasm-bindgen, wasm-opt, 25 MiB Cloudflare limit). `frontend/` is the website with captures of every component.

What does not fit the goal:

1. **The foundation is still a crate.** `src/{theme,primitives,motion,layout,i18n,typography,assets}` (`CORE` in `cli/src/graph.rs`) stays in `mok-gpui`, and every project that runs `cargo mok add` also depends on `mok-gpui` (and so on `build.rs` and `mok-gpui.toml`). That is a library with a copy step on top. You said it should not be something you add in Cargo.
2. **gpui-kit leftovers.** See section 5.
3. **Ely leftovers.** The README and CONTRIBUTING still point at upstream; a few comments and `tasks/` mention "Ely"; the `MokFocus` context was renamed but its history lives in `tasks/kit.md`.
4. **The registry is a git clone.** `cargo mok add` shallow-clones the whole repo (`source.rs`) and parses it. There is no static, versioned, hostable registry, so third parties cannot publish their own.
5. **Platforms are only desktop plus web.** There is no target abstraction, no packaging, no project templates, no mobile.
6. **Tooling is scattered.** Generators are Python, shell and PowerShell (`scripts/`: `captcha.py`, `countries.py`, `icons.sh`, `shots.py`, `stories.py`, `tiles.py`, `web.sh`, `web.ps1`, `check.sh`). They only run on the author's machine and are not part of `cargo mok`.
7. **`.github/` holds issue templates only.** `check.sh` and the docs say CI runs on every push, but no workflow file is in the tree. Please confirm whether CI lives elsewhere.

## 2. Target architecture

Three parts, one workflow.

```
cargo-mok (the CLI)        registry (static JSON, hosted)        your app
 new / init / add   <-----  index.json, r/<item>.json   ----->   src/ui/**  (you own it)
 build / run / pack         built from registry source           mok.toml, mok.lock
 gen / fmt / check
```

### 2.1 Repository layout (target)

```
cli/                 cargo-mok: the only thing published to crates.io
registry/            component source, compiled as an unpublished crate so it is tested and drawn
  base/              theme, primitives, motion, layout, i18n, typography, assets (copied, not depended on)
  ui/<chapter>/...   today's src/<chapter>
  blocks/            screens built from components (login, dashboard, settings)
templates/           project templates used by `cargo mok new`
gallery/             today's examples/gallery, a normal app that consumes the registry
site/                today's frontend/
tools/               dev-only generators that stay out of the CLI
```

Moving `src/` to `registry/` is mechanical (1,000+ files) and should be one dedicated commit, after the removals in section 5.

### 2.2 The foundation is copied too (the main decision)

shadcn copies components but still needs a few small shared things (`cn`, tailwind config). Mok's equivalent is bigger: theme tokens, motion, focus scopes, i18n. Options:

- **A. Keep `mok-gpui` as a tiny base crate** (what exists). Simple, but it is a library, and it blocks editing the theme internals.
- **B. Copy the base too** as `src/ui/base`, and replace `build.rs` with a generator (recommended).

Recommendation: **B**. Why: it makes "you own all the code" true, removes the `build.rs` plus `MOK_CONFIG_PATH` coupling that is the least friendly part today, and lets the CLI generate tokens per project.

How B works:

- `cargo mok init` copies `base` into `src/ui/base`, writes `mok.toml` (today's `mok-gpui.toml`), and runs `cargo mok gen theme`, which writes plain Rust (`src/ui/base/tokens.gen.rs`) instead of baking into `$OUT_DIR`. Generated files are checked in, diffable and editable.
- Icons and fonts keep the pinned-download flow and `assets/` pruning, but pruning becomes `cargo mok gen assets` (scan sources for icon names, copy only those).
- `build_support/{tokens,palette,presets,fonts,icons,media}.rs` (about 1,900 lines) move into the CLI as the generator. `build.rs` goes away from the registry.

Cost: `update` has to merge a larger set of files, and every unit must compile against a project-local `base`. The smoke test in 2.5 covers it.

### 2.3 Registry format

Today: one `registry.json` generated from source and embedded in the binary. Target, modelled on shadcn's registry:

- `registry/index.json`: name, version, `gpui` requirement, list of items with type (`base`, `ui`, `block`, `icons`, `template`), description, tags.
- `r/<item>.json` per item:

```json
{
  "name": "dialog",
  "type": "ui",
  "gpui": "=0.3.9",
  "files": [{ "path": "ui/overlays/dialog.rs", "sha256": "…", "content": "…" }],
  "requires": ["base", "button", "focus-scope"],
  "crates": { "smallvec": "1" },
  "assets": { "icons": ["x"], "fonts": [] },
  "targets": ["desktop", "web"],
  "docs": "https://…/docs/dialog"
}
```

- It is produced by `cargo mok registry build` from the existing `syn` graph (the dependency work is done; it needs a serializer with file contents and a per-item `targets` field from `cfg(target_family = "wasm")` analysis).
- Hosted as static files on the existing Cloudflare site (`site/public/r/`). The CLI fetches over HTTPS, verifies the sha256, and falls back to `--from <checkout>` for local work. The git clone path is kept as a fallback only.
- **Third-party registries**: `mok.toml` gets `[registries] acme = "https://acme.dev/r/{name}.json"`, so `cargo mok add @acme/data-grid` works. This is what makes it pluggable beyond this repo.
- The lock file keeps its current three-way-merge design and gains the registry URL and item version.

### 2.4 gpui version policy

`mok-gpui` pins `gpui-pre =0.3.9`, the crates.io snapshot of Zed's gpui, because gpui-kit pinned it. With the kit removed, the pin is free. Recommendation: keep `gpui-pre` (resolvable from crates.io, no git dependency in user apps) and record the pin in `index.json`. `cargo mok add` writes the matching `gpui` line and refuses, with a clear message, when the project's gpui differs. Bump with a registry release, not silently.

### 2.5 Quality gate for a registry of copied code

The thing that will break a copy-based system is an item that compiles inside the repo but not alone in a fresh project. Add, in CI:

- For each item (or each chapter at first, then each item): `cargo mok new` into a temp dir, `cargo mok add <item>`, `cargo check` for native and wasm32. This is the one test that proves the product.
- `cargo mok registry --check` (exists) so `index.json` never drifts from source.
- The house rules in `scripts/check.sh` (no raw `px()`, 500-line files, `web_time::Instant`, `deferred` only in one place) move to `cargo mok check --registry` so contributors and third-party registries share them.

## 3. CLI design

Single binary `cargo-mok`, invoked as `cargo mok …` and also `mok …`. Keep existing commands; additions marked new.

| Command | Purpose |
|---|---|
| `new <name> [--template desktop\|web\|mobile\|workspace] [--platform …]` (new) | Scaffold an app (like `create-gpui-app`, plus Mok already wired). Runs `init`. |
| `init` | Project config, base, theme generation, assets. Idempotent. |
| `add <item…> [--dry-run] [--overwrite]` | Items from the default or a named registry; shows the file plan first. |
| `remove <item>` (new) | Removes files only mok wrote and nobody else needs; warns about edited files. |
| `diff <item>` (new) | Registry version against your edited copy. |
| `update [item] [--dry-run]` | Existing three-way merge. |
| `list`, `search`, `view <item>` | Browse the registry; `view` prints files without writing. |
| `build [--target …] [--release]` (new) | Platform-aware build, section 4. |
| `run [--target …]` and `dev` (new) | Run, with a watch loop on desktop and a local server on web. |
| `package --target …` (new) | Installers and bundles, section 4. |
| `target add <name>` (new) | Installs the rustup targets and checks external SDKs for a platform. |
| `gen theme\|assets\|icons\|countries\|stories` (new) | The generators, section 6. |
| `fmt`, `check` (new) | `cargo fmt` + clippy + the Mok house rules on your `src/ui`. |
| `doctor` | Existing config checks, plus toolchain and SDK checks per target. |
| `registry build\|check` | Maintainer commands. |

Principles: every write is previewed with `--dry-run`; never overwrite an edited file without `--overwrite` or a merge; machine-readable `--json` on list/view/doctor so editors and agents can drive it; no network on `build`.

## 4. Platform build targets

Tracked in `mok.toml` as `[targets.<name>]`. `cargo mok build --target x` expands to the right cargo invocation, env and post-steps, so users never copy `web.sh`.

| Target | Status today | Plan |
|---|---|---|
| macOS, Windows, Linux | Native; gallery runs; no packaging | `build`/`run` are thin wrappers over cargo. `package`: macOS `.app` + `.dmg`, Windows `.msi`/zip (cargo-packager or cargo-wix), Linux AppImage + `.deb`. Evaluate `cargo-packager` first rather than writing installers. Shader note: gpui compiles shaders at runtime, so no Xcode is required. |
| Web (wasm) | Works via `scripts/web.sh`; 20 MB wasm | Port `web.sh`/`web.ps1` into `build --target web`: pins/checks the `wasm-bindgen` CLI at the `Cargo.lock` version, runs `wasm-opt`, emits `index.html`, reports size, `RUSTC_BOOTSTRAP` workaround kept and documented. Add `run --target web` serving locally. Asset URLs via `MOK_GALLERY_ASSETS`-style config. |
| iOS, Android | None | Build on [gpui-mobile](https://github.com/itsbalamurali/gpui-mobile) (wgpu, Metal/Vulkan, uses cargo-ndk, xcodegen, Gradle). Needs a spike: it does not state which gpui version it pins, so version compatibility with `gpui-pre =0.3.9` is unknown. Experimental flag until proven. |
| react-native-gpui | None | **Dropped (decided 2026-10-31).** GPUI draws its own GPU surface through its platform layer; there is no JS bridge, and embedding that surface inside a React Native Fabric view would mean writing a new GPUI platform plus a native view manager per OS, on top of the still-unproven gpui-mobile. The awesome-gpui entry links to a Swift/GPL project that does not use GPUI. Revisit only after iOS/Android work. |
| Terminal / embedded | None | Out of scope for v1. [Crepuscularity](https://github.com/tschk/crepuscularity) shows the multi-target idea (`crepus.toml`, `crepus build`) but renders its own IR, which does not fit Mok's source-copy model. |

Per-item `targets` in the registry (section 2.3) lets `add` warn "this component uses a terminal/webview and is native only". Some of this exists as `ui::native_only` in the gallery.

## 5. Removing gpui-kit and Ely

### gpui-kit (everything below is only there to run beside Longbridge's GPUI Kit)

Delete:

- `compat/kit/` (the whole crate: Cargo.toml, `src/{assets,desk,dialogs,focus}.rs`, `examples/start.rs`).
- `tasks/kit.md` and its line in `TASKS.md`.
- `frontend/src/docs/gpui-kit.md`, its entry in `frontend/src/docs/guides.ts`, and kit mentions in `installation.md`, `icons.md`.
- The kit block in `scripts/check.sh` (the `compat/kit` lines and the `cargo search gpui-kit` check).
- Kit text in `README.md` and `AGENTS.md`, and the `.gitignore` entry.

Change in code (small, check each):

- `src/assets.rs`: `Assets::before` exists so two libraries can share one asset source. Delete it and its tests once nothing else uses it. Keep the check that every embedded file loads, since that guards against a missing `init`.
- `src/lib.rs`: drop any kit-specific wording in `init`'s error message.
- Keep the `MokFocus` Tab binding (K02). It is correct for any host app with its own key contexts, only the justification text changes.
- Keep the gpui pin until section 2.4 is decided, then re-evaluate.

Acceptance: `grep -ri "gpui.kit\|compat/kit\|longbridge"` returns nothing outside license notes; `check.sh` passes without network access to crates.io search.

### Ely and fork wording

- README: keep one honest attribution line (MIT requires keeping the upstream copyright notice, check `LICENSE`), remove "elygpui.com shows upstream's components" and the upstream link in `CONTRIBUTING.md`.
- Clean "Ely" in `tasks/` and code comments; rename leftovers by grep.

### Library framing

- `Cargo.toml`: `publish = false` already; keep. Remove the "component library" wording in README, description and site copy: it should say "a CLI and registry; you own the code".
- README "Use" section leads with `cargo install cargo-mok`, `cargo mok new`, `cargo mok add`, not `mok_gpui::init`.

## 6. Scripts and generators

Fold what users need into the CLI, keep the rest as dev tooling.

| Script | Plan |
|---|---|
| `check.sh` | Split: registry-only checks to `cargo mok check --registry`; the rest stays as a thin CI entry point in `tools/`. |
| `web.sh`, `web.ps1` | Replaced by `cargo mok build --target web`. One implementation instead of two shell dialects. |
| `icons.sh` | `cargo mok gen icons` (pinned Lucide download exists in `fetch.rs`; add Heroicons, Hugeicons packs already referenced by `add icons`). |
| `countries.py`, `captcha.py` | `cargo mok gen data` for data files the `forms` components embed. Port to Rust so the generated data is reproducible with no Python. |
| `stories.py` | `cargo mok gen stories` (gallery index) or a derive, see below. |
| `shots.py`, `tiles.py` | Dev-only (website captures); keep in `tools/`, port later. |

Cross-platform rule: nothing the end user needs may require Python, sh or PowerShell.

## 7. What to reuse from awesome-gpui

I read the awesome-gpui list and the READMEs of five projects (create-gpui-app, gpui-mobile, the React Native GPUI link, Crepuscularity, gpui-storybook). The rest is judged from the one-line descriptions only; **verify code, licence and gpui version before borrowing anything**.

Tooling and patterns (highest value):

- **create-gpui-app** (MIT): `--name`, `--workspace` flags and a `templates/` dir. Model for `cargo mok new`; we do not need to depend on it.
- **gpui-mobile**: the route to iOS and Android; see section 4.
- **gpui-storybook**: `#[story]` / `ComponentStory` registration, plus `-test` capture matrices and visual baselines for CI, and an MCP crate. Mok keeps stories in `stories.json` generated by a script; a derive or attribute would remove that script and the manual "run stories.py after adding a story" step. Worth a prototype.
- **gpui-agent** (wsp-labs): CLI/MCP so agents can drive a GPUI app. Candidate for `cargo mok run --agent` and for scripted UI tests.
- **gpui-devtools**: element inspector; Mok already ships its own inspector, compare before adding.

Components and libraries to port as registry items (as reference or via adapter, licences permitting):

- **gpui-nav**, **gpui-router**: routing. Mok has `navigation/` but no router item.
- **gpui-hooks**, **gpui-tea**, **gpui-form**: state and form patterns. The `forms/` chapter could gain a form-state helper.
- **gpui-flow / ferrum-flow**: node editors; Mok has `canvas/`, check for overlap.
- **gpui-pdf, gpui-whiteboard, ratex-gpui** (from Zorite): Mok already depends on `ratex-*`; PDF viewer and whiteboard fill gaps in `documents/` and `canvas/`.
- **gpui-video-player**, **plotters-gpui**, **gpui-px/d3rs**: media and charts additions.
- **Base GPUI** (Base UI headless APIs), **guise** (Mantine tokens), **HIG for GPUI**, **adabraka-ui**, **gpui-component** (Longbridge): API and token inspiration; useful as a checklist of what users will expect.

Two rules for borrowing: record source and licence in the item's registry entry (`"credit"`), and prefer a registry adapter that copies the other project's source with attribution over a vendored fork that diverges.

## 8. Roadmap

Each phase ends in a merged PR and a working `cargo mok add` flow.

**Phase 0: approval.** Done.

**Phase 1: clean up (small, low risk).**
- Remove gpui-kit and Ely leftovers (section 5), including `Assets::before`.
- Reword README/site as CLI-first. Add a workflow file for CI if it is missing.
- Done when: grep is clean, `check.sh` passes offline.

**Phase 2: static registry and the smoke test.** (2a done: `cargo mok registry build <dir>` writes `index.json` + a sha256-checked `source.tar.gz`; `MOK_REGISTRY_URL=<dir url> cargo mok add …` fetches it instead of cloning. Still open: per-item `r/*.json`, hosting it on the site, the fresh-project compile CI job.)
- `registry build` writes `index.json` + `r/*.json`; CLI reads from URL with sha256 check and `--from` for local.
- Fresh-project CI job for each chapter (native + wasm32).
- Done when: `cargo mok add button` in an empty project fetches over HTTPS and compiles.

**Phase 3: own the foundation (decision 2.2).** (3a done: the foundation is copied as units, `cargo mok gen` replaces `build.rs`, `add` no longer adds `mok-gpui`; a fresh project with `button` compiles. Still open: moving `src/` to `registry/`, `frontend/` to `site/`, and `cargo mok remove`.)
- Move `src/` → `registry/`, copy `base` in `init`, replace `build.rs` with `cargo mok gen theme|assets`.
- Done when: a new project has no `mok-gpui` dependency and no `build.rs`.

**Phase 4: project lifecycle commands.**
- `new` with templates, `remove`, `diff`, `check`, `fmt`, `view`, `search`.
- Port `icons.sh`, `countries.py`, `captcha.py` into `gen`.

**Phase 5: platforms.**
- `build`/`run`/`package` for macOS, Windows, Linux, web (folds in `web.sh`).
- Spike gpui-mobile compatibility, then iOS/Android as experimental.
- react-native-gpui per your answer.

**Phase 6: ecosystem.**
- Third-party registries (`@namespace`), `blocks/`, storybook-style stories, awesome-gpui ports one at a time.
- Publish `cargo-mok` to crates.io and add `cargo binstall` binaries via a release workflow.

## 9. Risks

- **gpui churn.** Zed's gpui has no stable API. The registry pins one version per release; upgrading is a deliberate registry release with a migration note, and `update` merges edits.
- **Merge pain at scale.** Copying the base (2.2) makes updates touch more of the user's files. Mitigation: `diff`/`--dry-run` first, lock-file merge already handles edits, and the base is kept small and stable.
- **Size.** The gallery wasm is 20 MB against a 25 MiB limit; per-item copying helps users, but the gallery itself should be split by chapter before it hits the limit.
- **Licence.** Anything ported from awesome-gpui needs a licence check (Apache-2.0 needs a notice, GPL such as the nucleus repository cannot be copied into an MIT registry).
- **Naming.** Check that `cargo-mok` and `mok` are free on crates.io before the first publish.

## 10. Decisions (Amit, 2026-10-31)

1. react-native-gpui: dropped for now (section 4).
2. The foundation is copied into projects (2.2, option B): yes.
3. `src/` moves to `registry/` and `frontend/` to `site/`: yes, in one isolated commit.
4. Keep `gpui-pre` as the pinned source: yes.
5. Mobile follows desktop and web.

Phase 1 (cleanup) is done in this PR; later phases need their own PRs.
