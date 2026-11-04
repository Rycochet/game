//! Item Pictures' armour pages: rolled hand, shoulder, torso and head pieces, and a
//! picture of each in every style. Arm pieces are in `arms.rs`, leg pieces in `legs.rs`.

use bevy::prelude::*;
use shared::armour::hands::{self, HandsPiece};
use shared::armour::head::{self, HeadKind};
use shared::armour::shoulders::{self, ShouldersKind};
use shared::armour::torso::{self, TorsoKind};
use shared::armour::{Construction, Style};
use shared::item::{ItemKind, Quality};
use shared::rng::{Rng, Seed};

use super::{PICTURE_ROW_LABEL, RING_COLUMNS, Shown, picture_label, spawn_picture};
use crate::item_art::{HandsLook, HeadLook, Look, ShouldersLook, TorsoLook};

/// One armour picture.
#[derive(Clone, Copy)]
pub(super) enum ArmourShown {
    /// Hand pieces of this kind as the server would roll them from this number, with
    /// this many digits on each hand.
    Hands(HandsPiece, u64, u8),
    /// The first plate gauntlets rolled in this style.
    HandsStyle(Style),
    /// Shoulder pieces of this kind as the server would roll them from this number.
    Shoulders(ShouldersKind, u64),
    /// The first plate pauldrons rolled in this style.
    ShouldersStyle(Style),
    /// The so-manyth torso piece rolled in this construction, or an arming vest.
    Torso(Option<Construction>, u64),
    /// The first plate torso piece rolled in this style.
    TorsoStyle(Style),
    /// A head piece of this kind as the server would roll it from this number.
    Head(HeadKind, u64),
    /// The first helmet rolled in this style.
    HeadStyle(Style),
    /// An arm piece (see `arms.rs`).
    Arms(super::arms::ArmsShown),
    /// A leg piece (see `legs.rs`).
    Legs(super::legs::LegsShown),
}

/// The first of fifty rolls in plate, or the first at all where the style has none.
pub(super) fn first_plate<T>(
    roll: impl Fn(&mut Rng) -> T,
    construction: impl Fn(&T) -> Construction,
) -> Option<T> {
    (0..50)
        .map(|seed| roll(&mut Seed::new(seed).rng()))
        .reduce(|first, form| {
            let plate = Construction::Plate;
            if construction(&first) != plate && construction(&form) == plate {
                form
            } else {
                first
            }
        })
}

impl ArmourShown {
    pub(in crate::debug_menu) fn look(self, quality: Quality) -> Option<Look> {
        let rolled = quality;
        let quality = Some(quality);
        Some(match self {
            Self::Arms(arms) => arms.look(rolled)?,
            Self::Legs(legs) => legs.look(rolled)?,
            Self::Hands(piece, seed, digits) => hands_look(
                hands::roll(&mut Seed::new(seed).rng(), piece, None),
                digits,
                quality,
            ),
            Self::HandsStyle(style) => {
                let roll = |rng: &mut Rng| hands::roll(rng, HandsPiece::Gauntlets, Some(style));
                hands_look(first_plate(roll, |form| form.construction)?, 5, quality)
            }
            Self::Shoulders(kind, seed) => shoulders_look(
                shoulders::roll(&mut Seed::new(seed).rng(), Some(kind), None),
                quality,
            ),
            Self::ShouldersStyle(style) => {
                let pauldrons = Some(ShouldersKind::Pauldrons);
                let roll = |rng: &mut Rng| shoulders::roll(rng, pauldrons, Some(style));
                shoulders_look(first_plate(roll, |form| form.construction)?, quality)
            }
            Self::Torso(construction, nth) => {
                let kind = match construction {
                    Some(_) => TorsoKind::Armour,
                    None => TorsoKind::ArmingVest,
                };
                let form = (0..2000)
                    .map(|seed| torso::roll(&mut Seed::new(seed).rng(), kind, None, rolled))
                    .filter(|form| construction.is_none_or(|c| form.construction == c))
                    .nth(nth as usize)?;
                torso_look(form, quality)
            }
            Self::TorsoStyle(style) => {
                let roll = |rng: &mut Rng| torso::roll(rng, TorsoKind::Armour, Some(style), rolled);
                torso_look(first_plate(roll, |form| form.construction)?, quality)
            }
            Self::Head(kind, seed) => head_look(
                head::roll(&mut Seed::new(seed).rng(), kind, None, rolled),
                quality,
            ),
            Self::HeadStyle(style) => {
                let mut rng = Seed::new(style as u64).rng();
                head_look(
                    head::roll(&mut rng, HeadKind::Helmet, Some(style), rolled),
                    quality,
                )
            }
        })
    }
}

/// Shoulder pieces in the cells their form takes up.
fn shoulders_look(form: shoulders::ShouldersForm, quality: Option<Quality>) -> Look {
    let shape = form.shape();
    Look::Shoulders {
        shoulders: ShouldersLook::new(form, (shape.width, shape.height)),
        quality,
    }
}

/// Hand pieces as a pair of gauntlets takes them up.
fn hands_look(form: hands::HandsForm, digits: u8, quality: Option<Quality>) -> Look {
    let shape = ItemKind::Gauntlets.def().shape;
    Look::Hands {
        hands: HandsLook::new(form, digits, (shape.width, shape.height)),
        quality,
    }
}

/// A torso piece in the cells its form takes up.
fn torso_look(form: torso::TorsoForm, quality: Option<Quality>) -> Look {
    let shape = form.shape();
    Look::Torso {
        torso: TorsoLook::new(form, (shape.width, shape.height)),
        quality,
    }
}

