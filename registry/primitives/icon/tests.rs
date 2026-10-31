use gpui::{Context, IntoElement, Render, TestAppContext, Window, black};

use super::{FoundIcons, Icon, LucideIcon, found};
use crate::theme::{ActiveTheme, Theme};

#[test]
fn every_embedded_icon_loads_and_the_ones_the_core_draws_are_among_them() {
    for (path, bytes) in super::EMBEDDED_ICONS {
        let loaded = gpui::AssetSource::load(&crate::Assets, path)
            .unwrap()
            .unwrap();
        assert_eq!(&*loaded, *bytes, "{path}");
    }
    for icon in [
        LucideIcon::Check,
        LucideIcon::CircleAlert,
        LucideIcon::ChevronRight,
    ] {
        assert!(
            super::embedded_icon(icon.path()).is_some(),
            "{icon:?} missing at {}",
            icon.path()
        );
    }
}

#[test]
fn an_app_path_draws_as_given_and_a_name_as_its_bundled_file() {
    let custom = Icon::from_path("app-icons/mark.svg");
    assert_eq!(custom.path.as_ref(), "app-icons/mark.svg");
    assert!(!custom.bundled);
    let named = Icon::new(LucideIcon::Check);
    assert_eq!(named.path.as_ref(), LucideIcon::Check.path());
    assert!(named.bundled);
}

#[test]
fn the_source_answers_whether_it_holds_a_path() {
    assert!(found(&crate::Assets, LucideIcon::Check.path()));
    assert!(!found(&crate::Assets, "app-icons/missing.svg"));
}

struct Missing;

impl Render for Missing {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Icon::from_path("app-icons/missing.svg")
    }
}

#[gpui::test]
fn a_missing_app_icon_draws_its_mark(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (_, cx) = cx.add_window_view(|_, _| Missing);
    cx.run_until_parked();
    let seen = cx.update(|_, cx| cx.global::<FoundIcons>().0.clone());
    assert_eq!(seen.get("app-icons/missing.svg"), Some(&false));
}

#[gpui::test]
fn a_broken_icon_shows_a_broken_picture_in_danger(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        let icon = Icon::from_path("app-icons/missing.svg")
            .color(black())
            .group_hover_color("row", black())
            .broken(cx);
        assert_eq!(icon.path.as_ref(), LucideIcon::ImageOff.path());
        assert_eq!(icon.color, Some(cx.theme().colors.danger));
        assert!(icon.hover.is_none(), "hover keeps the danger color");
    });
}

#[gpui::test]
fn init_without_mok_assets_returns_an_error(cx: &mut TestAppContext) {
    let error = cx
        .update(crate::init)
        .expect_err("the test source holds no Mok assets");
    assert!(format!("{error:#}").contains("pass `mok_gpui::Assets`"));
}
