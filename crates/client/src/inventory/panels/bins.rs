//! Showing the bin and the ground to drop on: slots drawing themselves
//! (`dom::SlotShown`), told here what they show.

use bevy::prelude::*;

use super::super::hold::Target;
use super::super::{Carried, Holding};
use super::{BinIcon, BinSlot, DropIcon, DropNode};
use crate::dom::{SlotLook, SlotShown, SlotThing};

/// The bin's picture, what is in it (but not while that is held), and whether
/// dropping something on it would throw that away for good.
pub(in crate::inventory) fn show_bins(
    icon: Res<BinIcon>,
    carried: Res<Carried>,
    holding: Res<Holding>,
    mut bins: Query<&mut SlotShown, With<BinSlot>>,
) {
    let held = holding.held.map(|held| held.item.id);
    let inside = carried.0.bin.as_ref().filter(|item| Some(item.id) != held);
    let aim = match holding.aim {
        Some(Target::Bin) => Some(carried.0.bin.is_none()),
        _ => None,
    };
    for mut shown in &mut bins {
        // (Only when it differs: the slot draws itself again when changed.)
        let same = shown.thing.as_ref().map(|thing| &thing.item) == inside;
        if same && shown.aim == aim && shown.icon.is_some() {
            continue;
        }
        let thing = inside.map(|item| SlotThing {
            item: item.clone(),
            count: 1,
            faded: false,
            pressable: true,
        });
        *shown = SlotShown {
            look: SlotLook::Kept,
            icon: Some(icon.0.clone()),
            thing,
            aim,
        };
    }
}

/// The Drop icon's picture, and what dropping something on it would do.
pub(in crate::inventory) fn show_drop(
    icon: Res<DropIcon>,
    holding: Res<Holding>,
    mut icons: Query<&mut SlotShown, With<DropNode>>,
) {
    let aim = match holding.aim {
        Some(Target::Drop { allowed }) => Some(allowed),
        _ => None,
    };
    for mut shown in &mut icons {
        if shown.aim != aim || shown.icon.is_none() {
            shown.icon = Some(icon.0.clone());
            shown.aim = aim;
        }
    }
}
