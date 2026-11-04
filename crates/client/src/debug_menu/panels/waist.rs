//! Item Pictures' Waist page: rolled waist pieces by shape, cuirassier tassets, and a
//! piece in each style.

use bevy::prelude::*;
use shared::armour::Style;
use shared::armour::waist::{self, WaistKind, WaistPiece};
use shared::item::Quality;
use shared::rng::Seed;

use super::armour::{ArmourShown, spawn_row, spawn_style_rows};
use super::{RING_COLUMNS, Shown};
use crate::item_art::{Look, WaistLook};

/// One waist picture.
#[derive(Clone, Copy)]
pub(super) enum WaistShown {
    /// The so-manyth waist piece of this kind rolled in this shape.
    Rolled(WaistKind, WaistPiece, u64),
    /// The first plate tassets rolled in this style, or the first piece.
    Style(Style),
}

impl WaistShown {
    pub(super) fn look(self, quality: Quality) -> Option<Look> {
        let roll =
            |seed: u64, kind, style| waist::roll(&mut Seed::new(seed).rng(), kind, style, quality);
        let form = match self {
            Self::Rolled(kind, piece, nth) => (0..20_000)
                .map(|seed| roll(seed, kind, None))
                .filter(|form| form.piece == piece)
                .nth(nth as usize)?,
            Self::Style(style) => {
                let rolls = || (0..200).map(|seed| roll(seed, WaistKind::Armour, Some(style)));
                rolls()
                    .find(|form| form.piece == WaistPiece::Tassets)
                    .or_else(|| rolls().next())?
            }
        };
        let shape = form.shape();
        Some(Look::Waist {
            waist: WaistLook::new(form, (shape.width, shape.height)),
            quality: Some(quality),
        })
    }
}

/// Rolled waist pieces by shape, two rows of tassets and two of skirts, and a piece in
/// each style.
pub(super) fn spawn_waist_rows(grid: &mut ChildSpawnerCommands) {
    use WaistKind::{Armour, Cuirassier};
    use WaistPiece::{Breeches, Fauld, Girdle, Panels, Pteruges, Skirt, Tassets};
    let rows: [(&str, WaistKind, &[WaistPiece]); 8] = [
        ("Tassets", Armour, &[Tassets]),
        ("", Armour, &[Tassets]),
        ("Cuirassier", Cuirassier, &[Tassets]),
        ("Fauld, Girdle", Armour, &[Fauld, Girdle]),
        ("Skirts", Armour, &[Skirt]),
        ("", Armour, &[Skirt]),
        ("Panels", Armour, &[Panels]),
        ("Pteruges, Mail", Armour, &[Pteruges, Breeches]),
    ];
    for (row, (label, kind, pieces)) in rows.into_iter().enumerate() {
        let pictures = (0..RING_COLUMNS as u64).map(move |cell| {
            // A shared row's second half is its second shape.
            let half = (cell * pieces.len() as u64 / RING_COLUMNS as u64) as usize;
            // Second rows carry on from the first.
            let nth = match row {
                1 | 5 => cell + RING_COLUMNS as u64,
                _ => cell % (RING_COLUMNS as u64 / pieces.len() as u64),
            };
            Shown::Armour(ArmourShown::Waist(WaistShown::Rolled(
                kind,
                pieces[half],
                nth,
            )))
        });
        spawn_row(grid, label, pictures);
    }
    spawn_style_rows(grid, |style| ArmourShown::Waist(WaistShown::Style(style)));
}
