//! `Holding`: what the inventory knows of what is held, and letting go of it
//! with Esc. The holding itself (following the pointer, turning, letting go)
//! is `dom`'s (`HeldObject`).

use bevy::prelude::*;
use shared::id::Id;
use shared::item::{Inventory, Item};

use super::hold::{Held, Target};
pub(super) use crate::dom::Hand;
use crate::dom::{HeldItem, HeldObject, HeldThing, HeldWhat};

/// What is picked up, if anything, as the inventory's rules see it.
#[derive(Resource, Default)]
pub(super) struct Holding {
    /// What `dom` holds, noted each frame (`aim_items`) for what draws by it.
    pub(super) held: Option<HeldBy>,
    /// What dropping it now would do.
    pub(super) aim: Option<Target>,
    /// Where it is, in the interface's pixels, noted with `held`.
    pub(super) at: Option<Vec2>,
    /// If it is still someone else's: who, and the thing itself.
    pub(super) stock: Option<(Id, Item)>,
    /// Some of a mixed stack: the stack, and the items held, from the bottom up.
    pub(super) group: Option<(Id, Vec<Id>)>,
}

impl Holding {
    /// What is held.
    pub(super) fn item<'a>(&'a self, carried: &'a Inventory) -> Option<&'a Item> {
        match &self.stock {
            Some((_, item)) => Some(item),
            None => carried.find(self.held?.item.id),
        }
    }

    /// Picks `item` up as `held`: `picked` is where and when the mouse did.
    pub(super) fn pick_up(
        &mut self,
        object: &mut HeldObject,
        item: &Item,
        held: Held,
        by: Hand,
        picked: Option<(Vec2, f64)>,
    ) {
        let what = HeldWhat::Item(HeldItem {
            id: held.id,
            item: item.clone(),
            rotation: held.rotation,
            grip: held.grip,
        });
        object.pick_up(HeldThing { what, by }, picked);
        self.held = Some(HeldBy { item: held, by });
    }

    /// Lets go without dropping it anywhere.
    pub(super) fn put_back(&mut self, object: &mut HeldObject) {
        object.put_back();
        self.held = None;
        self.aim = None;
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct HeldBy {
    pub(super) item: Held,
    pub(super) by: Hand,
}

impl HeldBy {
    /// The item `dom` holds now, if it holds one.
    pub(super) fn of(object: &HeldObject) -> Option<Self> {
        object.thing().and_then(Self::from_thing)
    }

    /// Nothing for an ability: those are `abilities`'.
    pub(super) fn from_thing(thing: &HeldThing) -> Option<Self> {
        let held = thing.item()?;
        let item = Held {
            id: held.id,
            rotation: held.rotation,
            grip: held.grip,
        };
        Some(Self { item, by: thing.by })
    }
}

/// Esc puts back what is held, and goes no further.
pub(super) fn cancel_held(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut holding: ResMut<Holding>,
    mut object: ResMut<HeldObject>,
) {
    // (An ability dragged too: it goes back where it was.)
    if object.thing().is_some() && keys.clear_just_pressed(KeyCode::Escape) {
        holding.put_back(&mut object);
        holding.stock = None;
        holding.group = None;
    }
}
