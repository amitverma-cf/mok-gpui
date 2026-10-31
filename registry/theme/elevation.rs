use gpui::{BoxShadow, Hsla, px};

use super::{Mode, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elevation {
    Raised,
    Floating,
    Modal,
}

impl Theme {
    pub fn elevation(&self, level: Elevation) -> Vec<BoxShadow> {
        let dark = self.mode == Mode::Dark;
        let layers: &[(f32, f32, f32)] = match (level, dark) {
            (Elevation::Raised, false) => &[(1.0, 2.0, 0.06)],
            (Elevation::Floating, false) => &[(1.0, 2.0, 0.04), (6.0, 16.0, 0.08)],
            (Elevation::Modal, false) => &[(2.0, 4.0, 0.04), (16.0, 40.0, 0.14)],
            (Elevation::Raised, true) => &[(1.0, 2.0, 0.4)],
            (Elevation::Floating, true) => &[(2.0, 4.0, 0.3), (8.0, 24.0, 0.45)],
            (Elevation::Modal, true) => &[(4.0, 8.0, 0.3), (20.0, 48.0, 0.55)],
        };
        layers
            .iter()
            .map(|&(y, blur, alpha)| shadow(self.colors.shadow.alpha(alpha), y, blur))
            .collect()
    }
}

fn shadow(color: Hsla, y: f32, blur: f32) -> BoxShadow {
    BoxShadow::new(px(0.0), px(y), color).blur_radius(px(blur))
}
