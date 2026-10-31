use gpui::{Hsla, Rgba, rgb, rgba};

use super::Mode;

pub trait Mix {
    fn mix(&self, to: &Self, t: f32) -> Self;
}

impl Mix for Hsla {
    fn mix(&self, to: &Self, t: f32) -> Self {
        if t <= 0.0 {
            return *self;
        }
        if t >= 1.0 {
            return *to;
        }
        let (a, b) = (self.to_rgb(), to.to_rgb());
        let lerp = |x: f32, y: f32| x + (y - x) * t;
        Rgba {
            r: lerp(a.r, b.r),
            g: lerp(a.g, b.g),
            b: lerp(a.b, b.b),
            a: lerp(a.a, b.a),
        }
        .into()
    }
}

impl<const N: usize> Mix for [Hsla; N] {
    fn mix(&self, to: &Self, t: f32) -> Self {
        std::array::from_fn(|i| self[i].mix(&to[i], t))
    }
}

macro_rules! mixable {
    ($(#[$meta:meta])* $name:ident { $($color:ident),* $(,)? ; $($field:ident: $ty:ty),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            $(pub $color: Hsla,)*
            $(pub $field: $ty,)*
        }

        impl Mix for $name {
            fn mix(&self, to: &Self, t: f32) -> Self {
                Self {
                    $($color: self.$color.mix(&to.$color, t),)*
                    $($field: self.$field.mix(&to.$field, t),)*
                }
            }
        }

        impl $name {
            /// The names of its colors, each a field's.
            pub const NAMES: &'static [&'static str] = &[$(stringify!($color)),*];

            /// The color named `token`, a field's name; any other name fails.
            pub fn token(&self, token: &str) -> Hsla {
                match token {
                    $(stringify!($color) => self.$color,)*
                    other => panic!("no {} color {other}", stringify!($name).to_lowercase()),
                }
            }

            /// The color named `token`, to change; any other name fails.
            pub fn token_mut(&mut self, token: &str) -> &mut Hsla {
                match token {
                    $(stringify!($color) => &mut self.$color,)*
                    other => panic!("no {} color {other}", stringify!($name).to_lowercase()),
                }
            }
        }
    };
}

mixable!(
    /// Code token colors.
    Syntax {
        keyword,
        string,
        number,
        comment,
        function,
        type_name,
        constant,
        property,
        tag,
        attribute,
        operator,
        punctuation,
        variable,
        ;
    }
);

mixable!(
    /// Semantic colors. Components read these, never literals.
    Palette {
        bg,
        surface,
        sunken,
        overlay,
        hover,
        active,
        border,
        border_strong,
        fg,
        fg_muted,
        fg_subtle,
        fg_disabled,
        accent,
        accent_hover,
        on_accent,
        on_media,
        focus,
        link,
        selection,
        success,
        warning,
        danger,
        info,
        success_subtle,
        warning_subtle,
        danger_subtle,
        info_subtle,
        backdrop,
        media_backdrop,
        shimmer,
        glass,
        shadow,
        tooltip_bg,
        tooltip_fg,
        paper,
        ink,
        ;
        chart: [Hsla; 8],
        ansi: [Hsla; 16],
        syntax: Syntax,
    }
);

fn c(hex: u32) -> Hsla {
    rgb(hex).into()
}

fn c8<const N: usize>(hex: [u32; N]) -> [Hsla; N] {
    hex.map(c)
}

/// The chart's hues by name, for what wears one: an event, a label.
pub const HUE_NAMES: [&str; 8] = [
    "Blue", "Teal", "Ochre", "Rose", "Violet", "Green", "Rust", "Cyan",
];

/// Chart hues each 3:1 on its mode's page that stay apart for protanopia, deuteranopia and tritanopia; `HUE_NAMES` still name them.
pub fn color_blind_chart(mode: Mode) -> [Hsla; 8] {
    match mode {
        Mode::Light => c8([
            0x0240b1, 0x08a399, 0xcd8017, 0x844954, 0xa276e3, 0x184606, 0xb84f10, 0x20789d,
        ]),
        Mode::Dark => c8([
            0x3471e3, 0x57b6b2, 0xf6a537, 0xde545a, 0x9d8dec, 0xc4ecb7, 0xffd93d, 0x52dafe,
        ]),
    }
}

impl Palette {
    /// The chart color at `hue`; a hue past the chart fails, naming `owner`.
    pub fn hue(&self, hue: usize, owner: impl std::fmt::Display) -> Hsla {
        *self
            .chart
            .get(hue)
            .unwrap_or_else(|| panic!("{owner}: no chart hue {hue}"))
    }

