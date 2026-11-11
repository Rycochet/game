//! Making the cells `dom`'s grids and slots draw ours.

use bevy::prelude::*;

use super::super::body::BodyNode;
use super::{BinNode, GridNode, GridOwner, ItemCell};
use crate::dom::{GridBox, SlotShown, ThingCell};

/// Marks each grid's cells as drawn, and each cell of an item in a grid or a
/// slot, for finding what the pointer is over and picking things up.
pub(in crate::inventory) fn mark_grids(
    boxes: Query<(&GridOwner, &GridBox), Changed<GridBox>>,
    cells: Query<(Entity, &ThingCell), Added<ThingCell>>,
    owners: Query<(), With<GridOwner>>,
    slots: Query<(Has<BodyNode>, Has<BinNode>), With<SlotShown>>,
    mut commands: Commands,
) {
    for (owner, GridBox(cells)) in &boxes {
        let holder = owner.holder;
        commands.entity(*cells).try_insert(GridNode { holder });
    }
    for (entity, cell) in &cells {
        let ours = ItemCell {
            id: cell.item,
            rotation: cell.rotation,
            grip: cell.grip,
        };
        let mut entity = commands.entity(entity);
        // What is in a slot is part of the clothes, or of the bin, as the slot.
        match slots.get(cell.grid) {
            Ok((true, _)) => entity.try_insert((ours, BodyNode)),
            Ok((_, true)) => entity.try_insert((ours, BinNode)),
            Ok(_) => continue,
            Err(_) if owners.contains(cell.grid) => entity.try_insert(ours),
            Err(_) => continue,
        };
    }
}
