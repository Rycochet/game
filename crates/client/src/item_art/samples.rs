//! Looks of items as the server would make them, the same every time: for timing
//! drawing, and for drawing a few while the game loads, so that the graphics card's
//! programs are ready before the first panel full of pictures.

use bevy::prelude::*;
use shared::creature::Size;
use shared::id::Id;
use shared::item::{Item, ItemKind, Quality};
use shared::item_form::give_form;
use std::collections::HashSet;
use std::mem::discriminant;

use super::{ItemPictures, Look, look_of};

/// Where the looks drawn while loading start, well away from those timed.
const WARM_UP_FROM: u128 = 1 << 40;
/// Pixels per cell of the longest side, as the inventory asks for them.
pub(super) const PIXELS_PER_CELL: u32 = 64;

/// The first `count` different looks of items made from ids `first`, `first + 1`,
/// ..., through every kind and quality in turn.
pub(super) fn looks(first: u128, count: usize) -> Vec<Look> {
    let mut seen = HashSet::new();
    let mut looks = Vec::new();
    for n in first..first + 1_000_000 {
        if looks.len() >= count {
            break;
        }
        let kind = ItemKind::ALL[n as usize % ItemKind::ALL.len()];
        let mut item = Item::new(Id::for_test(n), kind);
        let quality = (n as usize / ItemKind::ALL.len()) % Quality::ALL.len();
        item.quality = Some(Quality::ALL[quality]);
        let mut gems = n << 32;
        give_form(&mut item, Some(Size::Medium), &mut || {
            gems += 1;
            Id::for_test(gems)
        });
        if let Some(look) = look_of(&item)
            && seen.insert(look.clone())
        {
            looks.push(look);
        }
    }
    looks
}

/// Asks for one picture of each kind of look (gem, ring, weapon, each place armour
/// goes, ...), so their programs are compiled while the game is loading.
pub(super) fn warm_up(mut pictures: ResMut<ItemPictures>, mut images: ResMut<Assets<Image>>) {
    let mut kinds = HashSet::new();
    for look in looks(WARM_UP_FROM, 300) {
        if kinds.insert(discriminant(&look)) {
            let pixels = look.longest() as u32 * PIXELS_PER_CELL;
            pictures.picture(look, pixels, &mut images);
        }
    }
}
