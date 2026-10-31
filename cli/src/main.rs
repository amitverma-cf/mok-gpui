use std::env;
use std::fs;
use std::path::{Path, PathBuf};

mod bundle;
mod component;
mod deps;
mod fetch;
mod graph;
mod icons;
mod install;
mod items;
mod lock;
mod notes;
mod registry;
mod rewrite;
mod source;
mod sync;
mod update;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "mok" {
        args.remove(1);
    }

    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    match command {
        "init" => handle_init(&args[2..]),
        "list" => handle_list(&args[2..]),
        "add" => handle_add(&args[2..]),
        "update" => handle_update(&args[2..]),
        "doctor" | "check" => handle_doctor(),
        "registry" if args.get(2).map(String::as_str) == Some("build") => {
            let out = args
                .get(3)
                .map_or_else(|| PathBuf::from("dist/registry"), PathBuf::from);
            if let Err(e) = bundle::build(Path::new("."), &out) {
                eprintln!("[error] {e}");
                std::process::exit(1);
            }
        }
        "registry" => {
            let check = args.iter().any(|a| a == "--check");
            let root = args
                .iter()
                .skip(2)
                .find(|a| !a.starts_with("--"))
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            if let Err(e) = registry::generate(&root, check) {
                eprintln!("[error] {e}");
                std::process::exit(1);
            }
        }
        "help" | "--help" | "-h" => print_help(),
        other => {
            eprintln!(
                "Unknown command: '{}'. Run 'cargo mok help' for usage.",
                other
            );
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("Mok GPUI Component CLI\n");
    println!("Usage:");
    println!("  cargo mok <command> [options]\n");
    println!("Commands:");
    println!("  init                     Initialize mok-gpui.toml and project asset structure");
    println!("  list [--json]            List all available components in the registry");
    println!("  add icons <pack>         Add an icon pack (lucide, heroicons, hugeicons)");
    println!("  add <component>          Copy a component and what it needs into src/ui");
    println!("  update [--dry-run]       Bring copied components up to date, keeping your edits");
    println!(
        "  registry build [dir]     Write index.json and source.tar.gz for hosting (maintainers)"
    );
    println!("  doctor, check            Verify mok-gpui.toml contrast, fonts, and assets");
    println!("  help                     Show this help message\n");
    println!("Examples:");
    println!("  cargo mok init");
    println!("  cargo mok list");
    println!("  cargo mok add button");
    println!("  cargo mok add command_palette");
    println!("  cargo mok add icons heroicons");
    println!("  cargo mok doctor");
}

fn detect_folder_structure(root: &Path) -> (Vec<String>, String) {
    let cargo_toml_path = root.join("Cargo.toml");
    let is_workspace = if let Ok(content) = fs::read_to_string(&cargo_toml_path) {
        content.contains("[workspace]") && content.contains("members")
    } else {
        false
    };

    if is_workspace {
        if root.join("crates").is_dir() {
            (
                vec!["crates/**/*.rs".into()],
                "workspace with 'crates/' folder".into(),
            )
        } else if root.join("packages").is_dir() {
            (
                vec!["packages/**/*.rs".into()],
                "workspace with 'packages/' folder".into(),
            )
        } else {
            (
                vec!["*/**/*.rs".into()],
                "workspace with root members".into(),
            )
        }
    } else if root.join("src").is_dir() {
        (
            vec!["src/**/*.rs".into(), "examples/**/*.rs".into()],
            "standard single-crate (src/)".into(),
        )
    } else {
        (vec!["**/*.rs".into()], "custom project structure".into())
    }
}

fn handle_init(args: &[String]) {
    let force = args.iter().any(|a| a == "--force" || a == "-f");
    let assets = !args.iter().any(|a| a == "--no-assets");
    if let Err(e) = init(force, assets) {
        eprintln!("[error] {e}");
        std::process::exit(1);
    }
}

fn init(force: bool, assets: bool) -> Result<(), String> {
    let cwd = env::current_dir().map_err(|e| e.to_string())?;
    if !cwd.join("Cargo.toml").is_file() {
        return Err("no Cargo.toml here; run this in your Rust project".into());
    }
    let config_path = cwd.join("mok-gpui.toml");
    if config_path.exists() && !force {
        println!("[info] mok-gpui.toml already exists (--force rewrites it)");
    } else {
        let (scan_sources, layout) = detect_folder_structure(&cwd);
        println!("[mok] Project layout: {layout}");
        let sources = scan_sources
            .iter()
            .map(|s| format!("    {s:?}"))
            .collect::<Vec<_>>()
            .join(
                ",
",
            );
        fs::write(&config_path, config_template(&sources))
            .map_err(|e| format!("mok-gpui.toml: {e}"))?;
        println!("[ok] Wrote mok-gpui.toml");
    }
    let cargo = cwd.join(".cargo").join("config.toml");
    let existing = fs::read_to_string(&cargo).unwrap_or_default();
    if existing.contains("MOK_CONFIG_PATH") {
        println!("[info] .cargo/config.toml already names mok-gpui.toml");
    } else {
        let line = "MOK_CONFIG_PATH = { value = \"mok-gpui.toml\", relative = true }";
        let updated = if existing.contains("[env]") {
            existing.replacen(
                "[env]",
                &format!(
                    "[env]
{line}"
                ),
                1,
            )
        } else {
            format!(
                "{existing}
# Mok bakes fonts, icons and tokens from this file at build time.
[env]
{line}
"
            )
        };
        fs::create_dir_all(cargo.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(&cargo, updated.trim_start()).map_err(|e| format!(".cargo/config.toml: {e}"))?;
        println!("[ok] .cargo/config.toml points the build at mok-gpui.toml");
    }
    if assets {
        let fonts = fetch::fonts(&cwd.join("assets").join("fonts"))?;
        println!("[ok] Fonts: {fonts} new files in assets/fonts");
        let icons = fetch::lucide(&cwd.join("assets").join("icons"))?;
        println!(
            "[ok] Lucide: {icons} files in assets/icons (the build embeds only the ones your code uses)"
        );
    }
    println!(
        "
Next: cargo mok add button   (then `mod ui;` in your crate)"
    );
    Ok(())
}

fn config_template(sources: &str) -> String {
    format!(
        r#"[paths]
assets_dir = "assets"
icons_dir = "assets/icons"
scan_sources = [
{sources}
]

[theme]
preset = "mok"
# Customize theme colors or tokens by adding tables here:
# [theme.palette.light]
# bg = 0xfcfaf7
# [theme.tokens]
# base_rem = 16.0

[fonts]
sans = {{ family = "Inter", files = [
    "fonts/Inter-Regular.ttf",
    "fonts/Inter-Medium.ttf",
    "fonts/Inter-SemiBold.ttf",
    "fonts/Inter-Italic.ttf"
] }}
mono = {{ family = "JetBrains Mono", files = [
    "fonts/JetBrainsMono-Regular.ttf",
    "fonts/JetBrainsMono-Medium.ttf"
] }}
fallback = {{ family = "IBM Plex Sans", files = [
    "fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf"
] }}

[icons]
mode = "auto"
minify = true

[media.images]

[media.videos]

[media.files]
"#
    )
}

fn handle_list(args: &[String]) {
    let as_json = args.iter().any(|a| a == "--json");
    let non_flags: Vec<&str> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .map(|s| s.as_str())
        .collect();

    let filter = if non_flags.is_empty() {
        if args.iter().any(|a| a == "--all") {
            Some("--all".to_string())
        } else {
            None
        }
    } else {
        Some(non_flags.join(" "))
    };

    if let Err(e) = component::list_components(filter.as_deref(), as_json) {
        eprintln!("[error] {e}");
        std::process::exit(1);
    }
}

fn handle_add(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: cargo mok add <component> OR cargo mok add icons <pack>");
        eprintln!("Run 'cargo mok list' to see all available components.");
        return;
    }

    if args[0] == "icons" {
        if args.len() < 2 {
            eprintln!("Usage: cargo mok add icons <pack> (e.g., heroicons, hugeicons, lucide)");
            return;
        }
        handle_add_icons(&args[1]);
        return;
    }

    handle_add_component(args);
}

fn handle_add_icons(pack: &str) {
    if let Err(e) = add_icons(pack) {
        eprintln!("[error] {e}");
        std::process::exit(1);
    }
}

/// The pack's SVGs into `assets/icons`; the build embeds only the icons the code names.
fn add_icons(pack: &str) -> Result<(), String> {
    let cwd = env::current_dir().map_err(|e| e.to_string())?;
    if !cwd.join("mok-gpui.toml").is_file() {
        return Err("mok-gpui.toml not found. Run 'cargo mok init' first.".into());
    }
    let pack = pack.to_ascii_lowercase();
    let icons = cwd.join("assets").join("icons");
    let (dir, enum_name) = match pack.as_str() {
        "lucide" => (icons, "LucideIcon"),
        "heroicons" => (icons.join("heroicons"), "HeroIcon"),
        "hugeicons" => (icons.join("hugeicons"), "HugeIcon"),
        other => {
            return Err(format!(
                "unknown icon pack '{other}': lucide, heroicons or hugeicons"
            ));
        }
    };
    let count = icons::download_icon_pack(&pack, &dir)?;
    println!(
        "[ok] {count} icons in {}",
        dir.strip_prefix(&cwd).unwrap_or(&dir).display()
    );
    println!("Use them as Icon::new({enum_name}::<Variant>)");
    Ok(())
}

fn handle_update(args: &[String]) {
    let mut opts = source::Options {
        from: None,
        dir: PathBuf::from("src/ui"),
        force: false,
    };
    let mut dry = false;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--from" => opts.from = it.next().map(PathBuf::from),
            "--force" => opts.force = true,
            "--dry-run" => dry = true,
            other => {
                eprintln!(
                    "[error] unknown option '{other}'; update takes --from, --dry-run and --force"
                );
                std::process::exit(1);
            }
        }
    }
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Err(e) = update::update(&cwd, &opts, dry) {
        eprintln!("[error] {e}");
        std::process::exit(1);
    }
}

