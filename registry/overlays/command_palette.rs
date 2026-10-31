use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, App, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::*,
};

use crate::{
    forms::Run,
    layout::on_axis,
    motion,
    primitives::{Backdrop, FocusScope, Icon, LucideIcon, Place, give_back, take_focus},
    theme::{ActiveTheme, Elevation, Radius, TextSize},
};

pub type OnQueryChange = Rc<dyn Fn(&str, &mut Window, &mut App)>;

/// An individual action item in the command palette.
#[derive(Clone)]
pub struct CommandItem {
    pub id: ElementId,
    pub title: SharedString,
    pub description: Option<SharedString>,
    pub icon: Option<LucideIcon>,
    pub shortcut: Option<SharedString>,
    pub action: Option<Run>,
}

impl CommandItem {
    /// Creates a new command item.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            icon: None,
            shortcut: None,
            action: None,
        }
    }

    /// Sets an optional description line.
    pub fn description(mut self, desc: impl Into<SharedString>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets an optional leading icon.
    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets a keyboard shortcut badge string (e.g. "⌘K" or "Ctrl+P").
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Attaches an action callback executed on selection.
    pub fn on_select(mut self, action: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.action = Some(Rc::new(action));
        self
    }
}

/// A grouped section of command items with a header title.
#[derive(Clone)]
pub struct CommandGroup {
    pub heading: SharedString,
    pub items: Vec<CommandItem>,
}

impl CommandGroup {
    /// Creates a new command group.
    pub fn new(heading: impl Into<SharedString>, items: Vec<CommandItem>) -> Self {
        Self {
            heading: heading.into(),
            items,
        }
    }
}

/// A modal command palette / quick search dialog.
#[derive(IntoElement)]
pub struct CommandPalette {
    id: ElementId,
    query: SharedString,
    placeholder: SharedString,
    groups: Vec<CommandGroup>,
    selected_index: usize,
    on_close: Run,
    on_query_change: Option<OnQueryChange>,
}

impl CommandPalette {
    /// Creates a new command palette overlay.
    pub fn new(
        id: impl Into<ElementId>,
        on_close: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            query: "".into(),
            placeholder: "Type a command or search...".into(),
            groups: Vec::new(),
            selected_index: 0,
            on_close: Rc::new(on_close),
            on_query_change: None,
        }
    }

    /// Sets the search input query string.
    pub fn query(mut self, query: impl Into<SharedString>) -> Self {
        self.query = query.into();
        self
    }

    /// Sets placeholder prompt text.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Adds a group of commands.
    pub fn group(mut self, group: CommandGroup) -> Self {
        self.groups.push(group);
        self
    }

    /// Sets the currently highlighted index among filtered items.
    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    /// Callback when search query changes.
    pub fn on_query_change(
        mut self,
        callback: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_query_change = Some(Rc::new(callback));
        self
    }
}

impl RenderOnce for CommandPalette {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let takeover = take_focus((self.id.clone(), "takeover"), window, cx);
        let on_close = self.on_close.clone();
        let close: Run = {
            let takeover = takeover.clone();
            Rc::new(move |window, cx| {
                give_back(&takeover, window, cx);
                on_close(window, cx);
            })
        };

        let theme = cx.theme();
        let colors = &theme.colors;
        let query_lower = self.query.to_ascii_lowercase();

        // Filter command items matching query
        let mut visible_groups: Vec<(SharedString, Vec<CommandItem>)> = Vec::new();
        let mut flat_items: Vec<CommandItem> = Vec::new();

        for group in &self.groups {
            let matched: Vec<CommandItem> = group
                .items
                .iter()
                .filter(|item| {
                    if query_lower.is_empty() {
                        true
                    } else {
                        item.title.to_ascii_lowercase().contains(&query_lower)
                            || item
                                .description
                                .as_ref()
                                .is_some_and(|d| d.to_ascii_lowercase().contains(&query_lower))
                    }
                })
                .cloned()
                .collect();

            if !matched.is_empty() {
                flat_items.extend(matched.clone());
                visible_groups.push((group.heading.clone(), matched));
            }
        }

        let total_items = flat_items.len();
        let selected = if total_items == 0 {
            0
        } else {
            self.selected_index.min(total_items - 1)
        };

        let escape = close.clone();
        let dismiss = close.clone();

        let top_offset = theme.titlebar_height().to_pixels(window.rem_size()) * 2.0;