    pub fn from_baked(p: &super::baked::BakedPalette, is_dark: bool, high_contrast: bool) -> Self {
        let (chart, ansi, syntax) = if is_dark {
            (
                c8([
                    0x79a7e2, 0x4eb9ad, 0xcd995c, 0xdc8a90, 0xa998dd, 0x7cb57d, 0xd9906f, 0x56b2d4,
                ]),
                c8([
                    0x3f3f3f, 0xe1897f, 0x6fb880, 0xc99d4e, 0x74a7e8, 0xc38ecf, 0x35b9c0, 0xafadab,
                    0x5c5a58, 0xfda297, 0x89d298, 0xe3b667, 0x8dc1ff, 0xdda7ea, 0x56d3da, 0xf3f1f0,
                ]),
                Syntax {
                    keyword: c(0xbba3e8),
                    string: c(0x8ac596),
                    number: c(0xeaab78),
                    comment: c(0xacaaa8),
                    function: c(0x85b6e9),
                    type_name: c(0x84c9c8),
                    constant: c(0xeda382),
                    property: c(0xe29fa9),
                    tag: c(0xe29fa9),
                    attribute: c(0xe1b671),
                    operator: c(0xafadab),
                    punctuation: c(0xafadab),
                    variable: c(0xf3f1f0),
                },
            )
        } else {
            (
                c8([
                    0x5182c1, 0x009589, 0xaa732b, 0xb8636b, 0x8572bb, 0x539156, 0xb66946, 0x138db1,
                ]),
                c8([
                    0x181613, 0xa34942, 0x287c42, 0x8c5f00, 0x346aac, 0x875093, 0x007c84, 0xacaaa8,
                    0x605d5a, 0xbd6158, 0x439458, 0xa57710, 0x4c82c6, 0xa067ac, 0x00959c, 0xcccac8,
                ]),
                Syntax {
                    keyword: c(0x714ca6),
                    string: c(0x317a45),
                    number: c(0xa3591b),
                    comment: c(0x696764),
                    function: c(0x2863ab),
                    type_name: c(0x1e7777),
                    constant: c(0xaa5529),
                    property: c(0xa14759),
                    tag: c(0xa14759),
                    attribute: c(0x9f5d12),
                    operator: c(0x605d5a),
                    punctuation: c(0x605d5a),
                    variable: c(0x181613),
                },
            )
        };

        let mut palette = Self {
            bg: c(p.bg),
            surface: c(p.surface),
            sunken: c(p.sunken),
            overlay: c(p.overlay),
            hover: c(p.hover),
            active: c(p.active),
            border: c(p.border),
            border_strong: c(p.border_strong),
            fg: c(p.fg),
            fg_muted: c(p.fg_muted),
            fg_subtle: c(p.fg_subtle),
            fg_disabled: c(p.fg_disabled),
            accent: c(p.accent),
            accent_hover: c(p.accent_hover),
            on_accent: c(p.on_accent),
            on_media: c(p.on_media),
            focus: c(p.focus),
            link: c(p.link),
            selection: rgba(p.selection).into(),
            success: c(p.success),
            warning: c(p.warning),
            danger: c(p.danger),
            info: c(p.info),
            success_subtle: c(p.success_subtle),
            warning_subtle: c(p.warning_subtle),
            danger_subtle: c(p.danger_subtle),
            info_subtle: c(p.info_subtle),
            backdrop: rgba(p.backdrop).into(),
            media_backdrop: rgba(p.media_backdrop).into(),
            shimmer: rgba(p.shimmer).into(),
            glass: rgba(p.glass).into(),
            shadow: c(p.shadow),
            tooltip_bg: c(p.tooltip_bg),
            tooltip_fg: c(p.tooltip_fg),
            paper: c(p.paper),
            ink: c(p.ink),
            chart,
            ansi,
            syntax,
        };

        if high_contrast {
            if is_dark {
                palette.fg_muted = c(0xcfcdcb);
                palette.fg_subtle = c(0xafadab);
                palette.border = c(0x6a6966);
                palette.border_strong = c(0x858380);
            } else {
                palette.fg_muted = c(0x44423f);
                palette.fg_subtle = c(0x575552);
                palette.border = c(0xa6a4a2);
                palette.border_strong = c(0x82807d);
            }
            palette.glass = palette.bg;
        }

        palette
    }

    pub fn light(high_contrast: bool) -> Self {
        Self::from_baked(&super::baked::BAKED_LIGHT_PALETTE, false, high_contrast)
    }

    pub fn dark(high_contrast: bool) -> Self {
        Self::from_baked(&super::baked::BAKED_DARK_PALETTE, true, high_contrast)
    }

    pub fn preset(name: &str, is_dark: bool, high_contrast: bool) -> Option<Self> {
        super::baked::get_baked_preset(name, is_dark)
            .map(|p| Self::from_baked(&p, is_dark, high_contrast))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_hits_both_ends_and_the_middle() {
        let (black, white) = (c(0x000000), c(0xffffff));
        assert_eq!(black.mix(&white, 0.0).to_rgb().r, 0.0);
        assert_eq!(black.mix(&white, 1.0).to_rgb().r, 1.0);
        assert!((black.mix(&white, 0.5).to_rgb().g - 0.5).abs() < 1e-4);
    }

    #[test]
    fn palette_mix_moves_every_field() {
        let (light, dark) = (Palette::light(false), Palette::dark(false));
        assert_eq!(light.mix(&dark, 1.0), dark);
        assert_eq!(light.mix(&dark, 0.0), light);
    }

    #[test]
    fn a_color_goes_by_its_name() {
        let palette = Palette::dark(false);
        assert_eq!(palette.token("fg_subtle"), palette.fg_subtle);
        assert_eq!(palette.syntax.token("keyword"), palette.syntax.keyword);
    }

    #[test]
    #[should_panic(expected = "no palette color ink_blot")]
    fn a_name_no_color_has_fails() {
        Palette::light(false).token("ink_blot");
    }

    #[test]
    fn high_contrast_strengthens_borders() {
        let (base, hc) = (Palette::light(false), Palette::light(true));
        assert!(hc.border.l < base.border.l);
        let (base, hc) = (Palette::dark(false), Palette::dark(true));
        assert!(hc.border.l > base.border.l);
    }
}
