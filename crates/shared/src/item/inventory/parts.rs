//! Parts of stacks: an id from inside a stack stands for it and everything above it.
//! Lifted out, a part lies where its stack does until a move takes it somewhere;
//! one still lying there goes back on top (`settle_parts`).

use crate::id::Id;
use crate::item::{Holder, Inventory, Item, Placed};

impl Inventory {
    /// The id standing for the top `count` of this carried stack: its own if that is
    /// all of it. `None` for none, more than it has, or no such stack.
    pub fn part_of(&self, stack: Id, count: u32) -> Option<Id> {
        let item = self.find(stack)?;
        let all = item.count();
        if count == 0 || count > all {
            return None;
        }
        item.members().nth((all - count) as usize).map(|one| one.id)
    }

    /// Lifts the part of a stack in a grid that `id` stands for out on its own, lying
    /// where the stack does, so any move can take it. The stack it came from, if it was
    /// in one.
    pub fn lift_part(&mut self, id: Id) -> Option<Id> {
        let (holder, stack, x, y, rotation) = self.stack_holding(id)?;
        let grid = self.grid_mut(holder)?;
        let placed = grid.iter_mut().find(|placed| placed.item.id == stack)?;
        let index = placed.item.above.iter().position(|one| one.id == id)?;
        let count = (placed.item.above.len() - index) as u32;
        let part = placed.item.split_top(count);
        grid.push(Placed {
            item: part,
            x,
            y,
            rotation,
        });
        Some(stack)
    }

    /// Lifts these items (some, not all) out of a stack in a grid as a stack of their
    /// own, in the order they were in, lying where the stack does: a group from a
    /// mixed stack. What is left keeps its order, and the id of its lowest. The
    /// group's id, which is its lowest's.
    pub fn lift_group(&mut self, stack: Id, items: &[Id]) -> Option<Id> {
        let (holder, placed) = self.placed(stack)?;
        let (x, y, rotation) = (placed.x, placed.y, placed.rotation);
        let ids: Vec<Id> = placed.item.members().map(|one| one.id).collect();
        let taking = ids.iter().filter(|id| items.contains(id)).count();
        if taking == 0 || taking == ids.len() || taking != items.len() {
            return None;
        }
        let mut whole = self.take(stack)?;
        let mut all = std::mem::take(&mut whole.above);
        all.insert(0, whole);
        let (group, kept): (Vec<Item>, Vec<Item>) =
            all.into_iter().partition(|one| items.contains(&one.id));
        let group_id = group[0].id;
        let grid = self.grid_mut(holder)?;
        for part in [kept, group] {
            grid.push(Placed {
                item: restacked(part),
                x,
                y,
                rotation,
            });
        }
        Some(group_id)
    }

    /// Puts back on its stack any part lifted out and not moved: lying just where an
    /// item of its kind does. Whether anything went back.
    pub fn settle_parts(&mut self) -> bool {
        let mut settled = false;
        let mut holders = vec![Holder::Inventory];
        holders.extend(self.containers().into_iter().map(Holder::Inside));
        for holder in holders {
            while let Some((part, stack)) = self.lying_on(holder) {
                let Some(item) = self.take(part) else { break };
                if let Some(onto) = self.find_mut(stack) {
                    onto.put_on(item);
                }
                settled = true;
            }
        }
        settled
    }

    /// The stack in a grid that `id` is inside (not its bottom), and where it lies.
    fn stack_holding(&self, id: Id) -> Option<(Holder, Id, u8, u8, crate::item::Rotation)> {
        let mut holders = vec![Holder::Inventory];
        holders.extend(self.containers().into_iter().map(Holder::Inside));
        holders.into_iter().find_map(|holder| {
            let placed = self.grid(holder)?.iter().find(|p| p.item.has_above(id))?;
            Some((holder, placed.item.id, placed.x, placed.y, placed.rotation))
        })
    }

    /// A part lying on a stack in this grid, and that stack: the later of two in
    /// the same place is the part.
    fn lying_on(&self, holder: Holder) -> Option<(Id, Id)> {
        let grid = self.grid(holder)?;
        grid.iter().enumerate().find_map(|(index, part)| {
            grid[..index]
                .iter()
                .find(|stack| same_place(stack, part) && stack.item.kind == part.item.kind)
                .map(|stack| (part.item.id, stack.item.id))
        })
    }

    /// Every carried container with a grid, at any depth.
    fn containers(&self) -> Vec<Id> {
        fn search(placed: &[Placed], found: &mut Vec<Id>) {
            for one in placed {
                collect(&one.item, found);
            }
        }
        fn collect(item: &Item, found: &mut Vec<Id>) {
            if item.kind.def().container.is_some() {
                found.push(item.id);
                search(&item.contents, found);
            }
        }
        let mut found = Vec::new();
        search(&self.items, &mut found);
        for item in &self.worn {
            collect(item, &mut found);
        }
        found
    }
}

/// Items from the bottom up as one stack.
fn restacked(mut items: Vec<Item>) -> Item {
    let mut bottom = items.remove(0);
    bottom.above = items;
    bottom
}

fn same_place(a: &Placed, b: &Placed) -> bool {
    (a.x, a.y, a.rotation) == (b.x, b.y, b.rotation)
}
