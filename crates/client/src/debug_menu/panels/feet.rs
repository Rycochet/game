//! Item Pictures' Feet page: rolled foot pieces by toe and kind, and a pair in each
//! style.

use bevy::prelude::*;
use shared::armour::feet::{self, FeetForm, FeetPiece, Toe};
use shared::armour::{Construction, Style};
use shared::item::Quality;
use shared::rng::Seed;

use super::armour::{ArmourShown, spawn_row, spawn_style_rows};
use super::{RING_COLUMNS, Shown};
use crate::item_art::{FeetLook, Look};

/// A row of the page: which rolled pieces it shows.
#[derive(Clone, Copy)]
pub(super) enum FeetRow {
    Pointed,
    Broad,
    RoundAndClaw,
    ScaleAndSplint,
    Boots,
    MailShoes,
    Guards,
}

impl FeetRow {
    const ALL: [(Self, &'static str); 7] = [
        (Self::Pointed, "Pointed"),
        (Self::Broad, "Broad"),
        (Self::RoundAndClaw, "Round, Claw"),
        (Self::ScaleAndSplint, "Scale, Splint"),
        (Self::Boots, "Boots"),
        (Self::MailShoes, "Mail"),
        (Self::Guards, "Guards"),
    ];

    fn shows(self, form: &FeetForm) -> bool {
        let sabatons = form.piece == FeetPiece::Sabatons && !form.high;
        let plain = matches!(
            form.construction,
            Construction::Plate | Construction::Leather
        );
        match self {
            Self::Pointed => sabatons && plain && form.toe == Toe::Pointed,
            Self::Broad => sabatons && form.toe.broad(),
            Self::RoundAndClaw => sabatons && plain && matches!(form.toe, Toe::Round | Toe::Claw),
            Self::ScaleAndSplint => sabatons && !plain,
            Self::Boots => form.high,
            Self::MailShoes => form.piece == FeetPiece::MailShoes,
            Self::Guards => form.piece == FeetPiece::FootGuards,
        }
    }
}

/// One foot picture.
#[derive(Clone, Copy)]
pub(super) enum FeetShown {
    /// The so-manyth pair rolled that this row shows.
    Rolled(FeetRow, u64),
    /// The first plate sabatons rolled in this style, or the first pair.
    Style(Style),
}

impl FeetShown {
    pub(super) fn look(self, quality: Quality) -> Option<Look> {
        let roll = |seed: u64, style| feet::roll(&mut Seed::new(seed).rng(), style, quality);
        let form = match self {
            Self::Rolled(row, nth) => (0..20_000)
                .map(|seed| roll(seed, None))
                .filter(|form| row.shows(form))
                .nth(nth as usize)?,
            Self::Style(style) => {
                let rolls = || (0..200).map(|seed| roll(seed, Some(style)));
                rolls()
                    .find(|form| form.construction == Construction::Plate)
                    .or_else(|| rolls().next())?
            }
        };
        let shape = form.shape();
        Some(Look::Feet {
            feet: FeetLook::new(form, (shape.width, shape.height)),
            quality: Some(quality),
        })
    }
}

/// Rolled foot pieces in a row for each toe and kind, and a pair in each style.
pub(super) fn spawn_feet_rows(grid: &mut ChildSpawnerCommands) {
    for (row, label) in FeetRow::ALL {
        let pictures = (0..RING_COLUMNS as u64)
            .map(move |nth| Shown::Armour(ArmourShown::Feet(FeetShown::Rolled(row, nth))));
        spawn_row(grid, label, pictures);
    }
    spawn_style_rows(grid, |style| ArmourShown::Feet(FeetShown::Style(style)));
}
