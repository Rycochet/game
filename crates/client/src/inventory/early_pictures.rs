//! Pictures of everything carried, asked for as soon as it is known (mostly while
//! the game is still loading), so they are drawn before a panel shows them.

use bevy::prelude::*;
use shared::item::{Item, Placed};

use super::Carried;
use crate::item_art::{ItemPictures, look_of};
use crate::widgets::PICTURE_PIXELS_PER_CELL;

pub(super) fn ask_early(
    carried: Res<Carried>,
    mut pictures: ResMut<ItemPictures>,
    mut images: ResMut<Assets<Image>>,
) {
    if !carried.is_changed() {
        return;
    }
    let mut ask = |item: &Item, pixels: u32| {
        if let Some(look) = look_of(item) {
            pictures.picture(look, pixels, &mut images);
        }
    };
    let inventory = &carried.0;
    // Worn things show in a cell of the clothes panel; in a grid they take their shape.
    let mut grids: Vec<&[Placed]> = vec![&inventory.items];
    for item in &inventory.worn {
        ask(item, PICTURE_PIXELS_PER_CELL);
        ask(item, longest(item) * PICTURE_PIXELS_PER_CELL);
        grids.push(&item.contents);
    }
    while let Some(grid) = grids.pop() {
        for placed in grid {
            ask(
                &placed.item,
                longest(&placed.item) * PICTURE_PIXELS_PER_CELL,
            );
            grids.push(&placed.item.contents);
        }
    }
}

fn longest(item: &Item) -> u32 {
    let shape = item.shape();
    shape.width.max(shape.height) as u32
}
