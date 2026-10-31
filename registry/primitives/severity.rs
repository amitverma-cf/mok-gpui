use gpui::Hsla;

use super::LucideIcon;
use crate::theme::Palette;

/// How much a message matters, with the color and icon that say so.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Info,
    Success,
    Warning,
    Danger,
}

impl Severity {
    pub fn color(self, colors: &Palette) -> Hsla {
        match self {
            Self::Info => colors.info,
            Self::Success => colors.success,
            Self::Warning => colors.warning,
            Self::Danger => colors.danger,
        }
    }

    /// The quiet fill behind its icon or a banner.
    pub fn subtle(self, colors: &Palette) -> Hsla {
        match self {
            Self::Info => colors.info_subtle,
            Self::Success => colors.success_subtle,
            Self::Warning => colors.warning_subtle,
            Self::Danger => colors.danger_subtle,
        }
    }

    pub fn icon(self) -> LucideIcon {
        match self {
            Self::Info => LucideIcon::Info,
            Self::Success => LucideIcon::CircleCheck,
            Self::Warning => LucideIcon::TriangleAlert,
            Self::Danger => LucideIcon::CircleAlert,
        }
    }
}
