//! Showing where what is held would go.

use bevy::prelude::*;
use shared::item::{Holder, Inventory, Rotation};

use super::super::hold::Target;
use super::super::{Carried, Holding};
use super::GridOwner;
use crate::dom::{GridAim, GridShown};

/// Colours the cells a drop would take, green if it can go there and red if not.
pub(in crate::inventory) fn show_aim(
    carried: Res<Carried>,
    holding: Res<Holding>,
    mut grids: Query<(&GridOwner, &mut GridShown)>,
    mut drawn: Local<Option<(Target, Rotation)>>,
) {
    let held = holding.held.map(|held| held.item);
    let aim = holding
        .aim
        .zip(held)
        .map(|(aim, held)| (aim, held.rotation));
    if *drawn == aim {
        return;
    }
    *drawn = aim;
    let (holder, aim) = aimed(aim, held.is_some(), &holding, &carried.0).unzip();
    for (owner, mut shown) in &mut grids {
        let now = aim.clone().filter(|_| Some(owner.holder) == holder);
        if shown.aim != now {
            shown.aim = now;
        }
    }
}

/// The grid aimed at, and the cells there a drop would take.
fn aimed(
    aim: Option<(Target, Rotation)>,
    held: bool,
    holding: &Holding,
    carried: &Inventory,
) -> Option<(Holder, GridAim)> {
    let (aim, rotation) = aim.filter(|_| held)?;
    let item = holding.item(carried)?;
    let (holder, cells, allowed) = match aim {
        Target::Nowhere
        | Target::Bin
        | Target::Drop { .. }
        | Target::Body { .. }
        | Target::Someone(_) => return None,
        Target::Cells {
            holder,
            x,
            y,
            allowed,
        } => {
            let shape = item.shape().turned(rotation);
            let cells: Vec<_> = shape
                .cells()
                .map(|(cell_x, cell_y)| (x + cell_x as i32, y + cell_y as i32))
                .collect();
            (holder, cells, allowed)
        }
        Target::Bag { bag, place, onto } => {
            let (holder, placed) = carried.placed(bag)?;
            let cells = placed.cells().map(|(x, y)| (x as i32, y as i32)).collect();
            (holder, cells, place.is_some() || onto.is_some())
        }
        Target::Stack { onto, allowed } => {
            let (holder, placed) = carried.placed(onto)?;
            let cells = placed.cells().map(|(x, y)| (x as i32, y as i32)).collect();
            (holder, cells, allowed)
        }
    };
    Some((holder, GridAim { cells, allowed }))
}
