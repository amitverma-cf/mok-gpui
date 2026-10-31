//! Copies components and what they need into the project, and brings copied files up to date.

use std::collections::BTreeSet;
use std::path::Path;

use crate::deps::{add_assets, add_crates};
use crate::graph::Tree;
use crate::lock::Lock;
use crate::source::{Options, Registry, closure, load, revision, source_root};
use crate::sync::{Action, apply, render};

/// `cargo mok add`: the named units, with what they need, joining what is already copied.
pub fn install(root: &Path, units: &[String], opts: &Options) -> Result<(), String> {
    let source = source_root(opts.from.as_deref())?;
    let reg = load(&source)?;
    let rel = opts.dir.to_string_lossy().replace('\\', "/");
    let lock = Lock::read(root, &rel);
    let mut wanted = lock.units();
    wanted.extend(units.iter().cloned());
    let rows = sync(root, &source, &reg, lock, &wanted, opts, false)?;
    report(&rows, false);
    println!("\nIn your crate: `mod ui;`, run the app with `.with_assets(ui::assets::Assets)`, and call `ui::init(cx);` once at start");
    Ok(())
}

/// Renders every wanted unit with its closure and writes what changed; the project's crates and data follow.
pub fn sync(
    root: &Path,
    source: &Path,
    reg: &Registry,
    mut lock: Lock,
    wanted: &BTreeSet<String>,
    opts: &Options,
    dry: bool,
) -> Result<Vec<(String, Action)>, String> {
    let lock_dir = opts.dir.to_string_lossy().replace('\\', "/");
    let tree = Tree::load(&source.join("src"))?;
    let mut base = wanted.clone();
    base.insert("assets.rs".into());
    let all = closure(reg, &base.iter().cloned().collect::<Vec<_>>())?;
    let rendered = render(
        root,
        source,
        reg,
        &tree,
        &all,
        &opts.dir.to_string_lossy().replace('\\', "/"),
    )?;
    let rows = apply(root, &mut lock, &rendered, opts.force, dry)?;
    if !dry {
        lock.source = revision(source);
        lock.ignore_scratch()?;
        lock.write()?;
        add_assets(root, source, reg, &all)?;
        add_crates(root, reg, &all, opts)?;
        if root.join("mok-gpui.toml").is_file() {
            crate::bake::run(root, &lock_dir)?;
        }
    }
    Ok(rows)
}

fn words(action: Action) -> &'static str {
    match action {
        Action::New => "new",
        Action::Current => "current",
        Action::Updated => "updated",
        Action::Replaced => "replaced",
        Action::Kept => "yours, the source did not change it",
        Action::Foreign => "not made by mok; left as it is (--force replaces it)",
        Action::Merged => "merged with your edits",
        Action::Conflict => "CONFLICT: fix the <<<<<<< marks",
        Action::Saved => "yours; the new version is beside it as .mok-new",
    }
}

/// One line for each file that changed or needs a look, then the counts.
pub fn report(rows: &[(String, Action)], dry: bool) {
    let mut counts = std::collections::BTreeMap::new();
    for (key, action) in rows {
        *counts.entry(words(*action)).or_insert(0) += 1;
        if !matches!(action, Action::Current | Action::New) {
            println!(
                "[{}] {key}: {}",
                if dry { "plan" } else { "mok" },
                words(*action)
            );
        }
    }
    let total = rows.len();
    let summary: Vec<String> = counts.iter().map(|(w, n)| format!("{n} {w}")).collect();
    let note = if dry {
        " (dry run: nothing written)"
    } else {
        ""
    };
    println!("[ok] {total} files: {}{note}", summary.join(", "));
}
