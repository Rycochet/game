//! What an item looks like, its name, colour, label and edge.

use bevy::prelude::*;
use shared::item::{Item, ItemKind, Quality, Rotation};

use super::super::ITEM_EDGE;
use super::super::aura::Aura;
use super::{CELL_BORDER, QUALITY_BORDER};
use crate::common::quality_colour;
use crate::item_art::{Look, look_of};

/// Follows the pointer above everything: not what the pointer is on.
#[derive(Component, Clone, Copy)]
pub struct Floating;

/// The name to show this frame, if any. Whoever knows what the pointer is on sets
/// it in `Update`, and it is cleared once shown.
#[derive(Resource, Default)]
pub struct ItemTooltip(pub Option<ItemTip>);

pub struct ItemTip {
    pub words: String,
    pub quality: Option<Quality>,
    /// Gets a metal edge.
    pub set: bool,
    /// Where its top left goes, in the interface's pixels (`UiPointer`'s).
    pub at: Vec2,
}

/// On an item's cells and label for its name to show when the pointer is on it,
/// for items whose owner has nothing more to say about them.
#[derive(Component, Clone)]
pub struct ItemName {
    pub words: String,
    pub quality: Option<Quality>,
    pub set: bool,
}

/// What an item looks like.
#[derive(Clone, Debug)]
pub struct ItemLook {
    pub kind: ItemKind,
    pub quality: Option<Quality>,
    pub set: bool,
    /// Its picture, if its kind has a model, and how it is turned.
    pub picture: Option<Look>,
    pub rotation: Rotation,
    /// How it glows, if it does.
    pub aura: Option<Aura>,
    /// How many there are of it: more than one for a stack.
    pub count: u32,
    /// A stack with something in it known to differ.
    pub mixed: bool,
}

impl ItemLook {
    /// How it looks, by the top of a stack, with how many there are.
    pub fn of(item: &Item) -> Self {
        let (count, mixed) = (item.count(), !item.is_even());
        let item = item.top();
        Self {
            kind: item.kind,
            quality: item.quality,
            set: item.set,
            picture: look_of(item),
            rotation: Rotation::None,
            aura: Aura::of(item),
            count,
            mixed,
        }
    }

    /// Turned this way.
    pub fn turned(self, rotation: Rotation) -> Self {
        Self { rotation, ..self }
    }
}

#[derive(Component)]
pub(in crate::widgets) struct NameBox;

#[derive(Component)]
pub(in crate::widgets) struct NameText;

/// The metal edge of the box, for a set item's name.
#[derive(Component)]
pub(in crate::widgets) struct NameEdge;

/// What a kind of item looks like until it has a picture: its colour.
pub fn item_colour(kind: ItemKind) -> Color {
    let index = ItemKind::ALL
        .iter()
        .position(|&other| other == kind)
        .unwrap_or(0);
    // Spread round the colour wheel by a step that doesn't repeat soon.
    Color::hsl((index as f32 * 67.0) % 360.0, 0.45, 0.5)
}

/// What it says on the first cell of an item.
pub fn item_label(kind: ItemKind) -> String {
    kind.def().name.chars().take(2).collect()
}

/// The colour of the edge of an item of this quality.
pub fn item_edge(quality: Option<Quality>) -> Color {
    quality.map_or(ITEM_EDGE, quality_colour)
}

/// How thick the edge of an item of this quality is.
pub fn item_edge_width(quality: Option<Quality>) -> f32 {
    if quality.is_some() {
        QUALITY_BORDER
    } else {
        CELL_BORDER
    }
}
