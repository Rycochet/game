//! Item Pictures' Bodies page, below the bodies: people in each hair style and beard,
//! and long hair and a long beard under head and neck armour.

use std::collections::BTreeMap;

use bevy::prelude::*;
use shared::creature::{CreatureKind, Size};
use shared::id::Id;
use shared::item::{Item, ItemKind, Quality};
use shared::item_form::give_form;
use shared::sheet::{Appearance, Trait};
use shared::wear::WornLook;

use super::Shown;
use super::armour::spawn_row;
use crate::item_art::{Look, character_look};

/// One character's picture: a kind with its hair style and beard chosen by place in
/// its lists, wearing up to two things rolled from these numbers.
#[derive(Clone, Copy)]
pub(super) struct HairShown {
    kind: CreatureKind,
    style: u8,
    beard: u8,
    worn: [Option<(ItemKind, u128)>; 2],
}

impl HairShown {
    pub(super) fn look(self, quality: Quality) -> Option<Look> {
        let appearance = Appearance(BTreeMap::from([
            (Trait::HairStyle, self.style),
            (Trait::Beard, self.beard),
        ]));
        let worn: Vec<WornLook> = self
            .worn
            .into_iter()
            .flatten()
            .filter_map(|(kind, seed)| {
                let mut item = Item::new(Id::for_test(seed), kind);
                item.quality = Some(quality);
                give_form(&mut item, Some(Size::Medium), &mut || Id::for_test(0));
                Some(WornLook {
                    form: item.form?,
                    quality: item.quality,
                    fingers: None,
                })
            })
            .collect();
        character_look(self.kind, &appearance, &worn)
    }
}

fn person(kind: CreatureKind, style: u8, beard: u8) -> Shown {
    Shown::Hair(HairShown {
        kind,
        style,
        beard,
        worn: [None, None],
    })
}

fn wearing(first: (ItemKind, u128), second: Option<(ItemKind, u128)>) -> Shown {
    // A human with long hair and a long beard.
    Shown::Hair(HairShown {
        kind: CreatureKind::Human,
        style: 1,
        beard: 3,
        worn: [Some(first), second],
    })
}

/// A human in each style (with each beard along the way), each other people's own
/// styles and beards, then under helmets, a coif, an aventail and a bevor.
pub(super) fn spawn_hair_rows(grid: &mut ChildSpawnerCommands) {
    use CreatureKind::{Dwarf, Elf, Halfling, Human, Orc};
    spawn_row(grid, "Hair", (0..6).map(|at| person(Human, at, at % 4)));
    let peoples = (0..5)
        .map(|at| person(Dwarf, at, at))
        .chain((0..4).map(|at| person(Elf, at, 0)))
        .chain((0..4).map(|at| person(Halfling, at, 0)))
        .chain((0..5).map(|at| person(Orc, at, at % 3)));
    spawn_row(grid, "Peoples", peoples);
    let helmets = (1..=6).map(|seed| wearing((ItemKind::Helmet, seed), None));
    let others = [
        wearing((ItemKind::Coif, 1), None),
        wearing((ItemKind::Helmet, 2), Some((ItemKind::Aventail, 1))),
        wearing((ItemKind::Bevor, 1), None),
        wearing((ItemKind::BevoredHelm, 1), None),
    ];
    spawn_row(grid, "Covered", helmets.chain(others));
}
