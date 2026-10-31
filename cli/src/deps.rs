//! What a copy needs beside its source: crates in Cargo.toml and data files in assets/.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::source::{Options, Registry};

/// Keys of the dependencies a project's manifest already names, as code spells them.
fn present(root: &Path) -> BTreeSet<String> {
    let mut have = BTreeSet::new();
    let Ok(text) = fs::read_to_string(root.join("Cargo.toml")) else {
        return have;
    };
    let Ok(manifest) = text.parse::<toml::Value>() else {
        return have;
    };
    let mut tables = vec![manifest.get("dependencies").cloned()];
    if let Some(t) = manifest.get("target").and_then(|t| t.as_table()) {
        tables.extend(t.values().map(|b| b.get("dependencies").cloned()));
    }
    for table in tables.into_iter().flatten() {
        if let Some(t) = table.as_table() {
            for (key, spec) in t {
                have.insert(key.replace('-', "_"));
                if let Some(rename) = spec.get("package").and_then(|p| p.as_str()) {
                    let _ = rename;
                }
            }
        }
    }
    have
}

/// `cargo add` for every crate the copied files use that the project lacks.
pub fn add_crates(
    root: &Path,
    reg: &Registry,
    units: &BTreeSet<String>,
    _opts: &Options,
) -> Result<(), String> {
    let mut need: BTreeSet<String> = ["gpui".to_string()].into();
    for unit in units {
        need.extend(reg.crates_of.get(unit).into_iter().flatten().cloned());
    }
    let have = present(root);
    for name in need.iter().filter(|n| !have.contains(*n)) {
        let Some(spec) = reg.specs.get(name) else {
            continue;
        };
        let key = spec["key"].as_str().unwrap_or(name);
        let package = spec["package"].as_str().unwrap_or(key);
        let version = spec["version"].as_str().unwrap_or("*");
        let mut cmd = Command::new("cargo");
        cmd.current_dir(root)
            .arg("add")
            .arg(format!("{package}@{version}"));
        if package != key {
            cmd.args(["--rename", key]);
        }
        if spec["default_features"] == Value::Bool(false) {
            cmd.arg("--no-default-features");
        }
        let features: Vec<&str> = spec["features"]
            .as_array()
            .map(|a| a.iter().filter_map(|f| f.as_str()).collect())
            .unwrap_or_default();
        if !features.is_empty() {
            cmd.args(["--features", &features.join(",")]);
        }
        if let Some(target) = spec["target"].as_str() {
            cmd.args(["--target", target]);
        }
        run(cmd, key)?;
    }
    Ok(())
}

fn run(mut cmd: Command, what: &str) -> Result<(), String> {
    let out = cmd.output().map_err(|e| format!("cargo add {what}: {e}"))?;
    if out.status.success() {
        println!("[ok] cargo add {what}");
        Ok(())
    } else {
        Err(format!(
            "cargo add {what} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// Data files the copied code loads: into `assets/`, and declared under `[media.files]` so the build embeds them.
pub fn add_assets(
    root: &Path,
    source: &Path,
    reg: &Registry,
    units: &BTreeSet<String>,
) -> Result<(), String> {
    let wanted: BTreeSet<&String> = units
        .iter()
        .filter_map(|u| reg.assets_of.get(u))
        .flatten()
        .collect();
    if wanted.is_empty() {
        return Ok(());
    }
    let config_path = root.join("mok-gpui.toml");
    let mut config = fs::read_to_string(&config_path)
        .map_err(|_| "mok-gpui.toml not found; run `cargo mok init` first".to_string())?;
    for file in wanted {
        let dest = root.join("assets").join(file);
        if !dest.is_file() {
            let from = source.join("assets").join(file);
            fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
            fs::copy(&from, &dest).map_err(|e| format!("{}: {e}", from.display()))?;
            println!("[ok] assets/{file}");
        }
        if !config.contains(&format!("path = {file:?}")) {
            let key = Path::new(file)
                .file_stem()
                .map_or_else(|| "file".into(), |s| s.to_string_lossy().replace('-', "_"));
            let line = format!("{key} = {{ path = {file:?} }}");
            match config.find("[media.files]") {
                Some(at) => {
                    let end = at + "[media.files]".len();
                    config.insert_str(
                        end,
                        &format!(
                            "
{line}"
                        ),
                    );
                }
                None => config.push_str(&format!(
                    "
[media.files]
{line}
"
                )),
            }
            println!("[ok] mok-gpui.toml: [media.files] {key}");
        }
    }
    fs::write(&config_path, config).map_err(|e| e.to_string())
}
