#![cfg_attr(target_family = "wasm", no_main)]

mod ui;

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, IntoElement, ParentElement, Render, Styled,
    Window, WindowBounds, WindowOptions, px, size,
};
use ui::{
    assets::Assets,
    buttons::Button,
    primitives::FocusScope,
    theme::{ActiveTheme, Mode, Theme},
    typography::{Heading, Paragraph},
};

struct Home {
    focus: FocusHandle,
}

impl Render for Home {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let next = if theme.is_dark() {
            Mode::Light
        } else {
            Mode::Dark
        };
        FocusScope::new(&self.focus)
            .root()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .bg(theme.colors.bg)
            .child(Heading::h1("Hello from Mok"))
            .child(Paragraph::new(
                "These components are source files in src/ui. Edit them.",
            ))
            .child(
                Button::new("mode", format!("{next:?} mode"))
                    .on_click(move |_, _, cx| Theme::set_mode(next, cx)),
            )
    }
}

fn launch(cx: &mut App) {
    ui::init(cx);
    Theme::set_mode_now(Mode::Dark, cx);
    let bounds = Bounds::centered(None, size(px(640.0), px(400.0)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        ..Default::default()
    };
    cx.open_window(options, |window, cx| {
        cx.new(|cx| {
            let focus = cx.focus_handle();
            window.focus(&focus, cx);
            Home { focus }
        })
    })
    .expect("window failed to open");
    cx.activate(true);
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(launch);
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    let platform = std::rc::Rc::new(gpui_web::WebPlatform::new_with_backend_and_font_fallback(
        false,
        gpui_web::WebBackendPreference::Auto,
        gpui_web::CanvasFontFallback::EmojiAndCjk,
    ));
    gpui::Application::with_platform(platform)
        .with_assets(Assets)
        .run(launch);
}
