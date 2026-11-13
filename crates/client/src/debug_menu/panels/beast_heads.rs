//! Item Pictures' Bodies page, at the bottom: each people with an animal's head
//! (`shared::body::Biped::head`) bare and under what it can wear on the head (open
//! helmets, as closed ones don't go over a snout: `shared::wear`), a coif and an
//! aventail, to see that they still fit it.

use bevy::prelude::*;
use shared::creature::{CreatureKind, Size};
use shared::id::Id;
use shared::item::{Item, ItemKind};
use shared::item_form::{ItemForm, give_form};

use super::armour::spawn_row;
use super::hair::dressed;

/// How many rows of rolled helmets each kind has.
const HELMET_ROWS: usize = 4;
/// Pictures in a row.
const COLUMNS: usize = 8;

/// The seeds of the first helmets rolled that leave the face open.
fn open_helmets(count: usize) -> Vec<u128> {
    (1..)
        .filter(|&seed| {
            let mut item = Item::new(Id::for_test(seed), ItemKind::Helmet);
            give_form(&mut item, Some(Size::Medium), &mut || Id::for_test(0));
            matches!(item.form, Some(ItemForm::Head(head)) if !head.face.closed())
        })
        .take(count)
        .collect()
}

/// A row or more for each people with an animal's head.
pub(super) fn spawn_beast_head_rows(grid: &mut ChildSpawnerCommands) {
    use CreatureKind::{Kobold, Lizardfolk, Minotaur};
    let seeds = open_helmets(COLUMNS * HELMET_ROWS - 3);
    for kind in [Kobold, Lizardfolk, Minotaur] {
        let helmet = |seed: u128| dressed(kind, (1, 0), [Some((ItemKind::Helmet, seed)), None]);
        let first = [
            dressed(kind, (0, 0), [None, None]),
            dressed(kind, (1, 0), [Some((ItemKind::Coif, 1)), None]),
            dressed(
                kind,
                (1, 0),
                [
                    Some((ItemKind::Helmet, seeds[0])),
                    Some((ItemKind::Aventail, 1)),
                ],
            ),
        ];
        let mut pictures: Vec<_> = first
            .into_iter()
            .chain(seeds.iter().map(|&s| helmet(s)))
            .collect();
        let mut label = kind.def().name;
        while !pictures.is_empty() {
            let rest = pictures.split_off(COLUMNS.min(pictures.len()));
            spawn_row(grid, label, pictures.into_iter());
            pictures = rest;
            label = "";
        }
    }
}
