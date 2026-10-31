//! Project commands: `new`, `build`, `run`, `package` and `check`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::source::Options;

const MAIN: &str = include_str!("../templates/main.rs");
const INDEX: &str = include_str!("../templates/index.html");
const WEB_TARGET: &str = "wasm32-unknown-unknown";
const GPUI: &str = "=0.3.9";

fn run(mut cmd: Command, what: &str) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{what}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{what} failed"))
    }
}

fn cargo_add(dir: &Path, args: &[&str]) -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(dir).arg("add").args(args);
    run(cmd, &format!("cargo add {}", args.join(" ")))
}

/// `cargo mok new <name>`: an app that runs natively and in a browser, with Mok's button.
pub fn new(parent: &Path, name: &str, opts: &Options) -> Result<(), String> {
    let dir = parent.join(name);
    let canon;
    let opts = match &opts.from {
        Some(f) => {
            canon = Options {
                from: Some(fs::canonicalize(f).map_err(|e| format!("{}: {e}", f.display()))?),
                dir: opts.dir.clone(),
                force: opts.force,
            };
            &canon
        }
        None => opts,
    };
    if dir.exists() {
        return Err(format!("{} exists already", dir.display()));
    }
    let mut cmd = Command::new("cargo");
    cmd.current_dir(parent)
        .args(["new", "--bin", "--quiet", name]);
    run(cmd, "cargo new")?;
    let gpui = format!("gpui-pre@{GPUI}");
    cargo_add(
        &dir,
        &[
            &gpui,
            "--rename",
            "gpui",
            "--no-default-features",
            "--features",
            "stacker",
        ],
    )?;
    let platform = format!("gpui-pre-platform@{GPUI}");
    cargo_add(
        &dir,
        &[
            &platform,
            "--rename",
            "gpui_platform",
            "--features",
            "font-kit,runtime_shaders",
        ],
    )?;
    let web = format!("gpui-pre-web@{GPUI}");
    cargo_add(
        &dir,
        &[
            &web,
            "--rename",
            "gpui_web",
            "--target",
            "cfg(target_family = \"wasm\")",
        ],
    )?;
    cargo_add(
        &dir,
        &["wasm-bindgen", "--target", "cfg(target_family = \"wasm\")"],
    )?;
    fs::write(dir.join("src/main.rs"), MAIN).map_err(|e| e.to_string())?;
    std::env::set_current_dir(&dir).map_err(|e| e.to_string())?;
    crate::init(false, true)?;
    crate::component::scaffold_components(
        &[
            "button".to_string(),
            "heading".to_string(),
            "paragraph".to_string(),
        ],
        &dir,
        opts,
    )?;
    println!(
        "\n[ok] {name} is ready: cd {name} && cargo run   (cargo mok run --target web for the browser)"
    );
    Ok(())
}

fn package_name(root: &Path) -> Result<String, String> {
    let text = fs::read_to_string(root.join("Cargo.toml")).map_err(|e| e.to_string())?;
    let manifest: toml::Value = text.parse().map_err(|e: toml::de::Error| e.to_string())?;
    manifest["package"]["name"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "Cargo.toml has no package name".into())
}

