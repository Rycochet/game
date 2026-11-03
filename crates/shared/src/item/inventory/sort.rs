//! Sorting a grid: stacking what looks the same, then laying everything out again
//! from the top left, largest first, in one go.

use std::cmp::Reverse;

use crate::id::Id;
use crate::item::{Holder, Inventory, Item, ItemCategory, Placed, StackLimits};

impl Inventory {
    /// The grid laid out again: what looks the same stacked as far as `limits`
    /// allow, then everything from the largest down in the first place it fits,
    /// rows from the top left, turning only if it must. `None` if something
    /// wouldn't fit, or there is no such grid. Both sides work it out the same,
    /// going only by what clients are told.
    pub fn sorted_grid(&self, holder: Holder, limits: StackLimits) -> Option<Vec<Placed>> {
        let mut items = stacked(self.grid(holder)?, limits);
        items.sort_by_cached_key(|placed| order(&placed.item));
        let mut copy = self.clone();
        copy.grid_mut(holder)?.clear();
        for placed in items {
            let shape = placed.item.shape();
            let (x, y, rotation) = copy.first_room(holder, shape, placed.rotation, None)?;
            copy.grid_mut(holder)?.push(Placed {
                x,
                y,
                rotation,
                ..placed
            });
        }
        copy.grid_mut(holder).map(std::mem::take)
    }

    /// Sorts a grid (`sorted_grid`): whether it changed. Nothing changes if
    /// something wouldn't fit.
    pub fn sort_grid(&mut self, holder: Holder, limits: StackLimits) -> bool {
        let Some(sorted) = self.sorted_grid(holder, limits) else {
            return false;
        };
        let Some(grid) = self.grid_mut(holder) else {
            return false;
        };
        if *grid == sorted {
            return false;
        }
        *grid = sorted;
        true
    }
}

/// What is in a grid, with each put on the first stack before it that has room,
/// as much of it as fits, from its top.
fn stacked(grid: &[Placed], limits: StackLimits) -> Vec<Placed> {
    let mut stacks: Vec<Placed> = Vec::new();
    for placed in grid {
        let mut item = Some(placed.item.clone());
        for stack in &mut stacks {
            let Some(adding) = &mut item else {
                break;
            };
            let room = limits.room(&stack.item, adding);
            if room == 0 {
                continue;
            }
            if room < adding.count() {
                stack.item.put_on(adding.split_top(room));
            } else {
                stack.item.put_on(item.take().expect("checked above"));
            }
        }
        if let Some(item) = item {
            stacks.push(Placed {
                item,
                ..placed.clone()
            });
        }
    }
    stacks
}

/// Largest first: by cells, then by the box round them; then by category and the
/// kind's name, so like goes with like, and last by id so that the order is
/// always the same. Never by the item's own name, which may give away secrets on
/// the server.
fn order(item: &Item) -> (Reverse<usize>, Reverse<u16>, ItemCategory, &'static str, Id) {
    let shape = item.shape();
    let kind = item.top().kind;
    (
        Reverse(shape.cells().count()),
        Reverse(shape.width as u16 * shape.height as u16),
        kind.category(),
        kind.def().name,
        item.id,
    )
}

#[cfg(test)]
mod tests;
