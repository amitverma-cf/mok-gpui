//! Brings the copied files to what the source now says, keeping what the project changed.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::graph::Tree;
use crate::lock::Lock;
use crate::rewrite::transform;
use crate::source::Registry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    New,
    Current,
    Updated,
    Replaced,
    /// Edited here, and the source did not change it.
    Kept,
    /// Edited here and not made by mok; left alone.
    Foreign,
    Merged,
    Conflict,
    /// The merge could not run; the new text sits beside the file.
    Saved,
}

/// What to do with one file, from its text on disk, the text mok last wrote and the new text.
#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    Do(Action),
    Merge,
}

fn lf(text: &str) -> String {
    text.replace("\r\n", "\n")
}

pub fn decide(
    existing: Option<&str>,
    base: Option<&str>,
    new: &str,
    tracked: bool,
    force: bool,
) -> Step {
    let Some(existing) = existing.map(lf) else {
        return Step::Do(Action::New);
    };
    let new = lf(new);
    if existing == new {
        return Step::Do(Action::Current);
    }
    if force {
        return Step::Do(Action::Replaced);
    }
    if !tracked {
        return Step::Do(Action::Foreign);
    }
    match base.map(lf) {
        Some(base) if base == existing => Step::Do(Action::Updated),
        Some(base) if base == new => Step::Do(Action::Kept),
        Some(_) => Step::Merge,
        None => Step::Do(Action::Saved),
    }
}

