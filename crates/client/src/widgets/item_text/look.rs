//! What fills an item's square in writing, as far as it is known: its picture
//! kept to its proportions, its kind's colour if it has none, a faint square
//! while it is asked after, or a grey "?" if nobody knows it.

use bevy::prelude::*;
use shared::item::{ItemKind, Rotation};

use super::tiles::UNKNOWN;
use crate::common::KnownItem;
use crate::item_art::{Look, look_of};
use crate::widgets::{
    Floating, PICTURE_PIXELS_PER_CELL, item_colour, item_picture, picture_in_square,
};

/// The tint of an item being asked after.
const LOADING: Color = Color::srgba(0.25, 0.17, 0.1, 0.18);

/// What a frame shows, to know when to change it.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub(super) enum Shows {
    #[default]
    Loading,
    Unknown,
    Plain(ItemKind),
    Picture(Look),
}

impl Shows {
    pub fn of(known: Option<&KnownItem>) -> Self {
        match known {
            Some(KnownItem::Found(item, _)) => {
                let top = item.top();
                look_of(top).map_or(Self::Plain(top.kind), Self::Picture)
            }
            Some(KnownItem::Unknown(_)) => Self::Unknown,
            Some(KnownItem::Asked(_)) | None => Self::Loading,
        }
    }

    /// Fills a frame with it.
    pub fn spawn(&self, frame: &mut ChildSpawnerCommands) {
        let filling = Node {
            width: percent(100),
            height: percent(100),
            ..default()
        };
        match self {
            Self::Loading => {
                frame.spawn((Floating, filling, BackgroundColor(LOADING)));
            }
            Self::Unknown => {
                frame.spawn((Floating, filling, ImageNode::new(UNKNOWN)));
            }
            Self::Plain(kind) => {
                frame.spawn((Floating, filling, BackgroundColor(item_colour(*kind))));
            }
            Self::Picture(look) => {
                let node = picture_in_square(look);
                let pixels = PICTURE_PIXELS_PER_CELL;
                frame.spawn(item_picture(look.clone(), pixels, node, Rotation::None));
            }
        }
    }
}
