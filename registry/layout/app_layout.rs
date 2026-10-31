use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div,
};
use smallvec::SmallVec;

use crate::theme::ActiveTheme;

/// A top-level application shell with optional header, sidebar, footer, and main content.
#[derive(IntoElement)]
pub struct AppLayout {
    id: Option<ElementId>,
    header: Option<AnyElement>,
    sidebar: Option<AnyElement>,
    sidebar_right: Option<AnyElement>,
    footer: Option<AnyElement>,
    children: SmallVec<[AnyElement; 2]>,
}

impl Default for AppLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl AppLayout {
    /// Creates a new application layout shell.
    pub fn new() -> Self {
        Self {
            id: None,
            header: None,
            sidebar: None,
            sidebar_right: None,
            footer: None,
            children: SmallVec::new(),
        }
    }

    /// Sets the layout element ID.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the top header or titlebar area.
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    /// Sets the left primary navigation sidebar.
    pub fn sidebar(mut self, sidebar: impl IntoElement) -> Self {
        self.sidebar = Some(sidebar.into_any_element());
        self
    }

    /// Sets an optional right secondary sidebar or inspector pane.
    pub fn sidebar_right(mut self, sidebar: impl IntoElement) -> Self {
        self.sidebar_right = Some(sidebar.into_any_element());
        self
    }

    /// Sets the bottom status bar or footer.
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }
}

impl ParentElement for AppLayout {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for AppLayout {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;

        let mut root = div()
            .id(self.id.unwrap_or_else(|| ("app-layout").into()))
            .size_full()
            .flex()
            .flex_col()
            .bg(colors.bg)
            .text_color(colors.fg);

        if let Some(header) = self.header {
            root = root.child(
                div()
                    .w_full()
                    .flex_none()
                    .border_b_1()
                    .border_color(colors.border)
                    .bg(colors.surface)
                    .child(header),
            );
        }

        let mut middle = div().w_full().flex_1().flex().flex_row().overflow_hidden();

        if let Some(sidebar) = self.sidebar {
            middle = middle.child(
                div()
                    .h_full()
                    .flex_none()
                    .border_r_1()
                    .border_color(colors.border)
                    .child(sidebar),
            );
        }

        let main_content = div()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .children(self.children);
        middle = middle.child(main_content);

        if let Some(sidebar_right) = self.sidebar_right {
            middle = middle.child(
                div()
                    .h_full()
                    .flex_none()
                    .border_l_1()
                    .border_color(colors.border)
                    .child(sidebar_right),
            );
        }

        root = root.child(middle);

        if let Some(footer) = self.footer {
            root = root.child(
                div()
                    .w_full()
                    .flex_none()
                    .border_t_1()
                    .border_color(colors.border)
                    .bg(colors.surface)
                    .child(footer),
            );
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_layout_builds_with_header_sidebars_and_footer() {
        let layout = AppLayout::new()
            .id("test-shell")
            .header(div().child("Header"))
            .sidebar(div().child("Left Sidebar"))
            .sidebar_right(div().child("Right Sidebar"))
            .footer(div().child("Footer"))
            .child(div().child("Content"));

        assert!(layout.id.is_some());
        assert!(layout.header.is_some());
        assert!(layout.sidebar.is_some());
        assert!(layout.sidebar_right.is_some());
        assert!(layout.footer.is_some());
        assert_eq!(layout.children.len(), 1);
    }
}
