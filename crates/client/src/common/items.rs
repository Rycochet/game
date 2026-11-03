//! Items between modules: quality colours, giving, held stock and taking.

use bevy::prelude::*;
use shared::item::Quality;

/// What an item's name is written in when it has no quality.
pub const NO_QUALITY_NAME: Color = Color::srgb(0.9, 0.85, 0.75);

/// How far the bands on set items' edges have slid, in pixels.
#[derive(Resource, Default)]
pub struct BandShift(pub f32);

/// On what an item can be dropped on to give it to someone, for all of it to count.
#[derive(Component, Clone, Copy, Debug)]
pub struct TakesGifts(pub shared::id::Id);

/// The cell of the bag on the ground the pointer is on, if any: a click there isn't
/// on the map under it.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub struct HoveredBag(pub Option<(i64, i64)>);

/// Who `StockCell` and `TakesGifts` say for the ground under the player, which is
/// nobody's.
pub const GROUND: shared::id::Id = shared::id::Id::NOBODY;

/// Something held over what `TakesGifts`.
#[derive(Clone, Copy, Debug)]
pub struct HeldGift {
    pub to: shared::id::Id,
    pub item: shared::id::Id,
    pub rotation: shared::item::Rotation,
    /// Where its shape's top left corner is, in logical pixels.
    pub corner: Vec2,
}

/// What is held over what `TakesGifts` now.
#[derive(Resource, Default)]
pub struct HeldOverGifts(pub Option<HeldGift>);

/// It was let go there, and is on its way to them.
#[derive(Message, Clone, Copy, Debug)]
pub struct GaveItem(pub HeldGift);

/// On a cell of something someone has to trade, and its label: a press picks it up
/// by this cell.
#[derive(Component, Clone, Debug)]
pub struct StockCell {
    pub with: shared::id::Id,
    pub item: shared::item::Item,
    pub rotation: shared::item::Rotation,
    pub grip: (u8, u8),
}

/// What is held that is still someone else's, if anything: not drawn where it was.
#[derive(Resource, Default, PartialEq)]
pub struct HeldStock(pub Option<shared::id::Id>);

/// It was taken from who they are trading with, and is on its way to the pockets.
#[derive(Message, Clone, Copy, Debug)]
pub struct TookItem {
    pub with: shared::id::Id,
    pub item: shared::id::Id,
}

/// What each quality of item looks like: a name in it, an edge around it.
pub fn quality_colour(quality: Quality) -> Color {
    match quality {
        Quality::Poor => Color::srgb(0.62, 0.62, 0.62),
        Quality::Common => Color::srgb(1.0, 1.0, 1.0),
        Quality::Uncommon => Color::srgb(0.12, 0.85, 0.2),
        Quality::Rare => Color::srgb(0.2, 0.5, 1.0),
        Quality::Epic => Color::srgb(0.7, 0.3, 0.95),
        Quality::Legendary => Color::srgb(1.0, 0.55, 0.05),
        Quality::Artifact => Color::srgb(1.0, 0.84, 0.25),
    }
}
