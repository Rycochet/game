//! How many are in a stack: written at the bottom of its frame, in the middle,
//! light with a black outline so it can be read over any picture.

use bevy::prelude::*;
use shared::item::Shape;

use super::super::spawn_outlined_text;
use super::{CELL, ITEM_LABEL_FONT};

/// How far above the bottom of the frame the number sits, inside its edge.
const ABOVE_EDGE: f32 = 1.0;

/// The count over an item's cells, if there are more than one. `edge` is how thick
/// its frame is. None of it is what the pointer is on (it isn't `dom::Solid`), so presses and
/// names go to the cells beneath.
pub(super) fn spawn_count(
    parent: &mut ChildSpawnerCommands,
    count: u32,
    shape: Shape,
    corner: (f32, f32),
    edge: f32,
) {
    if count < 2 {
        return;
    }
    let words = count.to_string();
    let across = Node {
        position_type: PositionType::Absolute,
        left: px(corner.0 * CELL),
        top: px(corner.1 * CELL),
        width: px(shape.width as f32 * CELL),
        height: px(shape.height as f32 * CELL),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::FlexEnd,
        align_items: AlignItems::Center,
        padding: UiRect::bottom(px(edge + ABOVE_EDGE)),
        ..default()
    };
    parent.spawn(across).with_children(|strip| {
        spawn_outlined_text(strip, &words, ITEM_LABEL_FONT);
    });
}
