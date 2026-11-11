//! Making the cells `dom`'s grids draw ours.

use bevy::prelude::*;

use super::{GridNode, GridOwner, ItemCell};
use crate::dom::{GridBox, ThingCell};

/// Marks each grid's cells as drawn, and each cell of an item in one, for
/// finding what the pointer is over and picking things up.
pub(in crate::inventory) fn mark_grids(
    boxes: Query<(&GridOwner, &GridBox), Changed<GridBox>>,
    cells: Query<(Entity, &ThingCell), Added<ThingCell>>,
    owners: Query<(), With<GridOwner>>,
    mut commands: Commands,
) {
    for (owner, GridBox(cells)) in &boxes {
        let holder = owner.holder;
        commands.entity(*cells).try_insert(GridNode { holder });
    }
    for (entity, cell) in &cells {
        if owners.contains(cell.grid) {
            commands.entity(entity).try_insert(ItemCell {
                id: cell.item,
                rotation: cell.rotation,
                grip: cell.grip,
            });
        }
    }
}
