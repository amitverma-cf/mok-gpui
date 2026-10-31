//! One file as the project gets it: imports pointed at where each module now lives.

use std::collections::BTreeSet;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Item, UseTree};

use crate::graph::{CORE, Mp, Tree, is_test};

struct Edit {
    start: usize,
    end: usize,
    text: String,
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut v = vec![0];
    v.extend(text.match_indices('\n').map(|(i, _)| i + 1));
    v
}

fn offset(text: &str, starts: &[usize], at: proc_macro2::LineColumn) -> usize {
    let line = starts[at.line - 1];
    let rest = &text[line..];
    line + rest
        .char_indices()
        .nth(at.column)
        .map_or(rest.len(), |(i, _)| i)
}

fn range(text: &str, starts: &[usize], span: proc_macro2::Span) -> (usize, usize) {
    (
        offset(text, starts, span.start()),
        offset(text, starts, span.end()),
    )
}

fn print(tree: &UseTree) -> String {
    match tree {
        UseTree::Path(p) => format!("{}::{}", p.ident, print(&p.tree)),
        UseTree::Name(n) => n.ident.to_string(),
        UseTree::Rename(r) => format!("{} as {}", r.ident, r.rename),
        UseTree::Glob(_) => "*".into(),
        UseTree::Group(g) => format!(
            "{{{}}}",
            g.items.iter().map(print).collect::<Vec<_>>().join(", ")
        ),
    }
}

fn head(tree: &UseTree) -> Option<String> {
    match tree {
        UseTree::Path(p) => Some(p.ident.to_string()),
        UseTree::Name(n) => Some(n.ident.to_string()),
        UseTree::Rename(r) => Some(r.ident.to_string()),
        _ => None,
    }
}

fn prefix_for(name: &str, chapters: &BTreeSet<String>) -> &'static str {
    if chapters.contains(name) {
        "crate::ui"
    } else {
        "mok_gpui"
    }
}

/// `use crate::...` split between the copied chapters and the crate that stays.
fn rewrite_use(tree: &UseTree, chapters: &BTreeSet<String>) -> Option<Vec<String>> {
    let UseTree::Path(root) = tree else {
        return None;
    };
    if root.ident != "crate" {
        return None;
    }
    let entries: Vec<&UseTree> = match &*root.tree {
        UseTree::Group(g) => g.items.iter().collect(),
        other => vec![other],
    };
    let (mut ui, mut core) = (vec![], vec![]);
    for e in entries {
        let name = head(e).unwrap_or_default();
        if prefix_for(&name, chapters) == "crate::ui" {
            ui.push(print(e));
        } else {
            core.push(print(e));
        }
    }
    let mut out = vec![];
    for (prefix, list) in [("mok_gpui", core), ("crate::ui", ui)] {
        match list.len() {
            0 => {}
            1 => out.push(format!("{prefix}::{}", list[0])),
            _ => out.push(format!("{prefix}::{{{}}}", list.join(", "))),
        }
    }
    Some(out)
}

/// The use tree without the leaves `keep` refuses; `None` when none is left.
fn prune(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    keep: &dyn Fn(&[String]) -> bool,
) -> Option<UseTree> {
    match tree {
        UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            let inner = prune(&p.tree, prefix, keep);
            prefix.pop();
            inner.map(|tree| {
                let mut out = p.clone();
                out.tree = Box::new(tree);
                UseTree::Path(out)
            })
        }
        UseTree::Name(n) => {
            let mut path = prefix.clone();
            if n.ident != "self" {
                path.push(n.ident.to_string());
            }
            keep(&path).then(|| tree.clone())
        }
        UseTree::Rename(r) => {
            let mut path = prefix.clone();
            if r.ident != "self" {
                path.push(r.ident.to_string());
            }
            keep(&path).then(|| tree.clone())
        }
        UseTree::Glob(_) => keep(prefix).then(|| tree.clone()),
        UseTree::Group(g) => {
            let items: Vec<UseTree> = g
                .items
                .iter()
                .filter_map(|i| prune(i, prefix, keep))
                .collect();
            match items.len() {
                0 => None,
                _ => {
                    let mut out = g.clone();
                    out.items = items.into_iter().collect();
                    Some(UseTree::Group(out))
                }
            }
        }
    }
}

struct Uses<'a>(Vec<&'a syn::ItemUse>);

impl<'ast> Visit<'ast> for Uses<'ast> {
    fn visit_item_use(&mut self, u: &'ast syn::ItemUse) {
        self.0.push(u);
    }
}