        let content = div()
            .id(self.id.clone())
            .role(Role::Dialog)
            .aria_label("Command Palette")
            .flex()
            .flex_col()
            .w(theme.dialog_width())
            .max_h(window.viewport_size().height - top_offset * 2.0)
            .rounded(theme.radius(Radius::Lg))
            .bg(colors.overlay)
            .border_1()
            .border_color(colors.border)
            .shadow(theme.elevation(Elevation::Modal))
            .overflow_hidden()
            // Search Input Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(colors.border)
                    .child(Icon::new(LucideIcon::Search).color(colors.fg_muted))
                    .child(
                        div()
                            .flex_1()
                            .text_size(theme.text_size(TextSize::Base))
                            .text_color(if self.query.is_empty() {
                                colors.fg_subtle
                            } else {
                                colors.fg
                            })
                            .child(if self.query.is_empty() {
                                self.placeholder.clone()
                            } else {
                                self.query.clone()
                            }),
                    )
                    .child(
                        div()
                            .px_1p5()
                            .py_0p5()
                            .rounded(theme.radius(Radius::Sm))
                            .bg(colors.sunken)
                            .border_1()
                            .border_color(colors.border)
                            .text_size(theme.text_size(TextSize::Xs))
                            .text_color(colors.fg_subtle)
                            .child("ESC"),
                    ),
            )
            // Results List
            .child(
                on_axis(div().id((self.id.clone(), "results")))
                    .flex_1()
                    .overflow_y_scroll()
                    .p_2()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .when(total_items == 0, |list| {
                        list.child(
                            div()
                                .py_8()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(theme.text_size(TextSize::Sm))
                                .text_color(colors.fg_muted)
                                .child("No commands found"),
                        )
                    })
                    .children(visible_groups.into_iter().map(|(heading, items)| {
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .px_2()
                                    .pt_1()
                                    .pb_0p5()
                                    .text_size(theme.text_size(TextSize::Xs))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors.fg_muted)
                                    .child(heading),
                            )
                            .children(items.into_iter().enumerate().map(|(ix, item)| {
                                let is_active = ix == selected;
                                let item_action = item.action.clone();
                                let item_close = close.clone();

                                div()
                                    .id(item.id)
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap_3()
                                    .px_3()
                                    .py_2()
                                    .rounded(theme.radius(Radius::Md))
                                    .cursor_pointer()
                                    .when(is_active, |el| el.bg(colors.hover))
                                    .hover(|s| s.bg(colors.hover))
                                    .on_click(move |_, window, cx| {
                                        if let Some(act) = &item_action {
                                            act(window, cx);
                                        }
                                        item_close(window, cx);
                                    })
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2p5()
                                            .children(item.icon.map(|ic| {
                                                Icon::new(ic).color(if is_active {
                                                    colors.fg
                                                } else {
                                                    colors.fg_muted
                                                })
                                            }))
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .child(
                                                        div()
                                                            .text_size(
                                                                theme.text_size(TextSize::Sm),
                                                            )
                                                            .font_weight(FontWeight::MEDIUM)
                                                            .text_color(colors.fg)
                                                            .child(item.title),
                                                    )
                                                    .children(item.description.map(|d| {
                                                        div()
                                                            .text_size(
                                                                theme.text_size(TextSize::Xs),
                                                            )
                                                            .text_color(colors.fg_muted)
                                                            .child(d)
                                                    })),
                                            ),
                                    )
                                    .children(item.shortcut.map(|sc| {
                                        div()
                                            .px_1p5()
                                            .py_0p5()
                                            .rounded(theme.radius(Radius::Sm))
                                            .bg(colors.surface)
                                            .text_size(theme.text_size(TextSize::Xs))
                                            .text_color(colors.fg_subtle)
                                            .child(sc)
                                    }))
                            }))
                    })),
            )
            // Footer helper line
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .border_t_1()
                    .border_color(colors.border)
                    .bg(colors.surface)
                    .text_size(theme.text_size(TextSize::Xs))
                    .text_color(colors.fg_subtle)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child("Navigation: ↑↓")
                            .child("Select: ↵")
                            .child("Close: esc"),
                    ),
            );

        let arrive =
            Animation::new(motion::duration(motion::BASE, cx)).with_easing(motion::ease_out_cubic);
        let card =
            div()
                .child(content)
                .with_animation((self.id.clone(), "in"), arrive, |card, t| {
                    card.opacity(t).pt(motion::NUDGE * (1.0 - t))
                });

        let layer = div()
            .on_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape" {
                    cx.stop_propagation();
                    escape(window, cx);
                }
            })
            .child(
                Backdrop::new((self.id.clone(), "scrim"))
                    .place(Place::Top)
                    .on_dismiss(move |window, cx| dismiss(window, cx))
                    .child(div().pt(top_offset).child(card)),
            );

        FocusScope::new(&takeover.read(cx).focus).child(layer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_palette_builds_and_groups() {
        let item1 = CommandItem::new("item-1", "Open Settings")
            .description("Open settings panel")
            .shortcut("⌘,");
        let item2 = CommandItem::new("item-2", "New File").shortcut("⌘N");

        let group = CommandGroup::new("General", vec![item1, item2]);
        let palette = CommandPalette::new("test-palette", |_, _| {})
            .query("settings")
            .placeholder("Search...")
            .group(group)
            .selected_index(0);

        assert_eq!(palette.groups.len(), 1);
        assert_eq!(palette.groups[0].items.len(), 2);
        assert_eq!(palette.query, "settings");
    }
}
