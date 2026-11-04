//! Item Pictures' Legs page: rolled leg pieces by construction, chausses and padded
//! chausses, and a plate leg in each style.

use bevy::prelude::*;
use shared::armour::legs::{self, Cover, LegsForm, LegsKind};
use shared::armour::{Construction, Style};
use shared::item::Quality;
use shared::rng::{Rng, Seed};

use super::armour::{ArmourShown, first_plate, spawn_row, spawn_style_rows};
use super::{RING_COLUMNS, Shown};
use crate::item_art::{LegsLook, Look};

/// One leg picture.
#[derive(Clone, Copy)]
pub(super) enum LegsShown {
    /// The so-manyth leg piece of this kind rolled in this construction.
    Rolled(LegsKind, Construction, u64),
    /// The first whole plate leg rolled in this style, or the first piece.
    Style(Style),
}

impl LegsShown {
    pub(super) fn look(self, quality: Quality) -> Option<Look> {
        let form = match self {
            Self::Rolled(kind, construction, nth) => (0..20_000)
                .map(|seed| legs::roll(&mut Seed::new(seed).rng(), kind, None, quality))
                .filter(|form| form.construction == construction)
                .nth(nth as usize)?,
            Self::Style(style) => {
                let roll = |rng: &mut Rng| legs::roll(rng, LegsKind::Armour, Some(style), quality);
                let whole = (0..200)
                    .map(|seed| roll(&mut Seed::new(seed).rng()))
                    .find(|form| {
                        form.construction == Construction::Plate && form.cover == Cover::Whole
                    });
                match whole {
                    Some(form) => form,
                    None => first_plate(roll, |form: &LegsForm| form.construction)?,
                }
            }
        };
        let shape = form.shape();
        Some(Look::Legs {
            legs: LegsLook::new(form, (shape.width, shape.height)),
            quality: Some(quality),
        })
    }
}

/// Rolled leg pieces by construction, two rows of plate, chausses, and plate in each
/// style.
pub(super) fn spawn_legs_rows(grid: &mut ChildSpawnerCommands) {
    use Construction::{
        Lamellar, Laminar, Leather, Mail, MailAndPlate, Padded, Plate, PlatesInCloth, RingMail,
        Scale, Splint,
    };
    use LegsKind::{Armour, Chausses};
    let rows: [(&str, LegsKind, &[Construction]); 9] = [
        ("Plate", Armour, &[Plate]),
        ("", Armour, &[Plate]),
        ("Leather", Armour, &[Leather]),
        ("Mail, Ring", Armour, &[Mail, RingMail]),
        ("Chausses", Chausses, &[Mail]),
        ("Scale, Lamellar", Armour, &[Scale, Lamellar]),
        ("Splint, Laminar", Armour, &[Splint, Laminar]),
        ("Plated, Cloth", Armour, &[MailAndPlate, PlatesInCloth]),
        ("Padded", Armour, &[Padded]),
    ];
    for (row, (label, kind, constructions)) in rows.into_iter().enumerate() {
        let pictures = (0..RING_COLUMNS as u64).map(move |cell| {
            // A shared row's second half is its second construction.
            let half = (cell * constructions.len() as u64 / RING_COLUMNS as u64) as usize;
            // The second plate row carries on from the first.
            let nth = if row == 1 {
                cell + RING_COLUMNS as u64
            } else {
                cell % (RING_COLUMNS as u64 / constructions.len() as u64)
            };
            // The padded row's second half is padded chausses.
            let kind = if constructions == [Padded] && cell >= RING_COLUMNS as u64 / 2 {
                LegsKind::Padded
            } else {
                kind
            };
            Shown::Armour(ArmourShown::Legs(LegsShown::Rolled(
                kind,
                constructions[half],
                nth,
            )))
        });
        spawn_row(grid, label, pictures);
    }
    spawn_style_rows(grid, |style| ArmourShown::Legs(LegsShown::Style(style)));
}