/// One file as the project gets it.
pub fn transform(
    text: &str,
    unit: &str,
    installed: &BTreeSet<String>,
    chapters: &BTreeSet<String>,
    tree: &Tree,
    mp: &Mp,
) -> Result<String, String> {
    // The crate root re-exports the asset types; the copy keeps them in `assets`.
    let rooted = regex::Regex::new(r"\bcrate::(Assets|Before|ImageAsset|VideoAsset)\b")
        .expect("a fixed pattern")
        .replace_all(text, "crate::assets::$1")
        .into_owned();
    let text = rooted.as_str();
    let ast = syn::parse_file(text).map_err(|e| format!("{unit}: {e}"))?;
    let starts = line_starts(text);
    let mut edits: Vec<Edit> = vec![];
    let dir = if unit.ends_with("/mod.rs") {
        unit.trim_end_matches("/mod.rs").to_string()
    } else {
        unit.trim_end_matches(".rs").to_string()
    };
    let present = |name: &str| {
        installed.contains(&format!("{dir}/{name}.rs"))
            || installed.contains(&format!("{dir}/{name}/mod.rs"))
    };
    for item in &ast.items {
        let attrs = match item {
            Item::Mod(i) => &i.attrs,
            Item::Use(i) => &i.attrs,
            Item::Fn(i) => &i.attrs,
            Item::Impl(i) => &i.attrs,
            Item::Const(i) => &i.attrs,
            Item::Struct(i) => &i.attrs,
            Item::Enum(i) => &i.attrs,
            _ => continue,
        };
        let (start, end) = range(text, &starts, item.span());
        let drop = |edits: &mut Vec<Edit>| {
            edits.push(Edit {
                start,
                end,
                text: String::new(),
            });
        };
        if is_test(attrs) {
            drop(&mut edits);
            continue;
        }
        match item {
            Item::Mod(m) if m.content.is_none() && !present(&m.ident.to_string()) => {
                drop(&mut edits)
            }
            _ => {}
        }
    }
    let mut uses = Uses(vec![]);
    uses.visit_file(&ast);
    for u in uses.0 {
        let (start, end) = range(text, &starts, u.span());
        if edits.iter().any(|e| e.start <= start && end <= e.end) {
            continue;
        }
        let pruned = prune(&u.tree, &mut vec![], &|path: &[String]| {
            let Some(abs) = tree.absolute(mp, path) else {
                return true;
            };
            match tree.resolve(&abs) {
                Some(provider) if !provider.is_empty() && !CORE.contains(&provider[0].as_str()) => {
                    installed.contains(&tree.mods[&provider].unit)
                }
                _ => true,
            }
        });
        let Some(kept) = pruned else {
            edits.push(Edit {
                start,
                end,
                text: String::new(),
            });
            continue;
        };
        let changed = print(&kept) != print(&u.tree);
        let parts = rewrite_use(&kept, chapters);
        if !changed && parts.is_none() {
            continue;
        }
        let parts = parts.unwrap_or_else(|| vec![print(&kept)]);
        let lead = &text[start..range(text, &starts, u.use_token.span).0];
        let line = starts[u.span().start().line - 1];
        let indent: String = text[line..start]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect();
        let stmts: Vec<String> = parts.iter().map(|p| format!("{lead}use {p};")).collect();
        edits.push(Edit {
            start,
            end,
            text: stmts.join(&format!(
                "
{indent}"
            )),
        });
    }
    edits.sort_by_key(|e| e.start);
    let mut out = String::new();
    let mut at = 0;
    for e in &edits {
        if e.start < at {
            continue;
        }
        out.push_str(&paths(&text[at..e.start], chapters));
        out.push_str(&e.text);
        at = e.end;
    }
    out.push_str(&paths(&text[at..], chapters));
    Ok(out)
}

/// `crate::name` outside a `use`, pointed at where `name` now lives.
fn paths(text: &str, chapters: &BTreeSet<String>) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find("crate::") {
        let before = rest[..i].chars().next_back();
        let boundary = !before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '$');
        out.push_str(&rest[..i]);
        let after = &rest[i + 7..];
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if boundary && !name.is_empty() {
            out.push_str(prefix_for(&name, chapters));
            out.push_str("::");
        } else {
            out.push_str("crate::");
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chapters() -> BTreeSet<String> {
        ["forms", "buttons"].iter().map(|s| s.to_string()).collect()
    }

    fn tree(text: &str) -> UseTree {
        syn::parse_str::<syn::ItemUse>(text).unwrap().tree
    }

    #[test]
    fn a_path_in_code_points_at_the_crate_or_the_copy() {
        let out = paths("crate::theme::Palette and crate::forms::Input", &chapters());
        assert_eq!(out, "mok_gpui::theme::Palette and crate::ui::forms::Input");
        assert_eq!(paths("my_crate::forms", &chapters()), "my_crate::forms");
    }

    #[test]
    fn a_grouped_use_splits_between_the_crate_and_the_copy() {
        let parts = rewrite_use(
            &tree("use crate::{i18n, forms::Input, motion::Spinner};"),
            &chapters(),
        );
        assert_eq!(
            parts.unwrap(),
            [
                "mok_gpui::{i18n, motion::Spinner}",
                "crate::ui::forms::Input"
            ]
        );
        assert_eq!(rewrite_use(&tree("use gpui::App;"), &chapters()), None);
    }

    #[test]
    fn leaves_of_files_left_out_are_pruned() {
        let kept = prune(
            &tree("use buttons::{Button, Gone, Other};"),
            &mut vec![],
            &|p: &[String]| p.last().is_some_and(|l| l != "Gone"),
        );
        assert_eq!(print(&kept.unwrap()), "buttons::{Button, Other}");
        let none = prune(
            &tree("use buttons::Gone;"),
            &mut vec![],
            &|_: &[String]| false,
        );
        assert!(none.is_none());
    }
}
