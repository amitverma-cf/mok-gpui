//! Resolves every `crate::` path through the library's modules to a per-file dependency graph.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use quote::ToTokens;
use syn::visit::Visit;
use syn::{Attribute, Item, UseTree};

/// Modules `cargo mok add` leaves in a crate. None: the foundation is copied like the rest.
pub const CORE: &[&str] = &[];

pub type Mp = Vec<String>;

pub struct Module {
    pub unit: String,
    pub children: BTreeMap<String, Mp>,
    pub reexports: BTreeMap<String, Mp>,
    pub globs: Vec<Mp>,
    pub defines: BTreeSet<String>,
    pub ast: syn::File,
}

pub struct Tree {
    pub mods: BTreeMap<Mp, Module>,
}

pub fn is_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg") && {
            let text = a.meta.to_token_stream().to_string();
            text.split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|w| w == "test")
        }
    })
}

/// `(path, name)` for every leaf of a use tree; a glob's name is `*`.
pub fn flatten(tree: &UseTree, prefix: &mut Vec<String>, out: &mut Vec<(Vec<String>, String)>) {
    match tree {
        UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            flatten(&p.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(n) => {
            let id = n.ident.to_string();
            if id == "self" {
                let name = prefix.last().cloned().unwrap_or_default();
                out.push((prefix.clone(), name));
            } else {
                let mut path = prefix.clone();
                path.push(id.clone());
                out.push((path, id));
            }
        }
        UseTree::Rename(r) => {
            let id = r.ident.to_string();
            let mut path = prefix.clone();
            if id != "self" {
                path.push(id);
            }
            out.push((path, r.rename.to_string()));
        }
        UseTree::Glob(_) => out.push((prefix.clone(), "*".into())),
        UseTree::Group(g) => {
            for item in &g.items {
                flatten(item, prefix, out);
            }
        }
    }
}

impl Tree {
    pub fn load(src: &Path) -> Result<Tree, String> {
        let mut tree = Tree {
            mods: BTreeMap::new(),
        };
        tree.read(src, vec![], &src.join("lib.rs"), src)?;
        for _ in 0..6 {
            if !tree.link() {
                break;
            }
        }
        Ok(tree)
    }

    fn read(&mut self, src: &Path, mp: Mp, file: &Path, dir: &Path) -> Result<(), String> {
        let text = fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
        let ast = syn::parse_file(&text).map_err(|e| format!("{}: {e}", file.display()))?;
        let unit = file
            .strip_prefix(src)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let mut module = Module {
            unit,
            children: BTreeMap::new(),
            reexports: BTreeMap::new(),
            globs: vec![],
            defines: BTreeSet::new(),
            ast: ast.clone(),
        };
        let mut pending = vec![];
        for item in &ast.items {
            match item {
                Item::Mod(m) if m.content.is_none() && !is_test(&m.attrs) => {
                    let name = m.ident.to_string();
                    let mut child = mp.clone();
                    child.push(name.clone());
                    module.children.insert(name.clone(), child.clone());
                    let flat = dir.join(format!("{name}.rs"));
                    let nested = dir.join(&name).join("mod.rs");
                    let (path, sub) = if flat.is_file() {
                        (flat, dir.join(&name))
                    } else if nested.is_file() {
                        (nested, dir.join(&name))
                    } else {
                        return Err(format!("{}: no file for `mod {name}`", file.display()));
                    };
                    pending.push((child, path, sub));
                }
                Item::Mod(m) => {
                    module.defines.insert(m.ident.to_string());
                }
                Item::Fn(i) => {
                    module.defines.insert(i.sig.ident.to_string());
                }
                Item::Struct(i) => {
                    module.defines.insert(i.ident.to_string());
                }
                Item::Enum(i) => {
                    module.defines.insert(i.ident.to_string());
                }
                Item::Union(i) => {
                    module.defines.insert(i.ident.to_string());
                }
                Item::Const(i) => {
                    module.defines.insert(i.ident.to_string());
                }
                Item::Static(i) => {
                    module.defines.insert(i.ident.to_string());
                }
                Item::Type(i) => {
                    module.defines.insert(i.ident.to_string());
                }
                Item::Trait(i) => {
                    module.defines.insert(i.ident.to_string());
                }
                Item::Macro(i) => {
                    if let Some(id) = &i.ident {
                        module.defines.insert(id.to_string());
                    }
                }
                _ => {}
            }
        }
        self.mods.insert(mp, module);
        for (child, path, sub) in pending {
            self.read(src, child, &path, &sub)?;
        }
        Ok(())
    }

    /// Makes a path absolute (from the crate root); `None` for external crates.
    pub fn absolute(&self, from: &Mp, segs: &[String]) -> Option<Vec<String>> {
        let first = segs.first()?;
        match first.as_str() {
            "crate" => Some(segs[1..].to_vec()),
            "self" => Some(from.iter().chain(&segs[1..]).cloned().collect()),
            "super" => {
                let mut base = from.clone();
                let mut rest = segs;
                while rest.first().map(String::as_str) == Some("super") {
                    base.pop()?;
                    rest = &rest[1..];
                }
                Some(base.into_iter().chain(rest.iter().cloned()).collect())
            }
            s => {
                let here = self.mods.get(from)?;
                (here.children.contains_key(s) || here.defines.contains(s))
                    .then(|| from.iter().chain(segs).cloned().collect())
            }
        }
    }

    /// The module that provides what an absolute path names.
    pub fn resolve(&self, abs: &[String]) -> Option<Mp> {
        self.resolve_from(vec![], abs)
    }

    fn resolve_from(&self, start: Mp, abs: &[String]) -> Option<Mp> {
        let mut cur = start;
        let mut last = None;
        for (i, s) in abs.iter().enumerate() {
            let m = self.mods.get(&cur)?;
            if let Some(c) = m.children.get(s) {
                cur = c.clone();
                last = Some(cur.clone());
            } else if let Some(t) = m.reexports.get(s) {
                return Some(t.clone());
            } else if m.defines.contains(s) {
                return Some(cur);
            } else {
                for g in &m.globs {
                    if let Some(found) = self.resolve_from(g.clone(), &abs[i..])
                        && self.mods.get(g).is_some_and(|gm| {
                            gm.children.contains_key(s)
                                || gm.reexports.contains_key(s)
                                || gm.defines.contains(s)
                        })
                    {
                        return Some(found);
                    }
                }
                return last;
            }
        }
        last
    }

    /// Fills each module's re-exports; true while it still learns something.
    fn link(&mut self) -> bool {
        let mut learned: Vec<(Mp, String, Mp)> = vec![];
        let mut globs: Vec<(Mp, Mp)> = vec![];
        for (mp, m) in &self.mods {
            for item in &m.ast.items {
                let Item::Use(u) = item else { continue };
                let mut leaves = vec![];
                flatten(&u.tree, &mut vec![], &mut leaves);
                for (path, name) in leaves {
                    let Some(abs) = self.absolute(mp, &path) else {
                        continue;
                    };
                    let Some(target) = self.resolve(&abs) else {
                        continue;
                    };
                    if name == "*" {
                        if let Some(t) = self.mods.get(&abs).map(|_| abs.clone()).or(Some(target)) {
                            globs.push((mp.clone(), t));
                        }
                    } else {
                        learned.push((mp.clone(), name, target));
                    }
                }
            }
        }
        let mut changed = false;
        for (mp, name, target) in learned {
            let m = self.mods.get_mut(&mp).unwrap();
            if target != mp && m.reexports.get(&name) != Some(&target) {
                m.reexports.insert(name, target);
                changed = true;
            }
        }
        for (mp, glob) in globs {
            let m = self.mods.get_mut(&mp).unwrap();
            if glob != mp && !m.globs.contains(&glob) {
                m.globs.push(glob);
                changed = true;
            }
        }
        changed
    }

    /// Chapters that are copied as source: the crate's modules outside `CORE`.
    pub fn is_chapter(&self, name: &str) -> bool {
        !CORE.contains(&name)
            && self
                .mods
                .get(&vec![])
                .is_some_and(|root| root.children.contains_key(name))
    }
}

pub struct Needs {
    pub requires: BTreeSet<String>,
    pub core: BTreeSet<String>,
    /// Roots of paths that name no module of the crate: external crates, or std.
    pub crates: BTreeSet<String>,
    /// Units whose types this file implements methods or traits for.
    pub extends: BTreeSet<String>,
    /// Data files the code loads by path, such as `maps/countries.geojson`.
    pub assets: BTreeSet<String>,
}

struct Collect {
    assets: BTreeSet<String>,
    aliases: BTreeMap<String, Vec<String>>,
    paths: Vec<Vec<String>>,
}

impl<'ast> Visit<'ast> for Collect {
    fn visit_item_use(&mut self, u: &'ast syn::ItemUse) {
        let mut leaves = vec![];
        flatten(&u.tree, &mut vec![], &mut leaves);
        for (path, name) in leaves {
            if name != "*" {
                self.aliases.insert(name, path.clone());
            }
            self.paths.push(path);
        }
    }

    fn visit_lit_str(&mut self, lit: &'ast syn::LitStr) {
        let value = lit.value();
        let data = ["maps/"].iter().any(|dir| value.starts_with(dir))
            && value
                .rsplit('.')
                .next()
                .is_some_and(|ext| ext.len() <= 8 && !ext.contains('/'))
            && !value.contains(char::is_whitespace);
        if data {
            self.assets.insert(value);
        }
    }

    fn visit_path(&mut self, p: &'ast syn::Path) {
        let segs: Vec<String> = p.segments.iter().map(|s| s.ident.to_string()).collect();
        self.paths.push(segs);
        syn::visit::visit_path(self, p);
    }
}

/// What the unit's file needs, outside itself.
pub fn needs(tree: &Tree, mp: &Mp) -> Needs {
    let module = &tree.mods[mp];
    let mut collect = Collect {
        assets: BTreeSet::new(),
        aliases: BTreeMap::new(),
        paths: vec![],
    };
    // Items of a mod.rs that only declare or re-export need nothing.
    let body: Vec<&Item> = module
        .ast
        .items
        .iter()
        .filter(|i| !is_test_item(i))
        .filter(|i| {
            let is_mod_rs = module.unit.ends_with("mod.rs");
            !(is_mod_rs && matches!(i, Item::Mod(_) | Item::Use(_)))
        })
        .collect();
    for item in &body {
        collect.visit_item(item);
    }
    // A mod.rs's own `use` lines need nothing, but the code in it may use the names they bring in.
    for item in &module.ast.items {
        if let Item::Use(_) = item
            && module.unit.ends_with("mod.rs")
            && !is_test_item(item)
        {
            let mut aside = Collect {
                assets: BTreeSet::new(),
                aliases: BTreeMap::new(),
                paths: vec![],
            };
            aside.visit_item(item);
            collect.aliases.extend(aside.aliases);
        }
    }
    let aliases = collect.aliases.clone();
    let mut out = Needs {
        requires: BTreeSet::new(),
        core: BTreeSet::new(),
        crates: BTreeSet::new(),
        extends: BTreeSet::new(),
        assets: collect.assets.clone(),
    };
    for item in &body {
        let Item::Impl(imp) = item else { continue };
        let syn::Type::Path(ty) = &*imp.self_ty else {
            continue;
        };
        let segs: Vec<String> = ty
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        let mut full = segs.clone();
        if let Some(target) = collect.aliases.get(&segs[0]) {
            full = target.iter().chain(&segs[1..]).cloned().collect();
        }
        if let Some(abs) = tree.absolute(mp, &full)
            && !CORE.contains(&abs.first().map_or("", String::as_str))
            && let Some(provider) = tree.resolve(&abs)
            && !provider.is_empty()
            && &provider != mp
            && let Some(m) = tree.mods.get(&provider)
        {
            out.extends.insert(m.unit.clone());
        }
    }
    for path in &collect.paths {
        let mut full = path.clone();
        if let Some(first) = path.first()
            && let Some(target) = aliases.get(first)
            && !matches!(first.as_str(), "crate" | "self" | "super")
        {
            full = target.iter().chain(&path[1..]).cloned().collect();
        }
        let Some(abs) = tree.absolute(mp, &full) else {
            if full.len() > 1 && full[0].chars().next().is_some_and(char::is_lowercase) {
                out.crates.insert(full[0].clone());
            }
            continue;
        };
        let Some(first) = abs.first() else { continue };
        if CORE.contains(&first.as_str()) {
            out.core.insert(first.clone());
            continue;
        }
        if let Some(provider) = tree.resolve(&abs)
            && !provider.is_empty()
            && !CORE.contains(&provider[0].as_str())
            && let Some(m) = tree.mods.get(&provider)
            && &provider != mp
        {
            out.requires.insert(m.unit.clone());
        }
    }
    out
}

fn is_test_item(item: &Item) -> bool {
    let attrs: &[Attribute] = match item {
        Item::Mod(i) => &i.attrs,
        Item::Fn(i) => &i.attrs,
        Item::Use(i) => &i.attrs,
        Item::Const(i) => &i.attrs,
        Item::Impl(i) => &i.attrs,
        Item::Struct(i) => &i.attrs,
        Item::Enum(i) => &i.attrs,
        _ => return false,
    };
    is_test(attrs)
}
