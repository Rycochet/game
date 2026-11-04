//! Item Pictures' Neck page: rolled neck pieces by kind, and a gorget in each style.

use bevy::prelude::*;
use shared::armour::Style;
use shared::armour::neck::{self, Front, NeckForm, NeckKind, Plates};
use shared::item::Quality;
use shared::rng::Seed;

use super::armour::{ArmourShown, spawn_row, spawn_style_rows};
use super::{RING_COLUMNS, Shown};
use crate::item_art::{Look, NeckLook};

/// A row of the page: which rolled pieces it shows.
#[derive(Clone, Copy)]
pub(super) enum NeckRow {
    Collars,
    Gorgets,
    Colletins,
    Bibs,
    Bevors,
    Aventails,
    Closed,
    Mantles,
}

impl NeckRow {
    const ALL: [(Self, &'static str); 8] = [
        (Self::Collars, "Collars"),
        (Self::Gorgets, "Gorgets"),
        (Self::Colletins, "Colletins"),
        (Self::Bibs, "Bibs, Collars"),
        (Self::Bevors, "Bevors"),
        (Self::Aventails, "Aventails"),
        (Self::Closed, "Closed"),
        (Self::Mantles, "Mantles"),
    ];

    fn kind(self) -> NeckKind {
        match self {
            Self::Collars => NeckKind::Collar,
            Self::Gorgets | Self::Colletins | Self::Bibs => NeckKind::Gorget,
            Self::Bevors => NeckKind::Bevor,
            Self::Aventails | Self::Closed => NeckKind::Aventail,
            Self::Mantles => NeckKind::Mantle,
        }
    }

    fn shows(self, form: &NeckForm) -> bool {
        match self {
            Self::Gorgets => form.plates == Plates::Plates,
            Self::Colletins => form.plates == Plates::Shoulders,
            Self::Bibs => matches!(form.plates, Plates::Bib | Plates::Collar),
            Self::Aventails => matches!(form.front, Front::Open | Front::Chin),
            Self::Closed => matches!(form.front, Front::Closed | Front::Ventail),
            _ => true,
        }
    }
}

/// One neck picture.
#[derive(Clone, Copy)]
pub(super) enum NeckShown {
    /// The so-manyth piece rolled that this row shows.
    Rolled(NeckRow, u64),
    /// The first gorget with plates rolled in this style, or the first gorget.
    Style(Style),
}

impl NeckShown {
    pub(super) fn look(self, quality: Quality) -> Option<Look> {
        let roll =
            |seed: u64, kind, style| neck::roll(&mut Seed::new(seed).rng(), kind, style, quality);
        let form = match self {
            Self::Rolled(row, nth) => (0..20_000)
                .map(|seed| roll(seed, row.kind(), None))
                .filter(|form| row.shows(form))
                .nth(nth as usize)?,
            Self::Style(style) => {
                let rolls = || (0..200).map(|seed| roll(seed, NeckKind::Gorget, Some(style)));
                rolls()
                    .find(|form| form.plates == Plates::Plates)
                    .or_else(|| rolls().next())?
            }
        };
        let shape = form.shape();
        Some(Look::Neck {
            neck: NeckLook::new(form, (shape.width, shape.height)),
            quality: Some(quality),
        })
    }
}

/// Rolled neck pieces in a row for each kind, and a gorget in each style.
pub(super) fn spawn_neck_rows(grid: &mut ChildSpawnerCommands) {
    for (row, label) in NeckRow::ALL {
        let pictures = (0..RING_COLUMNS as u64)
            .map(move |nth| Shown::Armour(ArmourShown::Neck(NeckShown::Rolled(row, nth))));
        spawn_row(grid, label, pictures);
    }
    spawn_style_rows(grid, |style| ArmourShown::Neck(NeckShown::Style(style)));
}
