# Contributing

Thanks for helping. Bugs, component requests and pull requests are all welcome.

## Before you start

- Try the component in the gallery (`cargo run --example gallery`) first; it may already do what you need.
- Open an issue from a template (Bug Report, Component Request or Other) before a large change, so we agree on the shape first.

## Setup

- Rust 1.95 with `rustup target add wasm32-unknown-unknown`. No full Xcode needed: gpui compiles its shaders at runtime.
- For the website: Node 24 and pnpm 11.
- Development happens on macOS or Windows; macOS also plays the capture's scripted states.
- The fonts and Lucide icons are not in git. Once, after cloning, fetch them: `cargo run --manifest-path cli/Cargo.toml -- init`. A build without them stops and says so.

```sh
cargo run --example gallery              # every chapter, light and dark
cargo run --example gallery -- --page forms
```

## Where things go

- A component lives in `src/<chapter>/`, one folder per chapter. One component, one home; anything that duplicates another points to it.
- Its demo is a story on the chapter's page, `examples/gallery/pages/<chapter>.rs`. After adding or renaming a story, run `python3 scripts/stories.py`.
- `TASKS.md` and `tasks/` track every component; tick its line when it lands.
- The website is `frontend/`; its README section has the commands.

## House rules

`AGENTS.md` holds them all. The ones reviews catch most:

- Colors, sizes, radii and shadows come from `cx.theme()`; no raw `px()` in components. Spacing uses gpui's rem helpers.
- Every component works in light and dark, at 280px wide, by keyboard (Tab reaches it, a 1px focus ring shows, Enter and Space press it), and holds still under reduced motion.
- Fail fast and log state changes; no silent fallbacks.
- Files stay at or under 500 lines. Comments are one short line, and rare.
- A new rule or dependency gets a line in `AGENTS.md`.

## Checks

```sh
scripts/check.sh                         # fmt, clippy, tests, the web lint, house rules
cd frontend && pnpm test && pnpm build && pnpm e2e   # when you touch the website
```

CI runs the same on every pull request. Fill in the pull request template: what and why, light and dark screenshots for anything that draws.

## License

Mok is MIT. Unless you say otherwise, what you contribute is licensed the same way.

## Opening a pull request on GitHub

1. Fork https://github.com/amitverma-cf/mok-gpui
2. Clone your fork, commit your changes, and create a topic branch:

   ```bash
   git clone https://github.com/<your-username>/mok-gpui.git
   git checkout -b <feature-branch>
   ```

3. Open a pull request against `main` on GitHub.
