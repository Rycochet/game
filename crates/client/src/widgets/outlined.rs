//! Light words with a black outline, so they can be read over any picture: a
//! stack's count, an action bar slot's key.

use bevy::prelude::*;

use super::{Floating, TextSize};

/// How far round the words their outline reaches, in logical pixels.
const OUTLINE: f32 = 1.0;
const WORDS: Color = Color::WHITE;
const OUTLINE_COLOUR: Color = Color::BLACK;

/// The words, white with a black outline, in a node of their own. None of it is
/// what the pointer is on (`Floating`), so presses and names go to what is beneath.
pub fn spawn_outlined_text(
    parent: &mut ChildSpawnerCommands,
    words: &str,
    size: TextSize,
) -> Entity {
    parent
        .spawn((Floating, Node::default()))
        .with_children(|holder| {
            // The outline: the words in black, nudged each way, behind them.
            for dx in [-OUTLINE, 0.0, OUTLINE] {
                for dy in [-OUTLINE, 0.0, OUTLINE] {
                    if (dx, dy) == (0.0, 0.0) {
                        continue;
                    }
                    holder.spawn((
                        Floating,
                        Text::new(words),
                        size.font(),
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
            holder.spawn((Floating, Text::new(words), size.font(), TextColor(WORDS)));
        })
        .id()
}
