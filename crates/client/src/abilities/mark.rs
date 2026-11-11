//! Making what `dom` draws of abilities ours: the page's abilities, and the
//! parts of the bar's slots with ids.

use bevy::prelude::*;

use super::bar::SlotNode;
use super::page::{PageAbility, PageList, ability_id};
use crate::dom::{AbilityCell, SlotPart};
use crate::widgets::UiId;

/// Marks each ability drawn on the page, for dragging it and for its id, and
/// gives a slot's edge for being on, and its stripes, their ids.
pub(super) fn mark_abilities(
    cells: Query<(Entity, &AbilityCell), Added<AbilityCell>>,
    parts: Query<(Entity, &SlotPart), Added<SlotPart>>,
    lists: Query<(), With<PageList>>,
    slots: Query<&SlotNode>,
    mut commands: Commands,
) {
    for (entity, cell) in &cells {
        if lists.contains(cell.of) {
            let id = UiId::new("abilities", ability_id(cell.ability));
            let marks = (id, PageAbility(cell.ability));
            commands.entity(entity).try_insert(marks);
        }
    }
    for (entity, part) in &parts {
        let (name, slot) = match *part {
            SlotPart::On(slot) => ("on", slot),
            SlotPart::Queued(slot) => ("queued", slot),
            SlotPart::Shade(_) => continue,
        };
        if let Ok(&SlotNode(slot)) = slots.get(slot) {
            let id = UiId::new("bar", name).at(slot as usize);
            commands.entity(entity).try_insert(id);
        }
    }
}
