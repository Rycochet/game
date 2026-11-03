//! Item Pictures' Arms page: rolled arm pieces by construction, and a plate arm in
//! each style.

use bevy::prelude::*;
use shared::armour::arms::{self, ArmsForm, Reach};
use shared::armour::{Construction, Style};
use shared::item::Quality;
use shared::rng::{Rng, Seed};

use super::armour::{ArmourShown, first_plate, spawn_row, spawn_style_rows};
use super::{RING_COLUMNS, Shown};
use crate::item_art::{ArmsLook, Look};

/// One arm picture.
#[derive(Clone, Copy)]
pub(super) enum ArmsShown {
    /// The so-manyth arm piece rolled in this construction.
    Rolled(Construction, u64),
    /// The first whole plate arm rolled in this style, or the first plate piece.
    Style(Style),
}

impl ArmsShown {
    pub(super) fn look(self, quality: Quality) -> Option<Look> {
        let form = match self {
            Self::Rolled(construction, nth) => (0..20_000)
                .map(|seed| arms::roll(&mut Seed::new(seed).rng(), None, quality))
                .filter(|form| form.construction == construction)
                .nth(nth as usize)?,
            Self::Style(style) => {
                let roll = |rng: &mut Rng| arms::roll(rng, Some(style), quality);
                let whole = (0..200)
                    .map(|seed| roll(&mut Seed::new(seed).rng()))
                    .find(|form| {
                        form.construction == Construction::Plate && form.reach == Reach::Whole
                    });
                match whole {
                    Some(form) => form,
                    None => first_plate(roll, |form: &ArmsForm| form.construction)?,
                }
            }
        };
        let shape = form.shape();
        Some(Look::Arms {
            arms: ArmsLook::new(form, (shape.width, shape.height)),
            quality: Some(quality),
        })
    }
}

/// Rolled arm pieces by construction, two rows of plate, and plate in each style.
pub(super) fn spawn_arms_rows(grid: &mut ChildSpawnerCommands) {
    use Construction::{
        Lamellar, Laminar, Leather, Mail, MailAndPlate, Padded, Plate, PlatesInCloth, RingMail,
        Scale, Splint,
    };
    let rows: [(&str, &[Construction]); 8] = [
        ("Plate", &[Plate]),
        ("", &[Plate]),
        ("Leather", &[Leather]),
        ("Mail, Ring", &[Mail, RingMail]),
        ("Scale, Lamellar", &[Scale, Lamellar]),
        ("Splint, Laminar", &[Splint, Laminar]),
        ("Kote", &[MailAndPlate]),
        ("Cloth, Padded", &[PlatesInCloth, Padded]),
    ];
    for (row, (label, constructions)) in rows.into_iter().enumerate() {
        let pictures = (0..RING_COLUMNS as u64).map(move |cell| {
            // A shared row's second half is its second construction.
            let half = (cell * constructions.len() as u64 / RING_COLUMNS as u64) as usize;
            // The second plate row carries on from the first.
            let nth = if row == 1 {
                cell + RING_COLUMNS as u64
            } else {
                cell % (RING_COLUMNS as u64 / constructions.len() as u64)
            };
            Shown::Armour(ArmourShown::Arms(ArmsShown::Rolled(
                constructions[half],
                nth,
            )))
        });
        spawn_row(grid, label, pictures);
    }
    spawn_style_rows(grid, |style| ArmourShown::Arms(ArmsShown::Style(style)));
}
