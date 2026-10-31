use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

mod build_support;

use build_support::fonts::FontsConfig;
use build_support::media::MediaConfig;
use build_support::tokens::ThemeConfig;

#[derive(Debug, Deserialize, Default)]
struct Config {
    paths: Option<PathsConfig>,
    theme: Option<ThemeConfig>,
    fonts: Option<FontsConfig>,
    icons: Option<IconsConfig>,
    media: Option<MediaConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct PathsConfig {
    assets_dir: Option<String>,
    icons_dir: Option<String>,
    scan_sources: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
struct IconsConfig {
    mode: Option<String>,
    minify: Option<bool>,
}

/// The modules `cargo mok add` never copies; keep in step with the CLI's `CORE`.
const CORE_MODULES: &[&str] = &[
    "i18n",
    "layout",
    "motion",
    "primitives",
    "theme",
    "typography",
];

/// The project's `mok-gpui.toml`, named by `MOK_CONFIG_PATH`, which `cargo mok init` sets.
fn find_config_file() -> PathBuf {
    let Some(path) = env::var_os("MOK_CONFIG_PATH") else {
        panic!(
            "mok: MOK_CONFIG_PATH is not set, so there is no mok-gpui.toml to bake fonts, icons and tokens from.
             Run `cargo mok init` in your project, or add to .cargo/config.toml:
             
    [env]
    MOK_CONFIG_PATH = {{ value = \"mok-gpui.toml\", relative = true }}"
        );
    };
    let path = PathBuf::from(path);
    if !path.is_file() {
        panic!(
            "mok: MOK_CONFIG_PATH names {}, which is not a file",
            path.display()
        );
    }
    path
}

fn main() {
    println!("cargo:rerun-if-env-changed=MOK_CONFIG_PATH");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let config_path = find_config_file();
    println!("cargo:rerun-if-changed={}", config_path.display());
    let content = fs::read_to_string(&config_path).expect("failed to read mok-gpui.toml");
    let config: Config = toml::from_str(&content).expect("failed to parse mok-gpui.toml");

    let root_dir = config_path.parent().expect("mok-gpui.toml has a folder");

    let assets_dir = config
        .paths
        .as_ref()
        .and_then(|p| p.assets_dir.as_deref())
        .map(|d| root_dir.join(d))
        .unwrap_or_else(|| root_dir.join("assets"));

    // 1. Bake theme tokens and palettes
    build_support::tokens::generate_tokens(config.theme.as_ref(), config.fonts.as_ref(), &out_dir);

    // 2. Bake and verify fonts
    build_support::fonts::generate_fonts(config.fonts.as_ref(), root_dir, &assets_dir, &out_dir);

    // 3. Bake and verify media
    build_support::media::generate_media(config.media.as_ref(), root_dir, &assets_dir, &out_dir);

    // 4. Discover and prune icons
    let icons_dir = config
        .paths
        .as_ref()
        .and_then(|p| p.icons_dir.as_deref())
        .map(|d| root_dir.join(d))
        .unwrap_or_else(|| root_dir.join("assets/icons"));

    let mut scan_sources = config
        .paths
        .as_ref()
        .and_then(|p| p.scan_sources.clone())
        .unwrap_or_else(|| vec!["registry".into(), "examples".into()]);
    // The modules that stay in this crate draw icons of their own, whatever the project uses.
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo sets it"));
    for core in CORE_MODULES {
        scan_sources.push(
            manifest_dir
                .join("registry")
                .join(core)
                .to_string_lossy()
                .into_owned(),
        );
    }

    let icon_mode = config
        .icons
        .as_ref()
        .and_then(|i| i.mode.as_deref())
        .unwrap_or("auto");

    let minify_icons = config.icons.as_ref().and_then(|i| i.minify).unwrap_or(true);

    build_support::icons::generate_icons(
        &icons_dir,
        &scan_sources,
        root_dir,
        &out_dir,
        icon_mode,
        minify_icons,
    );
}
