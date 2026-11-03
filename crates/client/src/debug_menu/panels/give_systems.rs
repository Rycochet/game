//! What Give Item's tabs and buttons, and Change Creature's buttons, do.

use bevy::prelude::*;
use shared::creature::Size;
use shared::item::{Favour, Quality};
use shared::protocol::ClientMessage;

use super::{
    CreatureButton, GiveButton, GiveFavour, GiveForm, GiveMagic, GiveQualities, GiveSet, GiveSizes,
};
use crate::connection::ToServer;
use crate::widgets::{ButtonPressed, TabChosen};

/// Follows Give Item's tabs.
pub(in crate::debug_menu) fn choose_give_options(
    mut chosen: MessageReader<TabChosen>,
    sizes: Query<(), With<GiveSizes>>,
    magic: Query<(), With<GiveMagic>>,
    favour: Query<(), With<GiveFavour>>,
    set: Query<(), With<GiveSet>>,
    qualities: Query<(), With<GiveQualities>>,
    mut form: ResMut<GiveForm>,
) {
    for &TabChosen(row, index) in chosen.read() {
        if sizes.contains(row) {
            form.size = index.checked_sub(1).map(|index| Size::ALL[index]);
        }
        if magic.contains(row) {
            form.magical = index == 1;
        }
        if favour.contains(row) {
            form.favour = match index {
                1 => Some(Favour::Blessed),
                2 => Some(Favour::Cursed),
                _ => None,
            };
        }
        if set.contains(row) {
            form.set = index == 1;
        }
        if qualities.contains(row) {
            form.quality = Quality::ALL[index];
        }
    }
}

/// What Give Item's buttons do.
pub(in crate::debug_menu) fn press_give_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&GiveButton>,
    form: Res<GiveForm>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok(&GiveButton(kind)) = buttons.get(press.0) {
            to_server.write(ToServer(ClientMessage::DevGiveItem {
                kind,
                size: form.size,
                magical: form.magical,
                quality: form.quality,
                set: form.set,
                favour: form.favour,
            }));
        }
    }
}

/// What Change Creature's buttons do.
pub(in crate::debug_menu) fn press_creature_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&CreatureButton>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok(&CreatureButton(kind)) = buttons.get(press.0) {
            to_server.write(ToServer(ClientMessage::DevSetCreature(kind)));
        }
    }
}
