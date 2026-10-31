//! Copies components and what they need into the project, and brings copied files up to date.

use std::collections::BTreeSet;
use std::path::Path;

use crate::deps::{add_assets, add_crates};
use crate::graph::Tree;
use crate::lock::Lock;
use crate::source::{LIB_DIR, Options, Registry, closure, load, revision, source_root};
use crate::sync::{Action, apply, render};

/// `cargo mok add`: the named units, with what they need, joining what is already copied.
pub fn install(root: &Path, units: &[String], opts: &Options) -> Result<(), String> {
    let source = source_root(opts.from.as_deref())?;
    let reg = load(&source)?;
    let rel = opts.dir.to_string_lossy().replace('\\', "/");
    let lock = Lock::read(root, &rel);
    let mut lock = lock;
    if lock.requested.is_empty() {
        lock.requested = lock.units();
    }
    lock.requested.extend(units.iter().cloned());
    let wanted = lock.requested.clone();
    let rows = sync(root, &source, &reg, lock, &wanted, opts, false)?;
    report(&rows, false);
    println!(
        "\nIn your crate: `mod ui;`, run the app with `.with_assets(ui::assets::Assets)`, and call `ui::init(cx);` once at start"
    );
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
    let tree = Tree::load(&source.join(LIB_DIR))?;
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

/// `cargo mok remove`: drops the named units and whatever only they needed; edited files stay.
pub fn remove(root: &Path, units: &[String], opts: &Options) -> Result<(), String> {
    let source = source_root(opts.from.as_deref())?;
    let reg = load(&source)?;
    let rel = opts.dir.to_string_lossy().replace('\\', "/");
    let mut lock = Lock::read(root, &rel);
    if lock.requested.is_empty() {
        lock.requested = lock.units();
    }
    for unit in units {
        if !lock.requested.remove(unit) {
            return Err(format!("{unit} was not added by name; see mok.lock"));
        }
    }
    let mut base: Vec<String> = lock.requested.iter().cloned().collect();
    base.push("assets.rs".into());
    let keep = closure(&reg, &base)?;
    let gone: Vec<(String, String)> = lock
        .files
        .iter()
        .filter(|(_, unit)| !keep.contains(*unit))
        .map(|(k, u)| (k.clone(), u.clone()))
        .collect();
    for (key, _) in &gone {
        let path = root.join(key);
        let edited = match (std::fs::read_to_string(&path), lock.read_base(key)) {
            (Ok(now), Some(base)) => now.replace("\r\n", "\n") != base.replace("\r\n", "\n"),
            (Ok(_), None) => true,
            _ => false,
        };
        if edited {
            println!("[mok] {key}: edited, left in place (delete it yourself)");
        } else {
            let _ = std::fs::remove_file(&path);
            println!("[mok] removed {key}");
        }
        lock.files.remove(key);
    }
    let wanted = lock.requested.clone();
    let rows = sync(root, &source, &reg, lock, &wanted, opts, false)?;
    report(&rows, false);
    Ok(())
}

/// `cargo mok diff`: your copies against what the source would write now.
pub fn diff(root: &Path, opts: &Options) -> Result<(), String> {
    let source = source_root(opts.from.as_deref())?;
    let reg = load(&source)?;
    let rel = opts.dir.to_string_lossy().replace('\\', "/");
    let lock = Lock::read(root, &rel);
    if lock.files.is_empty() {
        return Err("nothing was copied yet".into());
    }
    let tree = Tree::load(&source.join(LIB_DIR))?;
    let start: Vec<String> = lock.units().into_iter().collect();
    let all = closure(&reg, &start)?;
    let rendered = render(root, &source, &reg, &tree, &all, &rel)?;
    let mut changed = 0;
    for (unit, new) in &rendered {
        let path = root.join(&rel).join(unit);
        let now = std::fs::read_to_string(&path).unwrap_or_default();
        if now.replace("\r\n", "\n") == new.replace("\r\n", "\n") {
            continue;
        }
        changed += 1;
        let scratch = root.join(".mok").join("tmp");
        std::fs::create_dir_all(&scratch).map_err(|e| e.to_string())?;
        let theirs = scratch.join("diff-new");
        std::fs::write(&theirs, new).map_err(|e| e.to_string())?;
        let out = std::process::Command::new("diff")
            .args(["-u", "--label"])
            .arg(format!("yours/{unit}"))
            .arg("--label")
            .arg(format!("source/{unit}"))
            .arg(&path)
            .arg(&theirs)
            .output()
            .map_err(|e| format!("diff is needed: {e}"))?;
        print!("{}", String::from_utf8_lossy(&out.stdout));
    }
    println!("[ok] {changed} of {} files differ", rendered.len());
    Ok(())
}