/// A head piece in the cells its form takes up.
fn head_look(form: head::HeadForm, quality: Option<Quality>) -> Look {
    let shape = form.shape();
    Look::Head {
        head: HeadLook::new(form, (shape.width, shape.height)),
        quality,
    }
}

/// A labelled row of pictures.
pub(super) fn spawn_row(
    grid: &mut ChildSpawnerCommands,
    label: &str,
    pictures: impl Iterator<Item = Shown>,
) {
    grid.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|cells| {
        cells.spawn(picture_label(label, PICTURE_ROW_LABEL));
        for shown in pictures {
            spawn_picture(cells, shown);
        }
    });
}

/// Rolled gauntlets (the second row with a kobold's hands), demi-gauntlets and padded
/// gloves, and plate gauntlets in each style.
pub(super) fn spawn_hands_rows(grid: &mut ChildSpawnerCommands) {
    let rows = [
        ("Gauntlets", HandsPiece::Gauntlets, 5),
        ("", HandsPiece::Gauntlets, 4),
        ("Demi, Padded", HandsPiece::Demi, 5),
    ];
    for (row, (label, piece, digits)) in rows.into_iter().enumerate() {
        let pictures = (0..RING_COLUMNS as u64).map(move |cell| {
            let seed = row as u64 * 100 + cell;
            // Half the last row padded gloves.
            let piece = if piece == HandsPiece::Demi && cell >= RING_COLUMNS as u64 / 2 {
                HandsPiece::Padded
            } else {
                piece
            };
            Shown::Armour(ArmourShown::Hands(piece, seed, digits))
        });
        spawn_row(grid, label, pictures);
    }
    spawn_style_rows(grid, ArmourShown::HandsStyle);
}

/// Rolled shoulder pieces, a row for the commonest kinds and shared rows for the rest,
/// and plate pauldrons in each style.
pub(super) fn spawn_shoulders_rows(grid: &mut ChildSpawnerCommands) {
    use ShouldersKind::{Ailettes, Cape, Discs, Guards, Pauldrons, Spaulders, Wing};
    let rows: [(&str, &[ShouldersKind]); 5] = [
        ("Pauldrons", &[Pauldrons]),
        ("Spaulders", &[Spaulders]),
        ("Guards", &[Guards]),
        ("Cape, Discs", &[Cape, Discs]),
        ("Ailettes, Wing", &[Ailettes, Wing]),
    ];
    for (row, (label, kinds)) in rows.into_iter().enumerate() {
        let pictures = (0..RING_COLUMNS).map(move |cell| {
            // A shared row's last kind gets only its last picture.
            let kind = if cell + 1 == RING_COLUMNS {
                kinds[kinds.len() - 1]
            } else {
                kinds[0]
            };
            let seed = row as u64 * 100 + cell as u64;
            Shown::Armour(ArmourShown::Shoulders(kind, seed))
        });
        spawn_row(grid, label, pictures);
    }
    spawn_style_rows(grid, ArmourShown::ShouldersStyle);
}

/// Rolled torso pieces by construction, arming vests, and plate in each style.
pub(super) fn spawn_torso_rows(grid: &mut ChildSpawnerCommands) {
    use Construction::{Leather, Mail, Padded, Plate, RingMail};
    let rows: [(&str, &[Option<Construction>]); 5] = [
        ("Plate", &[Some(Plate)]),
        ("", &[Some(Plate)]),
        ("Leather", &[Some(Leather)]),
        ("Mail, Ring", &[Some(Mail), Some(RingMail)]),
        ("Padded, Vest", &[Some(Padded), None]),
    ];
    for (row, (label, constructions)) in rows.into_iter().enumerate() {
        let pictures = (0..RING_COLUMNS as u64).map(move |cell| {
            // A shared row's second half is its second kind.
            let half = (cell * constructions.len() as u64 / RING_COLUMNS as u64) as usize;
            // The second plate row carries on from the first.
            let nth = if row == 1 {
                cell + RING_COLUMNS as u64
            } else {
                cell
            };
            Shown::Armour(ArmourShown::Torso(constructions[half], nth))
        });
        spawn_row(grid, label, pictures);
    }
    spawn_style_rows(grid, ArmourShown::TorsoStyle);
}

/// Rolled helmets over three rows, then bevored helms, coifs and caps, and a helmet in
/// each style.
pub(super) fn spawn_head_rows(grid: &mut ChildSpawnerCommands) {
    use HeadKind::{BevoredHelm, Cap, Coif, Helmet};
    let rows: [(&str, &[HeadKind]); 5] = [
        ("Helmets", &[Helmet]),
        ("", &[Helmet]),
        ("", &[Helmet]),
        ("Bevored, Coif", &[BevoredHelm, Coif]),
        ("Coif, Cap", &[Coif, Cap]),
    ];
    for (row, (label, kinds)) in rows.into_iter().enumerate() {
        let pictures = (0..RING_COLUMNS as u64).map(move |cell| {
            // A shared row's second half is its second kind.
            let half = (cell * kinds.len() as u64 / RING_COLUMNS as u64) as usize;
            Shown::Armour(ArmourShown::Head(kinds[half], row as u64 * 100 + cell))
        });
        spawn_row(grid, label, pictures);
    }
    spawn_style_rows(grid, ArmourShown::HeadStyle);
}

/// A picture in each style, as many to a row as rings have.
pub(super) fn spawn_style_rows(grid: &mut ChildSpawnerCommands, shown: fn(Style) -> ArmourShown) {
    for (row, styles) in Style::ALL.chunks(RING_COLUMNS as usize).enumerate() {
        let label = if row == 0 { "Styles" } else { "" };
        let pictures = styles.iter().map(|&style| Shown::Armour(shown(style)));
        spawn_row(grid, label, pictures);
    }
}
