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

pub(super) fn words(text: impl Into<String>) -> impl Bundle {
    (Text::new(text), FONT.font(), TextColor(INK))
}
