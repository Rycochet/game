//! Rows that wrap, and words, for the panels' contents.

use bevy::prelude::*;

use super::FONT;
use crate::widgets::INK;

/// How wide the rows of buttons wrap, so a long list stays on the screen.
pub(super) const WRAPPED_WIDTH: f32 = 520.0;

pub(super) fn wrapped() -> Node {
    Node {
        width: px(WRAPPED_WIDTH),
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(4),
        row_gap: px(4),
        ..default()
    }
}

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
