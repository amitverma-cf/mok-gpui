use std::time::Duration;

use gpui::{TestAppContext, hsla};

use super::{ActiveTheme, Mode, Palette, Syntax, Theme};

/// An owner's palette serves its own mode; the other keeps Mok's, and none brings Mok's back.
#[gpui::test]
fn an_owners_palette_serves_its_mode_alone(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let mut own = Palette::light(false);
    own.accent = hsla(0.6, 0.5, 0.5, 1.0);
    cx.update(|cx| Theme::set_palette(Mode::Light, Some(own.clone()), cx));
    assert_eq!(cx.read(|cx| cx.theme().palette()), own);
    cx.update(|cx| Theme::set_mode(Mode::Dark, cx));
    assert_eq!(cx.read(|cx| cx.theme().palette()), Palette::dark(false));
    cx.update(|cx| {
        Theme::set_mode(Mode::Light, cx);
        Theme::set_palette(Mode::Light, None, cx);
    });
    assert_eq!(cx.read(|cx| cx.theme().palette()), Palette::light(false));
}

/// A mode set at once keeps its palette through fades.
#[gpui::test]
fn a_mode_set_at_once_skips_the_fade(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        Theme::update(cx, |theme| theme.reduced_motion = true);
        Theme::set_mode(Mode::Dark, cx);
        Theme::set_mode_now(Mode::Light, cx);
    });
    assert_eq!(
        cx.read(|cx| cx.theme().colors.clone()),
        Palette::light(false)
    );
    std::thread::sleep(Duration::from_millis(2));
    cx.executor().advance_clock(Duration::from_millis(100));
    cx.run_until_parked();
    assert!(!cx.read(|cx| cx.theme().is_dark()));
    assert_eq!(
        cx.read(|cx| cx.theme().colors.clone()),
        Palette::light(false)
    );
}

/// Every color of the palette and of code answers to its field's name, the quiet fills too.
#[test]
fn every_color_answers_to_its_name() {
    let mut palette = Palette::light(false);
    for (ix, name) in Palette::NAMES.iter().enumerate() {
        *palette.token_mut(name) = hsla(ix as f32 / 64.0, 0.5, 0.5, 1.0);
    }
    assert_eq!(Palette::NAMES.len(), 36, "every color of the palette");
    let at = |name: &str| {
        Palette::NAMES
            .iter()
            .position(|each| *each == name)
            .expect("a name")
    };
    assert_eq!(
        palette.success_subtle,
        hsla(at("success_subtle") as f32 / 64.0, 0.5, 0.5, 1.0)
    );
    assert_eq!(palette.ink, hsla(at("ink") as f32 / 64.0, 0.5, 0.5, 1.0));
    for name in Syntax::NAMES {
        *palette.syntax.token_mut(name) = hsla(0.4, 0.2, 0.3, 1.0);
    }
    assert_eq!(Syntax::NAMES.len(), 13, "every color of code");
    assert_eq!(palette.syntax.comment, hsla(0.4, 0.2, 0.3, 1.0));
}

#[test]
#[should_panic(expected = "no palette color glow")]
fn an_unknown_color_fails() {
    Palette::light(false).token_mut("glow");
}

#[test]
fn glass_lets_the_blur_through_until_high_contrast() {
    for (plain, strong) in [
        (Palette::light(false), Palette::light(true)),
        (Palette::dark(false), Palette::dark(true)),
    ] {
        assert!(
            plain.glass.a > 0.0 && plain.glass.a < 1.0,
            "{:?}",
            plain.glass
        );
        assert_eq!(strong.glass, strong.bg);
    }
}

#[gpui::test]
fn baked_tokens_match_config(cx: &mut TestAppContext) {
    use crate::theme::{ControlSize, Radius, TextSize, Theme};
    cx.update(Theme::init);
    cx.read(|cx| {
        let theme = cx.theme();
        assert_eq!(theme.control_height(ControlSize::Sm).0, 24.0 / 16.0);
        assert_eq!(theme.control_height(ControlSize::Md).0, 28.0 / 16.0);
        assert_eq!(theme.control_height(ControlSize::Lg).0, 32.0 / 16.0);

        assert_eq!(theme.radius(Radius::Sm).0, 4.0 / 16.0);
        assert_eq!(theme.radius(Radius::Md).0, 6.0 / 16.0);
        assert_eq!(theme.radius(Radius::Lg).0, 8.0 / 16.0);
        assert_eq!(theme.radius(Radius::Xl).0, 12.0 / 16.0);

        assert_eq!(theme.text_size(TextSize::Xs).0, 11.0 / 16.0);
        assert_eq!(theme.text_size(TextSize::Sm).0, 12.0 / 16.0);
        assert_eq!(theme.text_size(TextSize::Base).0, 13.0 / 16.0);
        assert_eq!(theme.text_size(TextSize::Display).0, 32.0 / 16.0);

        assert_eq!(theme.sidebar_width(true).0, 56.0 / 16.0);
        assert_eq!(theme.sidebar_width(false).0, 240.0 / 16.0);
        assert_eq!(theme.sheet_size().0, 380.0 / 16.0);
        assert_eq!(theme.dialog_width().0, 440.0 / 16.0);
        assert_eq!(theme.toast_width().0, 360.0 / 16.0);
        assert_eq!(theme.titlebar_height().0, 38.0 / 16.0);
        assert_eq!(theme.pane_min().0, 160.0 / 16.0);
        assert_eq!(theme.hairline(), gpui::px(1.0));
    });
}

#[gpui::test]
fn baked_fonts_register_in_theme(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (font, mono) = cx.read(|cx| {
        (
            cx.theme().font_family.clone(),
            cx.theme().mono_family.clone(),
        )
    });
    assert_eq!(font, "Inter");
    assert_eq!(mono, "JetBrains Mono");
    assert!(crate::assets::EMBEDDED_FONTS.len() >= 6);
}

#[test]
fn theme_presets_load_and_meet_contrast() {
    for &name in Theme::PRESET_NAMES {
        let light = Palette::preset(name, false, false)
            .unwrap_or_else(|| panic!("failed preset {name} light"));
        let dark = Palette::preset(name, true, false)
            .unwrap_or_else(|| panic!("failed preset {name} dark"));
        assert!(light.bg.l > 0.6, "{name} light bg should be bright");
        assert!(dark.bg.l < 0.4, "{name} dark bg should be dark");
    }
}
