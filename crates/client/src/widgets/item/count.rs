//! How many are in a stack: written at the bottom of its frame, in the middle,
//! light with a black outline so it can be read over any picture.

use bevy::prelude::*;
use shared::item::Shape;

use super::super::Floating;
use super::{CELL, ITEM_LABEL_FONT};

/// How far round the number its outline reaches, in logical pixels.
const OUTLINE: f32 = 1.0;
/// How far above the bottom of the frame the number sits, inside its edge.
const ABOVE_EDGE: f32 = 1.0;
const NUMBER: Color = Color::WHITE;
const OUTLINE_COLOUR: Color = Color::BLACK;

/// The count over an item's cells, if there are more than one. `edge` is how thick
/// its frame is. None of it is what the pointer is on (`Floating`), so presses and
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
    parent.spawn((Floating, across)).with_children(|strip| {
        strip
            .spawn((Floating, Node::default()))
            .with_children(|number| {
                // The outline: the number in black, nudged each way, behind it.
                for dx in [-OUTLINE, 0.0, OUTLINE] {
                    for dy in [-OUTLINE, 0.0, OUTLINE] {
                        if (dx, dy) == (0.0, 0.0) {
                            continue;
                        }
                        number.spawn((
                            Floating,
                            Text::new(words.clone()),
                            ITEM_LABEL_FONT.font(),
                            TextColor(OUTLINE_COLOUR),
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(dx),
                                top: px(dy),
                                ..default()
                            },
                        ));
                    }
                }
                number.spawn((
                    Floating,
                    Text::new(words),
                    ITEM_LABEL_FONT.font(),
                    TextColor(NUMBER),
                ));
            });
    });
}
