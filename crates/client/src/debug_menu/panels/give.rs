//! Give Item and Change Creature: their buttons, tabs and form.

use bevy::prelude::*;
use shared::creature::{CreatureKind, Size};
use shared::item::{Favour, ItemKind, Quality};

use super::{FONT, wrapped};
use crate::widgets::{BUTTON_PADDING, ButtonState, Pressable, spawn_button, spawn_toggle_row};

/// How new items are made, and a button for each kind.
pub(super) fn spawn_give_buttons(content: &mut ChildSpawnerCommands) {
    let sizes: Vec<String> = std::iter::once("Mine".to_owned())
        .chain(Size::ALL.iter().map(|size| format!("{size:?}")))
        .collect();
    let sizes: Vec<&str> = sizes.iter().map(String::as_str).collect();
    let row = spawn_toggle_row(content, &sizes, 0, FONT);
    content.commands().entity(row).insert(GiveSizes);
    let row = spawn_toggle_row(content, &["Plain", "Magical"], 0, FONT);
    content.commands().entity(row).insert(GiveMagic);
    let row = spawn_toggle_row(content, &["Plain", "Blessed", "Cursed"], 0, FONT);
    content.commands().entity(row).insert(GiveFavour);
    let row = spawn_toggle_row(content, &["Alone", "Set"], 0, FONT);
    content.commands().entity(row).insert(GiveSet);
    let qualities: Vec<String> = Quality::ALL.iter().map(|q| format!("{q:?}")).collect();
    let qualities: Vec<&str> = qualities.iter().map(String::as_str).collect();
    let common = Quality::ALL
        .iter()
        .position(|&quality| quality == Quality::default())
        .unwrap_or(0);
    let row = spawn_toggle_row(content, &qualities, common, FONT);
    content.commands().entity(row).insert(GiveQualities);
    content.spawn(wrapped()).with_children(|buttons| {
        for kind in ItemKind::ALL {
            let node = Node {
                padding: BUTTON_PADDING,
                ..default()
            };
            spawn_button(buttons, ButtonState::Normal, kind.def().name, FONT, node)
                .insert((GiveButton(kind), Pressable));
        }
    });
}

/// A button for each kind of creature.
pub(super) fn spawn_creature_buttons(content: &mut ChildSpawnerCommands) {
    for kind in CreatureKind::ALL {
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, ButtonState::Normal, kind.def().name, FONT, node)
            .insert((CreatureButton(kind), Pressable));
    }
}

/// Asks the server for an item of this kind.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct GiveButton(pub(super) ItemKind);

/// Asks the server to make the character this kind of creature.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct CreatureButton(pub(super) CreatureKind);

/// The tabs choosing what size new items are made for.
#[derive(Component)]
pub(in crate::debug_menu) struct GiveSizes;

/// The tabs choosing whether new items are magical.
#[derive(Component)]
pub(in crate::debug_menu) struct GiveMagic;

/// The tabs choosing whether new items are blessed or cursed.
#[derive(Component)]
pub(in crate::debug_menu) struct GiveFavour;

/// The tabs choosing whether new items are part of a set.
#[derive(Component)]
pub(in crate::debug_menu) struct GiveSet;

/// The tabs choosing how good new items are.
#[derive(Component)]
pub(in crate::debug_menu) struct GiveQualities;

/// How Give Item makes things.
#[derive(Resource, Default)]
pub(in crate::debug_menu) struct GiveForm {
    /// `None` for the character's own.
    pub(super) size: Option<Size>,
    pub(super) magical: bool,
    pub(super) favour: Option<Favour>,
    pub(super) quality: Quality,
    pub(super) set: bool,
}
