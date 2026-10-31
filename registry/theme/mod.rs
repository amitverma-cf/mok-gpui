mod calendar;
mod canvas;
mod chart;
mod dashboard;
mod elevation;
mod files;
mod generative;
mod interaction;
mod maps;
mod media;
mod messaging;
mod misc;
mod onboarding;
mod palette;
mod platform;
mod project;
mod syntax;
mod tokens;
mod tooling;

pub mod baked {
    include!(concat!(env!("OUT_DIR"), "/mok_baked.rs"));
}

#[cfg(all(test, feature = "test-support"))]
mod tests;

use std::time::Duration;

use gpui::{App, Global, SharedString, WindowAppearance};
use web_time::Instant;

use crate::motion;
pub use calendar::CalendarSizes;
pub use canvas::CanvasSizes;
pub use chart::ChartSizes;
pub use dashboard::DashboardSizes;
pub use elevation::Elevation;
pub use files::FileSizes;
pub use generative::GenerativeSizes;
pub use interaction::InteractionSizes;
pub use maps::MapSizes;
pub use media::MediaSizes;
pub use messaging::MessagingSizes;
pub use misc::MiscSizes;
pub use onboarding::OnboardingSizes;
pub use palette::{HUE_NAMES, Mix, Palette, Syntax};
pub use platform::Platform;
pub use project::ProjectSizes;
pub use syntax::{SyntaxTheme, syntax_themes};
pub use tokens::{AvatarSize, ContainerSize, ControlSize, Density, IconSize, Radius, TextSize};
pub use tooling::ToolingSizes;

const FRAME: Duration = Duration::from_millis(8);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Light,
    Dark,
}

impl From<WindowAppearance> for Mode {
    fn from(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Mode::Dark,
            WindowAppearance::Light | WindowAppearance::VibrantLight => Mode::Light,
        }
    }
}

/// Design tokens. One global per app.
pub struct Theme {
    mode: Mode,
    high_contrast: bool,
    color_blind_safe: bool,
    transition: u64,
    custom: [Option<Palette>; 2],
    pub colors: Palette,
    pub density: Density,
    pub radius_scale: f32,
    pub font_scale: f32,
    pub font_family: SharedString,
    pub mono_family: SharedString,
    pub reduced_motion: bool,
    pub platform: Platform,
}

impl Global for Theme {}

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}

