//! `cargo mok registry`: rebuilds `registry.json`, each entry pointing at the file that defines it.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use regex::Regex;
use serde_json::{Map, Value, json};

use crate::graph::{self, CORE, Tree};
use crate::items::{Found, elements, items, sentence};
use crate::notes::{key, key_ident, snake, tasks};

/// Gallery page slug to source folder, where they differ.
fn folder(slug: &str) -> &str {
    match slug {
        "collaboration" => "collab",
        "monitoring" => "dashboard",
        "a11y" => "i18n",
        "data" => "data_display",
        other => other,
    }
}

/// How many files a unit needs to compile, itself and the modules up its path included.
fn closure(units: &Map<String, Value>, start: &str) -> usize {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut work = vec![start.to_string()];
    while let Some(id) = work.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if let Some(u) = units.get(&id) {
            work.extend(
                u["requires"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str().map(String::from)),
            );
        }
        let parts: Vec<&str> = id.split('/').collect();
        for depth in 1..parts.len() {
            let parent = format!("{}/mod.rs", parts[..depth].join("/"));
            if parent != id && units.contains_key(&parent) {
                work.push(parent);
            }
        }
    }
    seen.len()
}

/// The file graph: what each copyable file needs.
fn file_graph(tree: &Tree, deps: &Map<String, Value>) -> Map<String, Value> {
    let mut units = Map::new();
    let mut extra: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (mp, m) in &tree.mods {
        let Some(chapter) = mp.first() else { continue };
        if !tree.is_chapter(chapter) {
            continue;
        }
        let needs = graph::needs(tree, mp);
        for target in &needs.extends {
            extra
                .entry(target.clone())
                .or_default()
                .push(m.unit.clone());
        }
        units.insert(
            m.unit.clone(),
            json!({
                "requires": needs.requires,
                "core": needs.core,
                "assets": needs.assets,
                "crates": needs.crates.iter().filter(|c| deps.contains_key(*c)).collect::<Vec<_>>(),
            }),
        );
    }
    for (target, adds) in extra {
        if let Some(unit) = units.get_mut(&target) {
            let list = unit["requires"].as_array_mut().unwrap();
            for a in adds {
                if !list.iter().any(|v| v == &Value::String(a.clone())) {
                    list.push(Value::String(a));
                }
            }
        }
    }
    units
}

/// One registry entry before it is resolved against the source.
struct Draft<'a> {
    name: String,
    display: &'a str,
    found: Option<&'a Found>,
    desc: String,
    native: Option<&'a str>,
    feature: bool,
}

struct Context<'a> {
    tree: &'a Tree,
    units: &'a Map<String, Value>,
}

impl Context<'_> {
    fn entry(&self, page: &Value, chapter: &str, d: Draft) -> Value {
        let Draft {
            name,
            display,
            found,
            desc,
            native,
            feature,
        } = d;
        let (status, unit, dir, file) = match found {
            Some(f) => {
                let unit = self.tree.mods[&f.mp].unit.clone();
                let (dir, file) = unit
                    .rsplit_once('/')
                    .map_or(("", unit.as_str()), |(d, f)| (d, f));
                let core = CORE.contains(&f.mp[0].as_str());
                // A part of something the crate keeps is the crate's too.
                let status = if core {
                    "core"
                } else if feature {
                    "feature"
                } else {
                    "component"
                };
                (
                    status,
                    Some(unit.clone()),
                    dir.to_string(),
                    file.to_string(),
                )
            }
            None => (
                native.unwrap_or("unmapped"),
                None,
                chapter.to_string(),
                String::new(),
            ),
        };
        let mut v = json!({
            "name": name,
            "display_name": display,
            "chapter": page["number"],
            "chapter_title": page["title"],
            "category": page["slug"],
            "source_dir": format!("{}/{dir}", crate::source::LIB_DIR),
            "file": file,
            "export_type": found.map_or_else(|| key_ident(display), |f| f.ident.clone()),
            "description": desc,
            "dependencies": [],
            "status": status,
        });
        if let Some(f) = found {
            let root = self.tree.mods.get(&vec![f.mp[0].clone()]);
            let head = if CORE.contains(&f.mp[0].as_str()) {
                "mok_gpui"
            } else {
                "crate::ui"
            };
            let reached = root.is_some_and(|r| {
                r.reexports.contains_key(&f.ident) || r.defines.contains(&f.ident)
            });
            let path = if reached {
                f.mp[0].clone()
            } else {
                f.mp.join("::")
            };
            v["import"] = json!(format!("{head}::{path}::{}", f.ident));
        }
        if let Some(u) = unit {
            if status == "component" || status == "feature" {
                v["files"] = json!(closure(self.units, &u));
            }
            v["unit"] = json!(u);
        }
        v
    }
}

