//! Where the library's source is, and what its registry says about each file.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

pub struct Options {
    pub from: Option<PathBuf>,
    pub dir: PathBuf,
    pub force: bool,
}

pub const DEFAULT_SOURCE: &str = "https://github.com/amitverma-cf/mok-gpui";

/// Where the library's source is: a given checkout, the current one, or a fresh shallow clone.
pub fn source_root(from: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(dir) = from {
        return Ok(dir.to_path_buf());
    }
    let here = Path::new(".");
    if here.join("registry.json").is_file() && here.join("src/lib.rs").is_file() {
        return Ok(here.to_path_buf());
    }
    if let Ok(url) = std::env::var("MOK_REGISTRY_URL") {
        return crate::bundle::fetch(&url);
    }
    let url = std::env::var("MOK_SOURCE_URL").unwrap_or_else(|_| DEFAULT_SOURCE.into());
    let cache = std::env::temp_dir().join(format!("mok-source-{}", std::process::id()));
    let _ = fs::remove_dir_all(&cache);
    println!("[mok] Fetching the library source from {url}");
    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--quiet", &url])
        .arg(&cache)
        .status()
        .map_err(|e| format!("git is needed to fetch the source: {e}"))?;
    if !status.success() {
        return Err(format!(
            "git clone {url} failed; pass --from <checkout> to use a local copy"
        ));
    }
    Ok(cache)
}

/// The commit a checkout is at, or empty when it is no git checkout.
pub fn revision(source: &Path) -> String {
    Command::new("git")
        .arg("-C")
        .arg(source)
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_default()
}

pub struct Registry {
    pub assets_of: BTreeMap<String, Vec<String>>,
    pub units: BTreeMap<String, Vec<String>>,
    pub crates_of: BTreeMap<String, Vec<String>>,
    pub specs: Value,
    pub chapters: BTreeSet<String>,
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

pub fn load(root: &Path) -> Result<Registry, String> {
    let path = root.join("registry.json");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let value: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let units = value["units"]
        .as_object()
        .ok_or("registry.json has no file graph; run `cargo mok registry` in the library")?;
    let mut reg = Registry {
        assets_of: BTreeMap::new(),
        units: BTreeMap::new(),
        crates_of: BTreeMap::new(),
        specs: value["crates"].clone(),
        chapters: BTreeSet::new(),
    };
    for (id, u) in units {
        if let Some(chapter) = id.split('/').next() {
            reg.chapters.insert(chapter.to_string());
        }
        reg.crates_of.insert(id.clone(), strings(&u["crates"]));
        let assets = strings(&u["assets"]);
        if !assets.is_empty() {
            reg.assets_of.insert(id.clone(), assets);
        }
        reg.units.insert(id.clone(), strings(&u["requires"]));
    }
    Ok(reg)
}

/// The units a unit needs to compile: itself, its requirements, and every module up its path.
pub fn closure(reg: &Registry, start: &[String]) -> Result<BTreeSet<String>, String> {
    let mut set = BTreeSet::new();
    let mut work: Vec<String> = start.to_vec();
    while let Some(id) = work.pop() {
        if !set.insert(id.clone()) {
            continue;
        }
        let needs = reg
            .units
            .get(&id)
            .ok_or_else(|| format!("registry has no unit {id}"))?;
        work.extend(needs.iter().cloned());
        let parts: Vec<&str> = id.split('/').collect();
        for depth in 1..parts.len() {
            let parent = format!("{}/mod.rs", parts[..depth].join("/"));
            if parent != id && reg.units.contains_key(&parent) {
                work.push(parent);
            }
        }
    }
    Ok(set)
}
