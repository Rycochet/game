//! Item Pictures' Sets page: rolled sets of each kind, and a full set in each style.

use bevy::prelude::*;
use shared::armour::Style;
use shared::armour::set::{self, SetKind};
use shared::item::{ItemKind, Quality};
use shared::rng::Seed;

use super::armour::{ArmourShown, spawn_row, spawn_style_rows};
use super::{RING_COLUMNS, Shown};
use crate::item_art::{Look, SetLook};

/// One set's picture.
#[derive(Clone, Copy)]
pub(super) enum SetShown {
    /// A set of this kind as the server would roll it from this number.
    Rolled(SetKind, u64),
    /// A full set in this style.
    Style(Style),
}

impl SetShown {
    pub(super) fn look(self, quality: Quality) -> Option<Look> {
        let (kind, seed, style) = match self {
            Self::Rolled(kind, seed) => (kind, seed, None),
            Self::Style(style) => (SetKind::Full, 0, Some(style)),
        };
        let form = set::roll(&mut Seed::new(seed).rng(), kind, style, quality);
        let shape = item_kind(kind).def().shape;
        Some(Look::Set {
            set: SetLook::new(form, 5, (shape.width, shape.height)),
            quality: Some(quality),
        })
    }
}

/// The item kind a set kind is made as.
fn item_kind(kind: SetKind) -> ItemKind {
    match kind {
        SetKind::Coat => ItemKind::ArmourCoat,
        SetKind::Corslet => ItemKind::Corslet,
        SetKind::Half => ItemKind::HalfArmour,
        SetKind::Full => ItemKind::FullArmour,
    }
}

/// A row of each kind of set, and a full set in each style.
pub(super) fn spawn_set_rows(grid: &mut ChildSpawnerCommands) {
    let rows = [
        ("Coats", SetKind::Coat),
        ("Corslets", SetKind::Corslet),
        ("Half", SetKind::Half),
        ("Full", SetKind::Full),
    ];
    for (label, kind) in rows {
        let pictures = (0..RING_COLUMNS as u64)
            .map(move |seed| Shown::Armour(ArmourShown::Set(SetShown::Rolled(kind, seed))));
        spawn_row(grid, label, pictures);
    }
    spawn_style_rows(grid, |style| ArmourShown::Set(SetShown::Style(style)));
}
