//! Where a held item would go: the rules for choosing, without Bevy, so they can
//! be tested alone. The rules for whether it may are `shared::item`'s.

use shared::id::Id;
use shared::item::{Holder, Inventory, Placed, Rotation};

/// An item picked up and following the pointer. Nothing has changed anywhere
/// until it is dropped: it is still where it was, only not drawn there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Held {
    pub id: Id,
    pub rotation: Rotation,
    /// Which cell of its shape, as turned, is under the pointer.
    pub grip: (u8, u8),
}

impl Held {
    /// A quarter turn further on, keeping hold of the same cell.
    pub fn turned(self, inventory: &Inventory) -> Self {
        let Some(item) = inventory.find(self.id) else {
            return self;
        };
        let shape = item.kind.def().shape.turned(self.rotation);
        let quarter = Rotation::Quarter;
        Self {
            rotation: self.rotation.next(),
            grip: quarter.turn_cell(self.grip.0, self.grip.1, shape.width, shape.height),
            ..self
        }
    }
}

/// What dropping a held item would do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    /// It isn't over a grid: it would snap back.
    Nowhere,
    /// On the bin: it and all in it would be destroyed.
    Bin,
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
    /// On a closed bag: wherever there is room in it, if there is.
    Bag {
        bag: Id,
        place: Option<(u8, u8, Rotation)>,
    },
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
) -> Target {
    let Some((holder, cell_x, cell_y)) = over else {
        return Target::Nowhere;
    };
    let Some(grid) = inventory.grid(holder) else {
        return Target::Nowhere;
    };
    if let Some(placed) = on_cell(grid, (cell_x, cell_y), held.id)
        && placed.item.kind.def().container.is_some()
    {
        let bag = placed.item.id;
        let place = inventory.first_place_for(held.id, Holder::Inside(bag), held.rotation);
        return Target::Bag { bag, place };
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

/// What to do to carry out a drop: the grid, place and turn to ask for. `None`
/// snaps back: it can't go there, or that is where it already is.
pub(super) fn drop_move(
    inventory: &Inventory,
    held: Held,
    target: Target,
) -> Option<(Holder, u8, u8, Rotation)> {
    let wanted = match target {
        Target::Nowhere
        | Target::Bin
        | Target::Body { .. }
        | Target::Someone(_)
        | Target::Cells { allowed: false, .. }
        | Target::Bag { place: None, .. } => {
            return None;
        }
        Target::Cells { holder, x, y, .. } => (holder, x as u8, y as u8, held.rotation),
        Target::Bag {
            bag,
            place: Some((x, y, rotation)),
        } => (Holder::Inside(bag), x, y, rotation),
    };
    // (Worn things aren't in a grid, so this is never where they already are.)
    let same = inventory
        .placed(held.id)
        .is_some_and(|(holder, placed)| (holder, placed.x, placed.y, placed.rotation) == wanted);
    (!same).then_some(wanted)
}

#[cfg(test)]
mod tests {
    use shared::item::{Item, ItemKind};
    use uuid::Uuid;

    use super::*;

    fn id(n: u128) -> Id {
        Id(Uuid::from_u128(n))
    }

    /// These kinds in the pockets, in the first places they fit, ids from 1.
    fn carrying(kinds: &[ItemKind]) -> Inventory {
        let mut inventory = Inventory::default();
        for (index, &kind) in kinds.iter().enumerate() {
            let item = Item::new(id(index as u128 + 1), kind);
            inventory.give(item).expect("room");
        }
        inventory
    }

    fn held(n: u128) -> Held {
        Held {
            id: id(n),
            rotation: Rotation::None,
            grip: (0, 0),
        }
    }

    #[test]
    fn the_bin_isnt_a_move() {
        let inventory = carrying(&[ItemKind::Coin]);
        assert_eq!(drop_move(&inventory, held(1), Target::Bin), None);
    }

    #[test]
    fn off_every_grid_it_snaps_back() {
        let inventory = carrying(&[ItemKind::Coin]);
        let aim = target(&inventory, held(1), None);
        assert_eq!(aim, Target::Nowhere);
        assert_eq!(drop_move(&inventory, held(1), aim), None);
    }

    #[test]
    fn it_goes_where_the_gripped_cell_is_held() {
        let inventory = carrying(&[ItemKind::Stone]);
        let grip = Held {
            grip: (1, 1),
            ..held(1)
        };
        let aim = target(&inventory, grip, Some((Holder::Pockets, 3, 2)));
        assert_eq!(
            drop_move(&inventory, grip, aim),
            Some((Holder::Pockets, 2, 1, Rotation::None))
        );
    }

    #[test]
    fn hanging_off_the_grid_or_on_something_else_it_snaps_back() {
        let inventory = carrying(&[ItemKind::Stone, ItemKind::Coin]);
        let grip = Held {
            grip: (1, 1),
            ..held(1)
        };
        for over in [(0, 0), (6, 3), (3, 1)] {
            let aim = target(&inventory, grip, Some((Holder::Pockets, over.0, over.1)));
            let spot = matches!(aim, Target::Cells { allowed: false, .. });
            assert!(spot, "{over:?}: {aim:?}");
            assert_eq!(drop_move(&inventory, grip, aim), None);
        }
    }

    #[test]
    fn dropped_back_where_it_was_nothing_needs_asking() {
        let inventory = carrying(&[ItemKind::Stone]);
        let aim = target(&inventory, held(1), Some((Holder::Pockets, 0, 0)));
        assert_eq!(drop_move(&inventory, held(1), aim), None);
    }

    #[test]
    fn on_a_bag_it_goes_inside_if_it_may() {
        let inventory = carrying(&[ItemKind::Pouch, ItemKind::Ring, ItemKind::Stone]);
        let aim = target(&inventory, held(2), Some((Holder::Pockets, 1, 1)));
        assert_eq!(
            drop_move(&inventory, held(2), aim),
            Some((Holder::Inside(id(1)), 0, 0, Rotation::None))
        );
        // A stone is too big for a pouch, so it stays where it was.
        let aim = target(&inventory, held(3), Some((Holder::Pockets, 1, 1)));
        assert!(matches!(aim, Target::Bag { place: None, .. }));
        assert_eq!(drop_move(&inventory, held(3), aim), None);
    }

    #[test]
    fn a_bag_held_over_its_own_place_is_not_dropped_in_itself() {
        let inventory = carrying(&[ItemKind::Pouch]);
        let aim = target(&inventory, held(1), Some((Holder::Pockets, 1, 1)));
        assert!(matches!(aim, Target::Cells { .. }));
    }

    #[test]
    fn turning_keeps_hold_of_the_same_cell() {
        // A sword held by its second cell from the top.
        let inventory = carrying(&[ItemKind::Sword]);
        let sword = Held {
            grip: (0, 1),
            rotation: Rotation::Quarter,
            ..held(1)
        };
        // As it was given, the sword lies across: this grip is on a 5 × 1 shape.
        let across = Held {
            grip: (1, 0),
            ..sword
        };
        let turned = across.turned(&inventory);
        assert_eq!(turned.rotation, Rotation::Half);
        assert_eq!(turned.grip, (0, 1));
        // Four turns come back to where it started.
        let mut round = across;
        for _ in 0..4 {
            round = round.turned(&inventory);
        }
        assert_eq!(round, across);
    }
}
