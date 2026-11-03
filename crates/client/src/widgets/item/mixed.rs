//! The mark on a mixed stack, one with something in it known to differ: a red "!" in
//! its top right corner, inside its frame.

use bevy::prelude::*;
use shared::item::Shape;

use super::super::Floating;
use super::{CELL, ITEM_LABEL_FONT};

/// How big the mark is, across, in logical pixels.
const SIZE: f32 = 14.0;
const BACKGROUND: Color = Color::srgb(0.7, 0.12, 0.1);
const EDGE: Color = Color::BLACK;
const SIGN: Color = Color::WHITE;

/// The mark over an item's cells, if it is a mixed stack. `edge` is how thick its
/// frame is. Like the count, it isn't what the pointer is on.
pub(super) fn spawn_mixed_mark(
    parent: &mut ChildSpawnerCommands,
    mixed: bool,
    shape: Shape,
    corner: (f32, f32),
    edge: f32,
) {
    if !mixed {
        return;
    }
    let right = (corner.0 + shape.width as f32) * CELL - edge;
    parent
        .spawn((
            Floating,
            Node {
                position_type: PositionType::Absolute,
                left: px(right - SIZE),
                top: px(corner.1 * CELL + edge),
                width: px(SIZE),
                height: px(SIZE),
                border: UiRect::all(px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BACKGROUND),
            BorderColor::from(EDGE),
        ))
        .with_child((
            Floating,
            Text::new("!"),
            ITEM_LABEL_FONT.font(),
            TextColor(SIGN),
        ));
}
