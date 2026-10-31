//! `cargo mok gen`: bakes `mok-gpui.toml` into plain Rust files in the project, the job `build.rs` does inside the library.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::build_support::fonts::FontsConfig;
use crate::build_support::media::MediaConfig;
use crate::build_support::tokens::ThemeConfig;

#[derive(Deserialize, Default)]
struct Config {
    paths: Option<Paths>,
    theme: Option<ThemeConfig>,
    fonts: Option<FontsConfig>,
    icons: Option<Icons>,
    media: Option<MediaConfig>,
}

#[derive(Deserialize, Default)]
struct Paths {
    assets_dir: Option<String>,
    icons_dir: Option<String>,
    scan_sources: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
struct Icons {
    mode: Option<String>,
    minify: Option<bool>,
}

/// Writes `mok_baked.rs`, `mok_fonts.rs`, `mok_media.rs` and `mok_icons.rs` into `<root>/<dir>/gen`.
pub fn run(root: &Path, dir: &str) -> Result<(), String> {
    let config_path = root.join("mok-gpui.toml");
    let text = fs::read_to_string(&config_path)
        .map_err(|_| "mok-gpui.toml not found; run `cargo mok init` first".to_string())?;
    let config: Config = toml::from_str(&text).map_err(|e| format!("mok-gpui.toml: {e}"))?;
    let out = root.join(dir).join("gen");
    fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let paths = config.paths.as_ref();
    let assets = root.join(
        paths
            .and_then(|p| p.assets_dir.as_deref())
            .unwrap_or("assets"),
    );
    let icons = root.join(
        paths
            .and_then(|p| p.icons_dir.as_deref())
            .unwrap_or("assets/icons"),
    );
    let sources = paths
        .and_then(|p| p.scan_sources.clone())
        .unwrap_or_else(|| vec!["src".into()]);
    let guard = |f: &dyn Fn()| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
            .map_err(|_| "mok-gpui.toml could not be baked (see the message above)".to_string())
    };
    guard(&|| {
        crate::build_support::tokens::generate_tokens(
            config.theme.as_ref(),
            config.fonts.as_ref(),
            &out,
        )
    })?;
    guard(&|| {
        crate::build_support::fonts::generate_fonts(config.fonts.as_ref(), root, &assets, &out)
    })?;
    guard(&|| {
        crate::build_support::media::generate_media(config.media.as_ref(), root, &assets, &out)
    })?;
    let mode = config
        .icons
        .as_ref()
        .and_then(|i| i.mode.as_deref())
        .unwrap_or("auto");
    let minify = config.icons.as_ref().and_then(|i| i.minify).unwrap_or(true);
    guard(&|| {
        crate::build_support::icons::generate_icons(&icons, &sources, root, &out, mode, minify)
    })?;
    println!("[ok] Baked fonts, icons, media and tokens into {dir}/gen");
    Ok(())
}
