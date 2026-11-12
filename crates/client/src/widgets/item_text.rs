//! Writing with items in it (`${item:<uuid>}`, `shared::item_text`): each item
//! shown as its picture in a frame a line tall, in among the words, named on
//! hovering.
//! `ItemText` shows such writing; text boxes that take items use the same parts.

mod display;
mod frame;
mod hover;
mod inline;
mod look;
mod tiles;

use bevy::prelude::*;

use super::TextSize;
pub(super) use frame::spawn_inline_item;
pub(super) use inline::InlineHeight;

/// Writing to show, items and all (component). Change `0` to change it.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemText(pub String);

/// What makes a `Text` (with its font and colour) show this writing with its
/// items as pictures: for writing spawned some other way, as markup's is.
pub fn item_text(text: impl Into<String>, size: TextSize) -> impl Bundle {
    let height = InlineHeight(super::text_box::line_height(size));
    (ItemText(text.into()), height)
}

pub(super) fn add(app: &mut App) {
    app.add_systems(Startup, tiles::make_tiles).add_systems(
        Update,
        (
            display::show_item_texts,
            inline::want_inline_items,
            hover::name_inline_items,
            frame::add_frames,
            frame::place_frames,
        )
            .chain()
            .after(super::text_box::show_text_boxes),
    );
}
