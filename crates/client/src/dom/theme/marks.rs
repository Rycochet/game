//! Quest markers' badges in markup (`span.mark-yellow`, `-blue`, `-orange`,
//! `-grey`): the "!" or "?" in its colour on the dark of the markers' rings,
//! as they read over doors (the Quest Log's list).

use bevy::prelude::*;

use super::{Look, Theme};
use crate::common::{QUEST_BLUE, QUEST_GREY, QUEST_ORANGE, QUEST_YELLOW};
use crate::dom::tag::Tag;

pub(super) fn add(theme: &mut Theme) {
    for (class, [r, g, b]) in [
        ("mark-yellow", QUEST_YELLOW),
        ("mark-blue", QUEST_BLUE),
        ("mark-orange", QUEST_ORANGE),
        ("mark-grey", QUEST_GREY),
    ] {
        let look = Look {
            colour: Some(Color::srgb_u8(r, g, b)),
            background: Some(Color::BLACK),
            ..default()
        };
        theme.set(Tag::Span, class, look);
    }
}