impl Theme {
    pub fn init(cx: &mut App) {
        let mode = match baked::BAKED_DEFAULT_MODE {
            "dark" => Mode::Dark,
            "light" => Mode::Light,
            _ => Mode::default(),
        };
        let density = match baked::BAKED_DENSITY_STR {
            "compact" => Density::Compact,
            "comfortable" => Density::Comfortable,
            _ => Density::Standard,
        };
        cx.set_global(Theme {
            mode,
            high_contrast: baked::BAKED_HIGH_CONTRAST,
            color_blind_safe: baked::BAKED_COLOR_BLIND_SAFE,
            transition: 0,
            custom: [None, None],
            colors: if mode == Mode::Dark {
                Palette::dark(baked::BAKED_HIGH_CONTRAST)
            } else {
                Palette::light(baked::BAKED_HIGH_CONTRAST)
            },
            density,
            radius_scale: baked::BAKED_RADIUS_SCALE,
            font_scale: baked::BAKED_FONT_SCALE,
            font_family: baked::BAKED_FONT_SANS.into(),
            mono_family: baked::BAKED_FONT_MONO.into(),
            reduced_motion: false,
            platform: Platform::current(),
        });
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn high_contrast(&self) -> bool {
        self.high_contrast
    }

    pub fn color_blind_safe(&self) -> bool {
        self.color_blind_safe
    }

    pub fn is_dark(&self) -> bool {
        self.mode == Mode::Dark
    }

    /// Cross-fades the palette to `mode`.
    pub fn set_mode(mode: Mode, cx: &mut App) {
        log::info!("theme: mode -> {mode:?}");
        cx.global_mut::<Theme>().mode = mode;
        Self::fade_to_target(cx);
    }

    /// Sets `mode` at once, stopping any fade under way.
    pub fn set_mode_now(mode: Mode, cx: &mut App) {
        log::info!("theme: mode -> {mode:?}, at once");
        let theme = cx.global_mut::<Theme>();
        theme.mode = mode;
        theme.transition += 1;
        theme.colors = theme.target();
        cx.refresh_windows();
    }

    pub fn set_high_contrast(on: bool, cx: &mut App) {
        log::info!("theme: high contrast -> {on}");
        cx.global_mut::<Theme>().high_contrast = on;
        Self::fade_to_target(cx);
    }

    /// Chart hues that stay apart for color-blind eyes, in Mok's own palettes.
    pub fn set_color_blind_safe(on: bool, cx: &mut App) {
        log::info!("theme: color-blind safe charts -> {on}");
        cx.global_mut::<Theme>().color_blind_safe = on;
        Self::fade_to_target(cx);
    }

    /// The owner's palette for `mode`, as an edited or imported theme gives, or none to bring Mok's back. The colors fade to it when `mode` shows. High contrast and color-blind safe charts change Mok's own palettes alone.
    pub fn set_palette(mode: Mode, palette: Option<Palette>, cx: &mut App) {
        log::info!(
            "theme: {mode:?} palette -> {}",
            if palette.is_some() {
                "the owner's"
            } else {
                "Mok's"
            }
        );
        cx.global_mut::<Theme>().custom[mode as usize] = palette;
        Self::fade_to_target(cx);
    }

    /// Sets light and dark palettes from a named preset (e.g., "zinc", "nord", "catppuccin", "tokyo-night").
    pub fn set_preset(name: &str, cx: &mut App) {
        let high_contrast = cx.global::<Theme>().high_contrast();
        if let Some(light) = Palette::preset(name, false, high_contrast) {
            let dark = Palette::preset(name, true, high_contrast).unwrap_or_else(|| light.clone());
            cx.global_mut::<Theme>().custom = [Some(light), Some(dark)];
            Self::fade_to_target(cx);
        }
    }

    pub const PRESET_NAMES: &'static [&'static str] = baked::BAKED_PRESET_NAMES;

    /// The palette the shown mode fades to: the owner's, or Mok's.
    pub fn palette(&self) -> Palette {
        self.target()
    }

    /// Edits non-color tokens, then repaints.
    pub fn update(cx: &mut App, edit: impl FnOnce(&mut Theme)) {
        edit(cx.global_mut::<Theme>());
        cx.refresh_windows();
    }

    fn target(&self) -> Palette {
        if let Some(palette) = &self.custom[self.mode as usize] {
            return palette.clone();
        }
        let mut palette = match self.mode {
            Mode::Light => Palette::light(self.high_contrast),
            Mode::Dark => Palette::dark(self.high_contrast),
        };
        if self.color_blind_safe {
            palette.chart = palette::color_blind_chart(self.mode);
        }
        palette
    }

    fn fade_to_target(cx: &mut App) {
        let duration = motion::duration(motion::THEME, cx);
        let theme = cx.global_mut::<Theme>();
        theme.transition += 1;
        let (id, from, to) = (theme.transition, theme.colors.clone(), theme.target());
        let start = Instant::now();
        cx.spawn(async move |cx| {
            loop {
                cx.background_executor().timer(FRAME).await;
                let t = (start.elapsed().as_secs_f32() / duration.as_secs_f32()).min(1.0);
                let running = cx.update(|cx| {
                    let theme = cx.global_mut::<Theme>();
                    if theme.transition != id {
                        return false;
                    }
                    theme.colors = from.mix(&to, motion::ease_in_out_cubic(t));
                    cx.refresh_windows();
                    t < 1.0
                });
                if !running {
                    return;
                }
            }
        })
        .detach();
    }
}
