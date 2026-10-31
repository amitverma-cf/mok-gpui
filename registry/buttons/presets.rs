use gpui::ElementId;

use super::IconButton;
use crate::primitives::LucideIcon;

/// Closes a panel, dialog or tab.
pub fn close_button(id: impl Into<ElementId>) -> IconButton {
    IconButton::new(id, LucideIcon::X).tooltip("Close")
}

/// Returns to the previous view.
pub fn back_button(id: impl Into<ElementId>) -> IconButton {
    IconButton::new(id, LucideIcon::ArrowLeft).tooltip("Back")
}

/// Opens more actions.
pub fn more_button(id: impl Into<ElementId>) -> IconButton {
    IconButton::new(id, LucideIcon::Ellipsis).tooltip("More")
}
