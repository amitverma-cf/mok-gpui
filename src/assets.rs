use std::borrow::Cow;

use gpui::{App, AssetSource, Result, SharedString};

include!(concat!(env!("OUT_DIR"), "/mok_fonts.rs"));
include!(concat!(env!("OUT_DIR"), "/mok_media.rs"));

/// Every path the build baked in: the icons the code uses, the configured fonts, images and files.
fn paths() -> impl Iterator<Item = &'static str> {
    crate::primitives::EMBEDDED_ICONS
        .iter()
        .map(|(path, _)| *path)
        .chain(FONT_NAMES.iter().copied())
        .chain(MEDIA_NAMES.iter().copied())
}

/// Icons, fonts and files baked in from `mok-gpui.toml`. Pass to `Application::with_assets`, alone or through `Assets::before`.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = crate::primitives::embedded_icon(path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        if let Some(bytes) = embedded_font(path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        Ok(embedded_media(path).map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(paths()
            .filter(|name| name.starts_with(path))
            .map(SharedString::from)
            .collect())
    }
}

pub(crate) fn load_fonts(cx: &App) -> Result<()> {
    let fonts: Vec<Cow<'static, [u8]>> = EMBEDDED_FONTS
        .iter()
        .map(|bytes| Cow::Borrowed(*bytes))
        .collect();
    log::info!("assets: registering {} fonts", fonts.len());
    cx.text_system().add_fonts(fonts)
}

/// Mok's files first, then another source's: one source for an app that runs both.
pub struct Before<T>(T);

impl Assets {
    /// Serves Mok's files, and every other path from `other`, such as the app's own `AssetSource`.
    pub fn before<T: AssetSource>(other: T) -> Before<T> {
        Before(other)
    }
}

impl<T: AssetSource> AssetSource for Before<T> {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match Assets.load(path)? {
            Some(data) => Ok(Some(data)),
            None => self.0.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut names = Assets.list(path)?;
        names.extend(self.0.list(path)?);
        names.sort();
        names.dedup();
        Ok(names)
    }
}

/// Mok's files the source cannot load, each with the reason.
pub(crate) fn missing(source: &dyn AssetSource) -> Vec<String> {
    paths()
        .filter_map(|path| match source.load(path) {
            Ok(Some(_)) => None,
            Ok(None) => Some(format!("{path}: not found")),
            Err(error) => Some(format!("{path}: {error}")),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use gpui::{AssetSource, Result, SharedString};

    use super::{Assets, missing};

    const PLEX: &str = "fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf";

    /// Icons alone, and an error for any other path, as a library's own source might answer.
    struct IconsOnly;

    impl AssetSource for IconsOnly {
        fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
            if path == "icons/other-only.svg" {
                return Ok(Some(Cow::Borrowed(b"<svg/>")));
            }
            match Assets.load(path)? {
                Some(data) if path.starts_with("icons/") => Ok(Some(data)),
                _ => Err(anyhow::anyhow!("could not find asset at path {path}")),
            }
        }

        fn list(&self, _: &str) -> Result<Vec<SharedString>> {
            Ok(vec![
                "icons/check.svg".into(),
                "icons/other-only.svg".into(),
            ])
        }
    }

    #[test]
    fn mok_s_own_source_holds_every_file() {
        assert_eq!(missing(&Assets), Vec::<String>::new());
    }

    #[test]
    fn an_icons_only_source_lacks_mok_s_fonts() {
        let missing = missing(&IconsOnly);
        assert!(
            missing.iter().any(|line| line.starts_with(PLEX)),
            "{missing:?}"
        );
    }

    #[test]
    fn mok_s_files_come_first_then_the_other_s() {
        let both = Assets::before(IconsOnly);
        assert_eq!(missing(&both), Vec::<String>::new());
        let plex = both.load(PLEX).unwrap().unwrap();
        assert_eq!(plex, Assets.load(PLEX).unwrap().unwrap());
        assert_eq!(
            &*both.load("icons/other-only.svg").unwrap().unwrap(),
            b"<svg/>"
        );
        assert!(both.load("icons/neither.svg").is_err());
        let icons = both.list("icons/").unwrap();
        let checks = icons
            .iter()
            .filter(|name| *name == "icons/check.svg")
            .count();
        assert_eq!(checks, 1);
        assert!(icons.iter().any(|name| name == "icons/other-only.svg"));
    }

    #[test]
    fn embedded_fonts_are_available() {
        assert!(!super::EMBEDDED_FONTS.is_empty());
        for font in super::EMBEDDED_FONTS {
            assert!(!font.is_empty());
        }
        assert!(super::embedded_font(PLEX).is_some());
    }

    #[test]
    fn type_safe_image_media_asset() {
        let logo_bytes = super::Image::Logo.bytes();
        assert!(!logo_bytes.is_empty());
        assert_eq!(super::Image::Logo.mime_type(), "image/png");
        assert_eq!(super::Image::Logo.path(), "logo.png");
        assert_eq!(super::embedded_media("logo.png"), Some(logo_bytes));
    }
}
