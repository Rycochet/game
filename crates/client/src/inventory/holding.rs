//! `Holding`: what is picked up and by what, and letting go of it with Esc.

use bevy::prelude::*;
use shared::item::{Inventory, Item};

use super::hold::{Held, Target};

/// What is picked up, if anything.
#[derive(Resource, Default)]
pub(super) struct Holding {
    pub(super) held: Option<HeldBy>,
    /// What dropping it now would do.
    pub(super) aim: Option<Target>,
    /// Where the pointer last was, for one just lifted, in the interface's pixels.
    pub(super) at: Option<Vec2>,
    /// If it is still someone else's: who, and the thing itself.
    pub(super) stock: Option<(shared::id::Id, Item)>,
    /// Where (in the interface's pixels) and when the mouse picked it up.
    pub(super) picked: Option<(Vec2, f64)>,
}

impl Holding {
    /// What is held.
    pub(super) fn item<'a>(&'a self, carried: &'a Inventory) -> Option<&'a Item> {
        match &self.stock {
            Some((_, item)) => Some(item),
            None => carried.find(self.held?.item.id),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct HeldBy {
    pub(super) item: Held,
    pub(super) by: Hand,
}

/// What let go of it drops it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Hand {
    /// The left button, until it is let go.
    Mouse,
    /// A click, until the next.
    Clicked,
    Finger,
    /// Nothing does (`InventoryDebug::PickUp`).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    Debug,
}

/// Esc puts back what is held, and goes no further.
pub(super) fn cancel_held(mut keys: ResMut<ButtonInput<KeyCode>>, mut holding: ResMut<Holding>) {
    if holding.held.is_some() && keys.clear_just_pressed(KeyCode::Escape) {
        holding.held = None;
        holding.aim = None;
        holding.stock = None;
    }
}
