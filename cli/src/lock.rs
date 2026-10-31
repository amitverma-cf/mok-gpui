//! `mok.lock`: which files were copied, and the text mok last wrote for each, to merge updates against.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Lock {
    path: PathBuf,
    root: PathBuf,
    /// The folder the files were copied into, relative to the project.
    pub dir: String,
    /// The source commit the files were last made from, when known.
    pub source: String,
    /// The units asked for by name; the rest of `files` is what they need.
    pub requested: BTreeSet<String>,
    /// Project path of a copied file, to the unit it came from.
    pub files: BTreeMap<String, String>,
}

impl Lock {
    pub fn read(root: &Path, dir: &str) -> Lock {
        let path = root.join("mok.lock");
        let mut lock = Lock {
            path,
            root: root.to_path_buf(),
            dir: dir.to_string(),
            source: String::new(),
            requested: BTreeSet::new(),
            files: BTreeMap::new(),
        };
        if let Ok(text) = fs::read_to_string(&lock.path)
            && let Ok(value) = text.parse::<toml::Value>()
        {
            let get = |v: &toml::Value, k: &str| {
                v.get(k).and_then(|s| s.as_str()).unwrap_or("").to_string()
            };
            let saved = get(&value, "dir");
            if !saved.is_empty() {
                lock.dir = saved;
            }
            lock.source = get(&value, "source");
            lock.requested = value
                .get("requested")
                .and_then(|r| r.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            for f in value
                .get("file")
                .and_then(|f| f.as_array())
                .into_iter()
                .flatten()
            {
                lock.files.insert(get(f, "path"), get(f, "unit"));
            }
        }
        lock
    }

    pub fn units(&self) -> BTreeSet<String> {
        self.files.values().cloned().collect()
    }

    pub fn write(&self) -> Result<(), String> {
        let mut out = String::from(
            "# Written by `cargo mok`. Records what was copied, so updates keep your edits.\n",
        );
        out.push_str(&format!(
            "dir = {:?}\nsource = {:?}\n",
            self.dir, self.source
        ));
        let listed: Vec<&String> = self.requested.iter().collect();
        out.push_str(&format!("requested = {listed:?}\n"));
        for (path, unit) in &self.files {
            out.push_str(&format!("\n[[file]]\npath = {path:?}\nunit = {unit:?}\n"));
        }
        fs::write(&self.path, out).map_err(|e| format!("{}: {e}", self.path.display()))
    }

    fn base_path(&self, key: &str) -> PathBuf {
        self.root.join(".mok").join("base").join(key)
    }

    /// What mok wrote for the file last time: the common ancestor of an update's merge.
    pub fn read_base(&self, key: &str) -> Option<String> {
        fs::read_to_string(self.base_path(key)).ok()
    }

    pub fn write_base(&self, key: &str, text: &str) -> Result<(), String> {
        let path = self.base_path(key);
        fs::create_dir_all(path.parent().expect("a base file has a folder"))
            .map_err(|e| e.to_string())?;
        fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Keeps the scratch folders out of git, and the bases in.
    pub fn ignore_scratch(&self) -> Result<(), String> {
        let dir = self.root.join(".mok");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        fs::write(dir.join(".gitignore"), "stage/\ntmp/\n").map_err(|e| e.to_string())
    }
}
