//! Keeps `ItemText`s on show: its words as spans, and its items as pictures
//! among them, made again whenever the writing changes.

use bevy::prelude::*;
use shared::item_text::{Piece, pieces};

use super::{InlineHeight, ItemText, spawn_inline_item};

pub(in crate::widgets) fn show_item_texts(
    mut commands: Commands,
    texts: Query<(Entity, &ItemText, &TextFont, &TextColor, &InlineHeight), Changed<ItemText>>,
) {
    for (entity, text, font, colour, height) in &texts {
        let mut shown = commands.entity(entity);
        shown.despawn_related::<Children>();
        shown.with_children(|spans| {
            for piece in pieces(&text.0) {
                match piece {
                    Piece::Text(words) => {
                        spans.spawn((TextSpan::new(words), font.clone(), *colour));
                    }
                    Piece::Item(id) => spawn_inline_item(spans, id, height.0, ()),
                }
            }
        });
    }
}
