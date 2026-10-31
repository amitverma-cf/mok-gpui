//! A hosted registry: `index.json` and a checksummed `source.tar.gz` that `cargo mok add` fetches in place of a git clone.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::source::revision;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn walk(base: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let Ok(read) = fs::read_dir(dir) else {
        return Ok(());
    };
    let mut entries: Vec<_> = read.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let path = e.path();
        if path.is_dir() {
            walk(base, &path, out)?;
        } else if path.is_file() {
            out.push(path.strip_prefix(base).map_err(|e| e.to_string())?.into());
        }
    }
    Ok(())
}

/// `cargo mok registry build`: writes `index.json` and `source.tar.gz` for `repo` into `out`.
pub fn build(repo: &Path, out: &Path) -> Result<(), String> {
    let registry = repo.join("registry.json");
    let text = fs::read_to_string(&registry).map_err(|e| format!("{}: {e}", registry.display()))?;
    let value: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let mut files = vec![PathBuf::from("registry.json")];
    for dir in ["src", "assets"] {
        walk(repo, &repo.join(dir), &mut files)?;
    }
    // Fonts and icons come from their pinned downloads, not from the bundle.
    files.retain(|f| !f.starts_with("assets/fonts") && !f.starts_with("assets/icons"));
    fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let mut tar = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::best()));
    tar.mode(tar::HeaderMode::Deterministic);
    for f in &files {
        tar.append_path_with_name(repo.join(f), f)
            .map_err(|e| format!("{}: {e}", f.display()))?;
    }
    let bytes = tar
        .into_inner()
        .and_then(|gz| gz.finish())
        .map_err(|e| e.to_string())?;
    fs::write(out.join("source.tar.gz"), &bytes).map_err(|e| e.to_string())?;
    let index = json!({
        "version": value["version"],
        "source": {
            "file": "source.tar.gz",
            "sha256": hex(&Sha256::digest(&bytes)),
            "bytes": bytes.len(),
            "revision": revision(repo),
            "files": files.len(),
        },
        "total_components": value["total_components"],
        "units": value["units"].as_object().map_or(0, |u| u.len()),
    });
    let text = serde_json::to_string_pretty(&index).map_err(|e| e.to_string())?;
    fs::write(out.join("index.json"), text + "\n").map_err(|e| e.to_string())?;
    println!(
        "[ok] {} files, {} bytes, into {}",
        files.len(),
        bytes.len(),
        out.display()
    );
    Ok(())
}

fn get(url: &str) -> Result<Vec<u8>, String> {
    let out = Command::new("curl")
        .args(["-fsSL", url])
        .output()
        .map_err(|e| format!("curl is needed to fetch {url}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "fetching {url} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}

/// Downloads the registry at `url` (a folder holding `index.json`), checks its checksum, and unpacks it.
pub fn fetch(url: &str) -> Result<PathBuf, String> {
    let base = url.trim_end_matches('/');
    let index: Value = serde_json::from_slice(&get(&format!("{base}/index.json"))?)
        .map_err(|e| format!("{base}/index.json: {e}"))?;
    let file = index["source"]["file"]
        .as_str()
        .ok_or("index.json names no source file")?;
    let want = index["source"]["sha256"]
        .as_str()
        .ok_or("index.json has no checksum")?;
    println!("[mok] Fetching the registry from {base}");
    let bytes = get(&format!("{base}/{file}"))?;
    let got = hex(&Sha256::digest(&bytes));
    if got != want {
        return Err(format!(
            "{base}/{file} does not match index.json (expected {want}, got {got})"
        ));
    }
    let dest = std::env::temp_dir().join(format!("mok-registry-{}", &got[..12]));
    if !dest.join("registry.json").is_file() {
        let _ = fs::remove_dir_all(&dest);
        tar::Archive::new(GzDecoder::new(&bytes[..]))
            .unpack(&dest)
            .map_err(|e| format!("unpacking {file}: {e}"))?;
    }
    Ok(dest)
}