/// `git merge-file`: the merged text and how many conflicts it holds.
fn merge3(root: &Path, ours: &str, base: &str, theirs: &str) -> Result<(String, usize), String> {
    let tmp = root.join(".mok").join("tmp");
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let files = [("yours", ours), ("base", base), ("new", theirs)];
    for (name, text) in files {
        fs::write(tmp.join(name), text).map_err(|e| e.to_string())?;
    }
    let out = Command::new("git")
        .args([
            "merge-file",
            "-p",
            "-L",
            "yours",
            "-L",
            "last mok",
            "-L",
            "new mok",
        ])
        .args(files.map(|(name, _)| tmp.join(name)))
        .output()
        .map_err(|e| format!("git is needed to merge: {e}"))?;
    match out.status.code() {
        Some(code) if (0..128).contains(&code) => Ok((
            String::from_utf8_lossy(&out.stdout).into_owned(),
            code as usize,
        )),
        other => Err(format!(
            "git merge-file failed ({other:?}): {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )),
    }
}

fn rustfmt(path: &Path) {
    let _ = Command::new("rustfmt")
        .args(["--edition", "2024", "--quiet"])
        .arg(path)
        .output();
}

/// `ui/mod.rs`: the chapters present and one call that binds their keys.
fn index(dest: &Path, units: &BTreeSet<String>, chapters: &BTreeSet<String>) -> Result<(), String> {
    let present: BTreeSet<&str> = units
        .iter()
        .filter_map(|u| u.split('/').next().map(|c| c.trim_end_matches(".rs")))
        .filter(|c| chapters.contains(*c))
        .collect();
    let mut out = String::from(
        "//! Managed by `cargo mok`; the components live in the folders beside this file.\n#![allow(dead_code, unused_imports)]\n\n",
    );
    let mut binds = vec![];
    for chapter in &present {
        out.push_str(&format!("pub mod {chapter};\n"));
        let chapter_index = dest.join(chapter).join("mod.rs");
        if fs::read_to_string(&chapter_index).is_ok_and(|t| t.contains("fn bind_keys")) {
            binds.push(*chapter);
        }
    }
    out.push_str("\n/// Loads the fonts and theme, and binds the keys the copied components use.\npub fn init(cx: &mut gpui::App) {\n");
    if present.contains("assets") {
        out.push_str(
            "    assets::load_fonts(cx).expect(\"mok: embedded fonts failed to register\");\n",
        );
    }
    if present.contains("theme") {
        out.push_str("    theme::Theme::init(cx);\n");
    }
    if present.contains("primitives") {
        out.push_str("    primitives::bind_focus_keys(cx);\n");
    }
    for chapter in &binds {
        out.push_str(&format!("    {chapter}::bind_keys(cx);\n"));
    }
    if binds.is_empty() && !present.contains("theme") && !present.contains("assets") {
        out.push_str("    let _ = cx;\n");
    }
    out.push_str("}\n");
    fs::write(dest.join("mod.rs"), out).map_err(|e| e.to_string())
}

/// The files as the source now makes them, formatted, by unit; the chapters' index is `mod.rs`.
pub fn render(
    root: &Path,
    source: &Path,
    reg: &Registry,
    tree: &Tree,
    all: &BTreeSet<String>,
    dir: &str,
) -> Result<BTreeMap<String, String>, String> {
    let stage = root.join(".mok").join("stage").join("ui");
    let _ = fs::remove_dir_all(&stage);
    for unit in all {
        let text = fs::read_to_string(source.join("src").join(unit))
            .map_err(|e| format!("{unit}: {e}"))?;
        let mp = tree
            .mods
            .iter()
            .find(|(_, m)| m.unit == *unit)
            .map(|(k, _)| k.clone())
            .ok_or_else(|| format!("{unit}: not in the source tree"))?;
        let baked = format!("concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{dir}/gen/");
        let out = transform(&text, unit, all, &reg.chapters, tree, &mp)?
            .replace("concat!(env!(\"OUT_DIR\"), \"/", &baked);
        let dest = stage.join(unit);
        fs::create_dir_all(dest.parent().expect("a unit has a folder"))
            .map_err(|e| e.to_string())?;
        fs::write(&dest, out).map_err(|e| format!("{}: {e}", dest.display()))?;
    }
    index(&stage, all, &reg.chapters)?;
    rustfmt(&stage.join("mod.rs"));
    let mut done = BTreeMap::new();
    for unit in all.iter().map(String::as_str).chain(["mod.rs"]) {
        let text = fs::read_to_string(stage.join(unit)).map_err(|e| format!("{unit}: {e}"))?;
        done.insert(unit.to_string(), text);
    }
    fs::remove_dir_all(root.join(".mok").join("stage")).map_err(|e| e.to_string())?;
    Ok(done)
}

/// Writes (or, when `dry`, only plans) each file; the lock follows what was written.
pub fn apply(
    root: &Path,
    lock: &mut Lock,
    rendered: &BTreeMap<String, String>,
    force: bool,
    dry: bool,
) -> Result<Vec<(String, Action)>, String> {
    let mut rows = vec![];
    let rel = lock.dir.clone();
    for (unit, new) in rendered {
        let key = format!("{rel}/{unit}");
        let dest = root.join(&key);
        let existing = fs::read_to_string(&dest).ok();
        if unit == "mod.rs" {
            // The index is managed: it always follows the chapters present.
            if !dry && existing.as_deref().map(lf) != Some(lf(new)) {
                fs::write(&dest, new).map_err(|e| e.to_string())?;
            }
            continue;
        }
        let base = lock.read_base(&key);
        let tracked = lock.files.contains_key(&key);
        let mut text = None;
        let action = match decide(existing.as_deref(), base.as_deref(), new, tracked, force) {
            Step::Do(Action::Saved) => {
                if !dry {
                    fs::write(format!("{}.mok-new", dest.display()), new)
                        .map_err(|e| e.to_string())?;
                }
                Action::Saved
            }
            Step::Do(action) => {
                if matches!(action, Action::New | Action::Updated | Action::Replaced) {
                    text = Some(new.clone());
                }
                action
            }
            Step::Merge => {
                let (merged, conflicts) = merge3(
                    root,
                    &lf(existing.as_deref().unwrap_or_default()),
                    &lf(base.as_deref().unwrap_or_default()),
                    &lf(new),
                )?;
                text = Some(merged);
                if conflicts == 0 {
                    Action::Merged
                } else {
                    Action::Conflict
                }
            }
        };
        // Markers an earlier merge left stay a conflict until someone fixes them.
        let unresolved = existing
            .as_deref()
            .is_some_and(|t| t.contains("<<<<<<< yours"));
        let action = if action == Action::Kept && unresolved {
            Action::Conflict
        } else {
            action
        };
        if !dry {
            if let Some(text) = &text {
                fs::create_dir_all(dest.parent().expect("a file has a folder"))
                    .map_err(|e| e.to_string())?;
                fs::write(&dest, text).map_err(|e| format!("{}: {e}", dest.display()))?;
            }
            if !matches!(action, Action::Kept | Action::Foreign | Action::Saved) {
                lock.write_base(&key, new)?;
                lock.files.insert(key.clone(), unit.clone());
            } else if action == Action::Kept {
                lock.files.insert(key.clone(), unit.clone());
            }
        }
        rows.push((key, action));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_nobody_touched_follows_the_source() {
        assert_eq!(
            decide(None, None, "new", false, false),
            Step::Do(Action::New)
        );
        assert_eq!(
            decide(Some("a"), Some("a"), "b", true, false),
            Step::Do(Action::Updated)
        );
        assert_eq!(
            decide(Some("b"), Some("a"), "b", true, false),
            Step::Do(Action::Current)
        );
    }

    #[test]
    fn an_edited_file_is_kept_or_merged_never_overwritten() {
        assert_eq!(
            decide(Some("mine"), Some("a"), "a", true, false),
            Step::Do(Action::Kept)
        );
        assert_eq!(
            decide(Some("mine"), Some("a"), "b", true, false),
            Step::Merge
        );
        assert_eq!(
            decide(Some("mine"), None, "b", true, false),
            Step::Do(Action::Saved)
        );
        assert_eq!(
            decide(Some("mine"), None, "b", false, false),
            Step::Do(Action::Foreign)
        );
        assert_eq!(
            decide(Some("mine"), Some("a"), "b", true, true),
            Step::Do(Action::Replaced)
        );
    }

    #[test]
    fn line_endings_do_not_make_an_edit() {
        assert_eq!(
            decide(Some("a\r\nb\r\n"), Some("a\nb\n"), "a\nc\n", true, false),
            Step::Do(Action::Updated)
        );
    }

    #[test]
    fn a_merge_takes_both_sides_or_marks_the_clash() {
        let dir = std::env::temp_dir().join(format!("mok-merge-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let base = "one\ntwo\nthree\nfour\nfive\nsix\nseven\n";
        let mine = "ONE\ntwo\nthree\nfour\nfive\nsix\nseven\n";
        let theirs = "one\ntwo\nthree\nfour\nfive\nsix\nSEVEN\n";
        let (clean, conflicts) = merge3(&dir, mine, base, theirs).unwrap();
        assert_eq!(
            (conflicts, clean.as_str()),
            (0, "ONE\ntwo\nthree\nfour\nfive\nsix\nSEVEN\n")
        );
        let (clash, conflicts) = merge3(
            &dir,
            mine,
            base,
            "uno\ntwo\nthree\nfour\nfive\nsix\nseven\n",
        )
        .unwrap();
        assert_eq!(conflicts, 1);
        assert!(clash.contains("<<<<<<< yours") && clash.contains(">>>>>>> new mok"));
        let _ = fs::remove_dir_all(&dir);
    }
}