fn handle_add_component(args: &[String]) {
    let mut opts = source::Options {
        from: None,
        dir: PathBuf::from("src/ui"),
        force: false,
    };
    let mut names = vec![];
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--from" => opts.from = it.next().map(PathBuf::from),
            "--dir" => opts.dir = it.next().map(PathBuf::from).unwrap_or(opts.dir),
            "--force" => opts.force = true,
            other => names.push(other.to_string()),
        }
    }
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Err(e) = component::scaffold_components(&names, &cwd, &opts) {
        eprintln!("[error] {e}");
        std::process::exit(1);
    }
}

fn srgb_luminance(rgb: u32) -> f64 {
    let r = ((rgb >> 16) & 0xff) as f64 / 255.0;
    let g = ((rgb >> 8) & 0xff) as f64 / 255.0;
    let b = (rgb & 0xff) as f64 / 255.0;
    let to_linear = |v: f64| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * to_linear(r) + 0.7152 * to_linear(g) + 0.0722 * to_linear(b)
}

fn contrast_ratio(c1: u32, c2: u32) -> f64 {
    let l1 = srgb_luminance(c1);
    let l2 = srgb_luminance(c2);
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

fn handle_doctor() {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let config_path = cwd.join("mok-gpui.toml");

    println!("Checking Mok configuration...\n");
    if !config_path.exists() {
        eprintln!("[FAIL] mok-gpui.toml was not found. Run 'cargo mok init' to create one.");
        return;
    }
    println!("[PASS] Configuration file found: {}", config_path.display());

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[FAIL] Could not read mok-gpui.toml: {}", e);
            return;
        }
    };

    let parsed: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[FAIL] Syntax error in mok-gpui.toml: {}", e);
            return;
        }
    };
    println!("[PASS] Syntax of mok-gpui.toml is valid TOML");

    // Check WCAG Contrast
    if let Some(theme) = parsed.get("theme").and_then(|t| t.get("palette")) {
        for mode in ["light", "dark"] {
            if let Some(pal) = theme.get(mode) {
                let paper = pal
                    .get("paper")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(0xffffff) as u32;
                let ink = pal
                    .get("ink")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(0x181613) as u32;
                let ratio = contrast_ratio(paper, ink);
                if ratio >= 4.5 {
                    println!(
                        "[PASS] {} palette ink on paper contrast: {:.2}:1 (WCAG AA compliant)",
                        mode, ratio
                    );
                } else {
                    println!(
                        "[WARN] {} palette ink on paper contrast: {:.2}:1 (below WCAG 4.5:1)",
                        mode, ratio
                    );
                }
            }
        }
    }

    // Check Assets
    let mut missing_fonts = 0;
    if let Some(fonts) = parsed.get("fonts").and_then(|f| f.as_table()) {
        for (name, entry) in fonts {
            if let Some(files) = entry.get("files").and_then(|f| f.as_array()) {
                for file_val in files {
                    if let Some(rel) = file_val.as_str() {
                        let path = cwd.join("assets").join(rel);
                        let root_path = cwd.join(rel);
                        if !path.exists() && !root_path.exists() {
                            println!(
                                "[WARN] Font file declared in [fonts.{}] not found: {}",
                                name, rel
                            );
                            missing_fonts += 1;
                        }
                    }
                }
            }
        }
    }
    if missing_fonts == 0 {
        println!("[PASS] All declared fonts resolved successfully on disk");
    }

    // Check Icons
    let icons = cwd.join("assets").join("icons");
    let count = ["", "heroicons", "hugeicons"]
        .iter()
        .filter(|pack| {
            fs::read_dir(icons.join(pack)).is_ok_and(|d| {
                d.flatten()
                    .any(|e| e.path().extension().is_some_and(|x| x == "svg"))
            })
        })
        .count();
    println!("[PASS] Icon packs on disk: {count}");

    println!(
        "
[ok] Mok Doctor completed."
    );
}
