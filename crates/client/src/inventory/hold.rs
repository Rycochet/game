//! Where a held item would go: the rules for choosing, without Bevy, so they can
//! be tested alone. The rules for whether it may are `shared::item`'s.

use shared::id::Id;
use shared::item::{Holder, Inventory, Item, Placed, Rotation, StackLimits};

/// An item picked up and following the pointer. Nothing has changed anywhere
/// until it is dropped: it is still where it was, only not drawn there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Held {
    pub id: Id,
    pub rotation: Rotation,
    /// Which cell of its shape, as turned, is under the pointer.
    pub grip: (u8, u8),
}

/// What dropping a held item would do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    /// It isn't over a grid: it would snap back.
    Nowhere,
    /// On the bin: it and all in it would be destroyed.
    Bin,
    /// On the Drop icon: it would go on the ground under the player, if it can.
    Drop { allowed: bool },
    /// On someone's row: it would be given to them.
    Someone(Id),
    /// On the clothes panel: it would be put on, if it can be.
    Body { allowed: bool },
    /// In this grid with its top left cell here, which may be off the grid.
    Cells {
        holder: Holder,
        x: i32,
        y: i32,
        allowed: bool,
    },
    /// On something that looks the same: as much as fits would go on its stack.
    Stack { onto: Id, allowed: bool },
    /// On a closed bag: on the first stack in it it all fits on, or else wherever
    /// there is room in it, if there is.
    Bag {
        bag: Id,
        place: Option<(u8, u8, Rotation)>,
        onto: Option<Id>,
    },
}

/// The stack a drop would go on, if that is what it would do and may.
pub(super) fn stack_onto(target: Target) -> Option<Id> {
    match target {
        Target::Stack {
            onto,
            allowed: true,
        }
        | Target::Bag {
            onto: Some(onto), ..
        } => Some(onto),
        _ => None,
    }
}

/// What dropping `item` on this other item would do, if it is a bag or looks the
/// same. `place` finds room in a bag.
fn onto_item(
    inventory: &Inventory,
    item: &Item,
    under: &Placed,
    limits: StackLimits,
    place: impl FnOnce(Id) -> Option<(u8, u8, Rotation)>,
) -> Option<Target> {
    let id = under.item.id;
    if under.item.kind.def().container.is_some() {
        let onto = inventory.first_stack_for(item, Holder::Inside(id), limits);
        return Some(Target::Bag {
            bag: id,
            place: place(id),
            onto,
        });
    }
    under.item.looks_like(item).then(|| Target::Stack {
        onto: id,
        allowed: inventory.stack_room(item, id, limits).is_ok(),
    })
}

/// What is on this cell of a grid, other than the item held.
fn on_cell(grid: &[Placed], cell: (i32, i32), held: Id) -> Option<&Placed> {
    grid.iter().find(|placed| {
        placed.item.id != held && placed.cells().any(|(x, y)| (x as i32, y as i32) == cell)
    })
}

/// What dropping it with the pointer on this cell of this grid would do.
pub(super) fn target(
    inventory: &Inventory,
    held: Held,
    over: Option<(Holder, i32, i32)>,
    limits: StackLimits,
) -> Target {
    let Some((holder, cell_x, cell_y)) = over else {
        return Target::Nowhere;
    };
    let Some(grid) = inventory.grid(holder) else {
        return Target::Nowhere;
    };
    if let Some(placed) = on_cell(grid, (cell_x, cell_y), held.id)
        && let Some(item) = inventory.find(held.id)
        && let Some(aim) = onto_item(inventory, item, placed, limits, |bag| {
            inventory.first_place_for(held.id, Holder::Inside(bag), held.rotation)
        })
    {
        return aim;
    }
    let (x, y) = (cell_x - held.grip.0 as i32, cell_y - held.grip.1 as i32);
    let allowed = u8::try_from(x).is_ok_and(|x| {
        u8::try_from(y).is_ok_and(|y| {
            inventory
                .check_move(held.id, holder, x, y, held.rotation)
                .is_ok()
        })
    });
    Target::Cells {
        holder,
        x,
        y,
        allowed,
    }
}

/// The same for something that isn't carried yet.
pub(super) fn target_for_new(
    inventory: &Inventory,
    item: &Item,
    held: Held,
    over: Option<(Holder, i32, i32)>,
    limits: StackLimits,
) -> Target {
    let Some((holder, cell_x, cell_y)) = over else {
        return Target::Nowhere;
    };
    let Some(grid) = inventory.grid(holder) else {
        return Target::Nowhere;
    };
    if let Some(placed) = on_cell(grid, (cell_x, cell_y), held.id)
        && let Some(aim) = onto_item(inventory, item, placed, limits, |bag| {
            inventory.first_place_for_new(item, Holder::Inside(bag), held.rotation)
        })
    {
        return aim;
    }
    let (x, y) = (cell_x - held.grip.0 as i32, cell_y - held.grip.1 as i32);
    let allowed = u8::try_from(x).is_ok_and(|x| {
        u8::try_from(y).is_ok_and(|y| {
            inventory
                .check_add(item, holder, x, y, held.rotation)
                .is_ok()
        })
    });
    Target::Cells {
        holder,
        x,
        y,
        allowed,
    }
}

/// What to do to carry out a drop: the grid, place and turn to ask for. `None`
/// snaps back: it can't go there, or that is where it already is. Going on a stack
/// is `stack_onto`'s.
pub(super) fn drop_move(
    inventory: &Inventory,
    held: Held,
    target: Target,
) -> Option<(Holder, u8, u8, Rotation)> {
    let wanted = match target {
        Target::Nowhere
        | Target::Bin
        | Target::Drop { .. }
        | Target::Body { .. }
        | Target::Someone(_)
        | Target::Stack { .. }
        | Target::Cells { allowed: false, .. }
        | Target::Bag { place: None, .. }
        | Target::Bag { onto: Some(_), .. } => {
            return None;
        }
        Target::Cells { holder, x, y, .. } => (holder, x as u8, y as u8, held.rotation),
        Target::Bag {
            bag,
            place: Some((x, y, rotation)),
            onto: None,
        } => (Holder::Inside(bag), x, y, rotation),
    };
    // (Worn things aren't in a grid, so this is never where they already are.)
    let same = inventory
        .placed(held.id)
        .is_some_and(|(holder, placed)| (holder, placed.x, placed.y, placed.rotation) == wanted);
    (!same).then_some(wanted)
}

#[cfg(test)]
mod tests;