fn have(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// `cargo mok build --target web`: wasm, wasm-bindgen, wasm-opt and an index.html in `dist/web`.
fn build_web(root: &Path, release: bool) -> Result<PathBuf, String> {
    let name = package_name(root)?;
    let installed = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map_err(|e| format!("rustup: {e}"))?;
    if !String::from_utf8_lossy(&installed.stdout)
        .lines()
        .any(|l| l == WEB_TARGET)
    {
        return Err(format!("run: rustup target add {WEB_TARGET}"));
    }
    let lock = fs::read_to_string(root.join("Cargo.lock")).unwrap_or_default();
    let version = lock
        .split("name = \"wasm-bindgen\"\nversion = \"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .map(String::from);
    if let Some(v) = &version {
        let out = Command::new("wasm-bindgen").arg("--version").output();
        let found = out
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
        if found.as_deref() != Some(&format!("wasm-bindgen {v}")) {
            return Err(format!(
                "wasm-bindgen {v} is needed: cargo install wasm-bindgen-cli --version {v} --locked"
            ));
        }
    }
    let mut cmd = Command::new("cargo");
    cmd.current_dir(root)
        .args(["build", "--target", WEB_TARGET])
        // gpui_web's wasm_thread asks for an unstable feature, as in Zed.
        .env("RUSTC_BOOTSTRAP", "1");
    if release {
        cmd.arg("--release");
    }
    run(cmd, "cargo build")?;
    let profile = if release { "release" } else { "debug" };
    let wasm = root
        .join("target")
        .join(WEB_TARGET)
        .join(profile)
        .join(format!("{}.wasm", name.replace('-', "_")));
    let out = root.join("dist").join("web");
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut bindgen = Command::new("wasm-bindgen");
    bindgen.arg(&wasm).arg("--out-dir").arg(&out).args([
        "--out-name",
        "app",
        "--target",
        "web",
        "--no-typescript",
    ]);
    run(bindgen, "wasm-bindgen")?;
    let bg = out.join("app_bg.wasm");
    if release && have("wasm-opt") {
        let mut opt = Command::new("wasm-opt");
        opt.args([
            "-Oz",
            "--enable-bulk-memory",
            "--enable-nontrapping-float-to-int",
            "--enable-sign-ext",
            "--enable-mutable-globals",
            "--enable-reference-types",
            "--enable-multivalue",
        ])
        .arg(&bg)
        .arg("-o")
        .arg(&bg);
        run(opt, "wasm-opt")?;
    } else if release {
        println!("[mok] wasm-opt (binaryen) not found: the wasm is larger than it needs to be");
    }
    fs::write(out.join("index.html"), INDEX.replace("{{name}}", &name))
        .map_err(|e| e.to_string())?;
    let size = fs::metadata(&bg).map(|m| m.len()).unwrap_or(0);
    println!("[ok] dist/web/app_bg.wasm: {size} bytes");
    Ok(out)
}

/// `build`, `run` and `package` for `--target native|web`.
pub fn target(root: &Path, verb: &str, args: &[String]) -> Result<(), String> {
    let mut web = false;
    let mut release = false;
    let mut rest = vec![];
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--target" => match it.next().map(String::as_str) {
                Some("web") => web = true,
                Some("native" | "desktop") | None => {}
                Some(other) => return Err(format!("target is native or web, not {other}")),
            },
            "--release" => release = true,
            other => rest.push(other.to_string()),
        }
    }
    match (verb, web) {
        ("build", true) => build_web(root, release).map(|_| ()),
        ("run", true) => {
            let out = build_web(root, release)?;
            println!("[mok] http://localhost:8080 (Ctrl-C stops)");
            let mut serve = Command::new("python3");
            serve.args(["-m", "http.server", "8080", "-d"]).arg(out);
            run(serve, "python3 -m http.server")
        }
        ("package", true) => {
            build_web(root, true)?;
            println!("[ok] dist/web is a static site: host the folder anywhere");
            Ok(())
        }
        ("package", false) => {
            if !have("cargo-packager") {
                return Err(
                    "installers come from cargo-packager: cargo install cargo-packager --locked, then add [package.metadata.packager] to Cargo.toml".into(),
                );
            }
            let mut cmd = Command::new("cargo");
            cmd.current_dir(root)
                .args(["packager", "--release"])
                .args(&rest);
            run(cmd, "cargo packager")
        }
        (verb, false) => {
            let mut cmd = Command::new("cargo");
            cmd.current_dir(root).arg(verb);
            if release {
                cmd.arg("--release");
            }
            cmd.args(&rest);
            run(cmd, &format!("cargo {verb}"))
        }
        (other, true) => Err(format!("{other} has no web form")),
    }
}

/// `cargo mok check`: the configuration, formatting and a type check, native and web when installed.
pub fn check(root: &Path) -> Result<(), String> {
    let mut fmt = Command::new("cargo");
    fmt.current_dir(root).args(["fmt", "--check"]);
    run(fmt, "cargo fmt --check")?;
    let mut check = Command::new("cargo");
    check.current_dir(root).arg("check");
    run(check, "cargo check")?;
    println!("[ok] formatted and compiling");
    Ok(())
}
