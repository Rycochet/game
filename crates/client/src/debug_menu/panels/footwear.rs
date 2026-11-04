//! Item Pictures' Footwear page: rolled shoes, boots by height, sandals, clogs and
//! slippers.

use bevy::prelude::*;
use shared::footwear::{self, FootwearForm, FootwearPiece, Height};
use shared::item::Quality;
use shared::rng::Seed;

use super::armour::spawn_row;
use super::{RING_COLUMNS, Shown};
use crate::item_art::{FootwearLook, Look};

/// What a picture of the page shows: rolled pairs of a kind, a height of boot, or the
/// cavalier boot.
#[derive(Clone, Copy)]
pub(super) enum FootwearRow {
    Shoes,
    Ankle,
    Calf,
    Knee,
    Cavalier,
    Sandals,
    Clogs,
    Slippers,
}

impl FootwearRow {
    fn piece(self) -> FootwearPiece {
        match self {
            Self::Shoes => FootwearPiece::Shoes,
            Self::Sandals => FootwearPiece::Sandals,
            Self::Clogs => FootwearPiece::Clogs,
            Self::Slippers => FootwearPiece::Slippers,
            _ => FootwearPiece::Boots,
        }
    }

    fn shows(self, form: &FootwearForm) -> bool {
        match self {
            Self::Ankle => form.height == Height::Ankle,
            Self::Calf => form.height == Height::Calf,
            Self::Knee => form.height == Height::Knee,
            Self::Cavalier => form.height == Height::Thigh,
            _ => true,
        }
    }
}

/// The so-manyth pair rolled that a row shows.
#[derive(Clone, Copy)]
pub(super) struct FootwearShown(FootwearRow, u64);

impl FootwearShown {
    pub(super) fn look(self, quality: Quality) -> Option<Look> {
        let FootwearShown(row, nth) = self;
        let form = (0..20_000u64)
            .map(|seed| footwear::roll(&mut Seed::new(seed).rng(), row.piece(), quality))
            .filter(|form| row.shows(form))
            .nth(nth as usize)?;
        let shape = form.shape();
        Some(Look::Footwear {
            footwear: FootwearLook::new(form, (shape.width, shape.height)),
            quality: Some(quality),
        })
    }
}

/// Rows of rolled footwear: two of shoes, one for each height of boot (the knee row
/// ending with a cavalier boot), sandals, clogs and slippers.
pub(super) fn spawn_footwear_rows(grid: &mut ChildSpawnerCommands) {
    let rows = [
        ("Shoes", FootwearRow::Shoes, 0),
        ("", FootwearRow::Shoes, 1),
        ("Ankle Boots", FootwearRow::Ankle, 0),
        ("Calf Boots", FootwearRow::Calf, 0),
        ("Knee Boots", FootwearRow::Knee, 0),
        ("Sandals", FootwearRow::Sandals, 0),
        ("Clogs", FootwearRow::Clogs, 0),
        ("Slippers", FootwearRow::Slippers, 0),
    ];
    let columns = RING_COLUMNS as u64;
    for (label, row, page) in rows {
        let pictures = (0..columns).map(move |cell| {
            let shown = match row {
                FootwearRow::Knee if cell + 1 == columns => FootwearShown(FootwearRow::Cavalier, 0),
                _ => FootwearShown(row, page * columns + cell),
            };
            Shown::Footwear(shown)
        });
        spawn_row(grid, label, pictures);
    }
}