/// Writes `registry.json`; with `check`, only says whether it is current.
pub fn generate(repo: &Path, check: bool) -> Result<(), String> {
    let tree = Tree::load(&repo.join(crate::source::LIB_DIR))?;
    let deps = dependencies(repo)?;
    let units = file_graph(&tree, &deps);
    let cx = Context {
        tree: &tree,
        units: &units,
    };

    let stories: Value = serde_json::from_str(
        &fs::read_to_string(repo.join("examples/gallery/stories.json"))
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let notes = tasks(repo)?;
    let index = items(&tree);
    let mut lower: BTreeMap<String, Vec<&Found>> = BTreeMap::new();
    for f in index.values().flatten() {
        lower.entry(key(&f.ident)).or_default().push(f);
    }
    let pick = |ident: &str, chapter: &str| -> Option<&Found> {
        let pool: Vec<&Found> = match index.get(ident) {
            Some(v) => v.iter().collect(),
            None => lower.get(&key(ident)).cloned()?,
        };
        pool.iter()
            .find(|f| f.mp.first().is_some_and(|c| c == chapter))
            .or_else(|| pool.first())
            .copied()
    };

    let paren = Regex::new(r"\s*\([^)]*\)").unwrap();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut taken: BTreeSet<String> = BTreeSet::new();
    let mut slugs: BTreeSet<String> = BTreeSet::new();
    let mut components: Vec<Value> = vec![];
    let mut chapters: Vec<Value> = vec![];
    let mut missing: Vec<String> = vec![];
    let pages = stories["pages"]
        .as_array()
        .ok_or("stories.json has no pages")?;

    for page in pages {
        let slug = page["slug"].as_str().unwrap_or("");
        let chapter = folder(slug).to_string();
        let title = page["title"].as_str().unwrap_or("");
        let mut names: Vec<String> = vec![];
        for story in page["stories"].as_array().into_iter().flatten() {
            for raw in story["components"].as_array().into_iter().flatten() {
                let clean = paren
                    .replace_all(raw.as_str().unwrap_or(""), "")
                    .trim()
                    .to_string();
                if clean.is_empty() || !seen.insert(clean.clone()) || !slugs.insert(snake(&clean)) {
                    continue;
                }
                names.push(clean.clone());
                let note = notes.get(&key(&clean));
                // `→ Type::method` and `→ module::Type`: the first segment that names a type.
                let hosted = note.and_then(|n| {
                    let segs: Vec<&str> = n.target.split("::").collect();
                    let types = segs
                        .iter()
                        .filter(|s| s.starts_with(|c: char| c.is_ascii_uppercase()));
                    let others = segs
                        .iter()
                        .filter(|s| !s.starts_with(|c: char| c.is_ascii_uppercase()));
                    types.chain(others).find_map(|seg| pick(seg, &chapter))
                });
                let own = pick(&key_ident(&clean), &chapter);
                let sibling = note.and_then(|n| n.group.iter().find_map(|g| pick(g, &chapter)));
                let hosted = hosted.or(sibling);
                let found = own.or(hosted);
                let feature = own.is_none() && hosted.is_some();
                let prose = note.map(|n| n.desc.clone()).filter(|d| !d.is_empty());
                let arrow = note.map(|n| n.arrow.clone()).filter(|d| !d.is_empty());
                let lacks = prose.as_deref().or(arrow.as_deref()).and_then(|d| {
                    ["native", "blocked", "host"]
                        .into_iter()
                        .find(|word| d.contains(word))
                });
                let desc = prose
                    .or_else(|| match (found, feature) {
                        (Some(f), true) => Some(match &arrow {
                            Some(a) => format!("Part of {}: {a}", f.ident),
                            None => format!("Part of {}", f.ident),
                        }),
                        (Some(f), false) => Some(sentence(&f.doc)).filter(|d| !d.is_empty()),
                        (None, _) => None,
                    })
                    .or(arrow)
                    .unwrap_or_else(|| format!("{clean} component in {title}"));
                match found {
                    Some(f) => {
                        taken.insert(f.ident.clone());
                    }
                    None if lacks.is_none() => missing.push(clean.clone()),
                    None => {}
                }
                components.push(cx.entry(
                    page,
                    &chapter,
                    Draft {
                        name: snake(&clean),
                        display: &clean,
                        found,
                        desc,
                        native: lacks,
                        feature,
                    },
                ));
            }
        }
        // Public components the gallery's stories do not name.
        for (mp, ident, doc) in elements(&tree) {
            if mp[0] != chapter || taken.contains(&ident) || !slugs.insert(snake(&ident)) {
                continue;
            }
            let Some(found) = pick(&ident, &chapter) else {
                continue;
            };
            taken.insert(ident.clone());
            names.push(ident.clone());
            let desc = Some(sentence(&doc))
                .filter(|d| !d.is_empty())
                .unwrap_or_else(|| format!("{ident} component in {title}"));
            components.push(cx.entry(
                page,
                &chapter,
                Draft {
                    name: snake(&ident),
                    display: &ident,
                    found: Some(found),
                    desc,
                    native: None,
                    feature: false,
                },
            ));
        }
        chapters.push(json!({
            "number": page["number"],
            "name": slug,
            "title": page["title"],
            "summary": page["summary"],
            "source_dir": format!("{}/{chapter}", crate::source::LIB_DIR),
            "component_count": names.len(),
            "components": names,
        }));
    }

    let count = |status: &str| components.iter().filter(|c| c["status"] == status).count();
    let tally = (
        count("component"),
        count("core"),
        count("feature"),
        count("native") + count("blocked") + count("host"),
        count("unmapped"),
    );
    let total = components.len();
    let registry = json!({
        "version": "2.0.0",
        "total_components": total,
        "total_chapters": chapters.len(),
        "chapters": chapters,
        "components": components,
        "core": CORE,
        "crates": Value::Object(deps),
        "units": Value::Object(units),
    });
    let path = repo.join("registry.json");
    let text = serde_json::to_string_pretty(&registry).unwrap() + "\n";
    if check {
        let current = fs::read_to_string(&path).unwrap_or_default();
        if current.replace("\r\n", "\n") != text {
            return Err("registry.json is stale: run `cargo mok registry` in the repo".into());
        }
        println!("[ok] registry.json is current ({total} components)");
        return Ok(());
    }
    fs::write(&path, text).map_err(|e| e.to_string())?;
    println!(
        "[ok] {total} components in {} chapters: {} copyable, {} in the mok-gpui crate, {} features of another component, {} native, blocked or host-provided, {} unmapped",
        pages.len(),
        tally.0,
        tally.1,
        tally.2,
        tally.3,
        tally.4
    );
    if std::env::args().any(|a| a == "--list") {
        for name in &missing {
            println!("[unmapped] {name}");
        }
    }
    Ok(())
}

/// The library's dependencies as `cargo add` needs them, by the name code uses.
fn dependencies(repo: &Path) -> Result<Map<String, Value>, String> {
    let text = fs::read_to_string(repo.join("Cargo.toml")).map_err(|e| e.to_string())?;
    let manifest: toml::Value = text.parse().map_err(|e: toml::de::Error| e.to_string())?;
    let mut out = Map::new();
    let mut take = |table: &toml::Value, target: Option<&str>| {
        let Some(table) = table.as_table() else {
            return;
        };
        for (key, spec) in table {
            let (version, package, features, default) = match spec {
                toml::Value::String(v) => (v.clone(), None, vec![], true),
                toml::Value::Table(t) => (
                    t.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("*")
                        .to_string(),
                    t.get("package").and_then(|v| v.as_str()).map(String::from),
                    t.get("features")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|f| f.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    t.get("default-features")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                ),
                _ => continue,
            };
            let name = key.replace('-', "_");
            out.entry(name).or_insert(json!({
                "key": key, "version": version, "package": package,
                "features": features, "default_features": default, "target": target,
            }));
        }
    };
    if let Some(t) = manifest.get("dependencies") {
        take(t, None);
    }
    if let Some(targets) = manifest.get("target").and_then(|t| t.as_table()) {
        for (cfg, body) in targets {
            if let Some(t) = body.get("dependencies") {
                take(t, Some(cfg));
            }
        }
    }
    Ok(out)
}
