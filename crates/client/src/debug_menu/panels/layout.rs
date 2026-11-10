//! Rows and words, for the panels' contents.

use bevy::prelude::*;

use super::FONT;
use crate::widgets::INK;

pub(super) fn row() -> Node {
    Node {
        column_gap: px(4),
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Takes a button out of its row, or puts it back: for those the player has
/// no permission for, which are hidden rather than greyed (owner).
pub(super) fn show_if(node: &mut Node, shown: bool) {
    let display = if shown { Display::Flex } else { Display::None };
    if node.display != display {
        node.display = display;
    }
}

pub(super) fn words(text: impl Into<String>) -> impl Bundle {
    (Text::new(text), FONT.font(), TextColor(INK))
}
