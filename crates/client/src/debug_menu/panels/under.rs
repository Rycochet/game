//! Item Pictures' Underclothes page: rolled pieces of each kind, two rows of shirts.

use bevy::prelude::*;
use shared::item::Quality;
use shared::rng::Seed;
use shared::underclothes::{self, UnderPiece};

use super::armour::spawn_row;
use super::{RING_COLUMNS, Shown};
use crate::item_art::{Look, UnderLook};

/// The so-manyth piece of a kind rolled.
#[derive(Clone, Copy)]
pub(super) struct UnderShown(UnderPiece, u64);

impl UnderShown {
    pub(super) fn look(self, quality: Quality) -> Option<Look> {
        let UnderShown(piece, nth) = self;
        let form = underclothes::roll(&mut Seed::new(nth).rng(), piece, quality);
        let shape = form.shape();
        Some(Look::Under {
            under: UnderLook::new(form, (shape.width, shape.height)),
            quality: Some(quality),
        })
    }
}

/// A row for each kind, two for shirts.
pub(super) fn spawn_under_rows(grid: &mut ChildSpawnerCommands) {
    let rows = [
        ("Shirts", UnderPiece::Shirt, 0),
        ("", UnderPiece::Shirt, 1),
        ("Braies", UnderPiece::Braies, 0),
        ("Breast Bands", UnderPiece::BreastBand, 0),
        ("Loincloths", UnderPiece::Loincloth, 0),
        ("Hose", UnderPiece::Hose, 0),
        ("Socks", UnderPiece::Socks, 0),
        ("Petticoats", UnderPiece::Petticoat, 0),
        ("Coifs", UnderPiece::Coif, 0),
        ("Wimples", UnderPiece::Wimple, 0),
    ];
    let columns = RING_COLUMNS as u64;
    for (label, piece, page) in rows {
        let pictures =
            (0..columns).map(move |cell| Shown::Under(UnderShown(piece, page * columns + cell)));
        spawn_row(grid, label, pictures);
    }
}
