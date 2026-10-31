pub mod account;
pub mod agent;
pub mod assets;

pub mod buttons;
pub mod calendar;
pub mod canvas;
pub mod charts;
pub mod chat;
pub mod collab;
pub mod dashboard;
pub mod data_display;
pub mod debug;
pub mod devtools;
pub mod documents;
pub mod editor;
pub mod feedback;
pub mod files;
pub mod finance;
pub mod forms;
pub mod generative;
pub mod git;
pub mod i18n;
pub mod interaction;
pub mod layout;
pub mod lists;
pub mod mail;
pub mod maps;
pub mod media;
pub mod menus;
pub mod messaging;
pub mod misc;
pub mod motion;
pub mod navigation;
pub mod onboarding;
pub mod overlays;
pub mod primitives;
pub mod project;
pub mod rendering;
pub mod settings;
pub mod shell;
pub mod tables;
pub mod terminal;
pub mod theme;
pub mod tooling;
pub mod typography;

pub use assets::{Assets, Before, ImageAsset, VideoAsset};

use anyhow::Context as _;
use gpui::App;

/// Loads fonts and the theme, binds Tab and text keys. Call once, first.
pub fn init(cx: &mut App) -> anyhow::Result<()> {
    let missing = assets::missing(cx.asset_source().as_ref());
    if let Some(first) = missing.first() {
        anyhow::bail!(
            "mok: the asset source lacks {} of Mok's files, first {first}; pass `mok_gpui::Assets`, or `Assets::before` your own, to `Application::with_assets`",
            missing.len()
        );
    }
    log::info!("assets: the app's source loads all of Mok's files");
    assets::load_fonts(cx).context("mok: embedded fonts failed to register")?;
    setup(cx);
    Ok(())
}

/// `init` without assets or fonts, for app tests that render.
#[cfg(any(test, feature = "test-support"))]
pub fn init_for_tests(cx: &mut App) {
    setup(cx);
}

/// The theme and key bindings.
fn setup(cx: &mut App) {
    theme::Theme::init(cx);
    primitives::bind_focus_keys(cx);
    forms::bind_keys(cx);
    editor::bind_keys(cx);
    #[cfg(not(target_family = "wasm"))]
    terminal::bind_keys(cx);
    documents::bind_keys(cx);
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{AppContext, TestAppContext};

    #[gpui::test]
    #[should_panic(expected = "leaked handles")]
    fn a_leaked_entity_fails_its_test(cx: &mut TestAppContext) {
        std::mem::forget(cx.new(|_| ()));
    }
}
