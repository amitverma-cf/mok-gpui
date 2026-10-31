use std::collections::HashMap;

use gpui::{
    App, AssetSource, Global, Hsla, IntoElement, Radians, RenderOnce, SharedString, Transformation,
    Window, prelude::*, size, svg,
};

use crate::theme::{ActiveTheme, IconSize};

include!(concat!(env!("OUT_DIR"), "/mok_icons.rs"));

pub fn embedded_icon(path: &str) -> Option<&'static [u8]> {
    EMBEDDED_ICONS
        .iter()
        .find(|(p, _)| *p == path)
        .map(|(_, b)| *b)
}

#[derive(IntoElement)]
pub struct Icon {
    path: SharedString,
    bundled: bool,
    size: IconSize,
    color: Option<Hsla>,
    hover: Option<(SharedString, Hsla)>,
    rotation: Option<Radians>,
    scale: f32,
}

impl Icon {
    pub fn new(icon: impl IconAsset) -> Self {
        Self {
            bundled: true,
            ..Self::from_path(icon.path())
        }
    }

    /// Draws an SVG supplied by the application's asset source; a missing one logs and shows `broken`.
    pub fn from_path(path: impl Into<SharedString>) -> Self {
        Self {
            path: path.into(),
            bundled: false,
            size: IconSize::Md,
            color: None,
            hover: None,
            rotation: None,
            scale: 1.0,
        }
    }

    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    /// Defaults to `colors.fg`.
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Color while the pointer is over `group`.
    pub fn group_hover_color(
        mut self,
        group: impl Into<SharedString>,
        color: impl Into<Hsla>,
    ) -> Self {
        self.hover = Some((group.into(), color.into()));
        self
    }

    pub fn rotate(mut self, angle: impl Into<Radians>) -> Self {
        self.rotation = Some(angle.into());
        self
    }

    /// Draws the glyph larger or smaller inside the same box.
    pub fn scale(mut self, factor: f32) -> Self {
        self.scale = factor;
        self
    }
}

impl<T: IconAsset> From<T> for Icon {
    fn from(icon: T) -> Self {
        Icon::new(icon)
    }
}

/// App paths already looked up, and whether the source held them.
#[derive(Default)]
struct FoundIcons(HashMap<SharedString, bool>);

impl Global for FoundIcons {}

fn found(source: &dyn AssetSource, path: &str) -> bool {
    match source.load(path) {
        Ok(Some(_)) => true,
        Ok(None) => {
            log::error!("mok: icon {path} is not in the app's asset source");
            false
        }
        Err(error) => {
            log::error!("mok: icon {path} failed to load: {error:#}");
            false
        }
    }
}

impl Icon {
    /// What a missing app icon shows: a broken picture in danger.
    fn broken(self, cx: &App) -> Self {
        Self {
            path: LucideIcon::ImageOff.path().into(),
            bundled: true,
            color: Some(cx.theme().colors.danger),
            hover: None,
            ..self
        }
    }
}

impl RenderOnce for Icon {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let source = cx.asset_source().clone();
        let missing = !self.bundled
            && !*cx
                .default_global::<FoundIcons>()
                .0
                .entry(self.path.clone())
                .or_insert_with(|| found(source.as_ref(), &self.path));
        let icon = if missing { self.broken(cx) } else { self };
        let theme = cx.theme();
        let color = icon.color.unwrap_or(theme.colors.fg);
        svg()
            .path(icon.path)
            .size(theme.icon_size(icon.size))
            .flex_none()
            .text_color(color)
            .when_some(icon.hover, |svg, (group, hover)| {
                svg.group_hover(group, |style| style.text_color(hover))
            })
            .when(icon.rotation.is_some() || icon.scale != 1.0, |svg| {
                svg.with_transformation(
                    Transformation::rotate(icon.rotation.unwrap_or(Radians(0.0)))
                        .with_scaling(size(icon.scale, icon.scale)),
                )
            })
    }
}

#[cfg(test)]
mod tests;
