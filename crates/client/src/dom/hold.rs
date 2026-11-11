//! Holding a thing picked up from an `object-grid` or `object-slot`: it
//! follows the pointer or the finger, turns, and is let go, and what it is
//! over is worked out. Whoever owns the thing says when it is picked up
//! (`HeldObject::pick_up`) and what letting it go somewhere means (`Dropped`):
//! this knows nothing of the rules.

mod follow;
mod show;

use bevy::prelude::*;
use shared::id::Id;
use shared::item::{Item, Rotation};

use crate::common::{Preferences, TouchGesture, UiPointer};

/// When what is held follows the pointer and may be let go (`PreUpdate`). An
/// owner's system picking things up goes `.before` this, and one reading
/// `Dropped`, or what the thing is over, `.after`.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HoldObjects;

/// What holds a thing, and so what lets go of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hand {
    /// The left button, until it is let go.
    Mouse,
    /// A click, until the next.
    Clicked,
    Finger,
    /// Nothing lets go of it: its owner puts it back.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    Kept,
}

/// A thing picked up and following the pointer. Nothing has changed anywhere
/// until it is dropped: it is still where it was.
#[derive(Clone, Debug, PartialEq)]
pub struct HeldThing {
    /// Which thing it is, to its owner.
    pub id: Id,
    /// What it is like: its look and shape. Its owner keeps it up
    /// (`HeldObject::keep_up`).
    pub item: Item,
    pub rotation: Rotation,
    /// Which cell of its shape, as turned, is under the pointer.
    pub grip: (u8, u8),
    pub by: Hand,
}

impl HeldThing {
    /// A quarter turn further on, keeping hold of the same cell.
    pub fn turn(&mut self) {
        let shape = self.item.shape().turned(self.rotation);
        let (x, y) = self.grip;
        self.grip = Rotation::Quarter.turn_cell(x, y, shape.width, shape.height);
        self.rotation = self.rotation.next();
    }
}

/// What a held thing is over.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeldOver {
    /// The node in front at the pointer, of any kind: its owner knows what
    /// it is.
    pub front: Option<Entity>,
    /// The grid in front there (the entity with its `GridShown`), and the
    /// cell of it the pointer is in, which may be off its edge.
    pub cell: Option<(Entity, i32, i32)>,
}

/// What is held, if anything.
#[derive(Resource, Default)]
pub struct HeldObject {
    thing: Option<HeldThing>,
    /// Where it is, in the interface's pixels: the pointer or finger, or
    /// where that last was.
    pub at: Option<Vec2>,
    /// Where (in the interface's pixels) and when the mouse picked it up.
    picked: Option<(Vec2, f64)>,
    pub over: HeldOver,
}

impl HeldObject {
    pub fn thing(&self) -> Option<&HeldThing> {
        self.thing.as_ref()
    }

    /// Picks this up. `picked` is where and when the mouse did, for telling a
    /// click, which keeps hold until the next, from a drag.
    pub fn pick_up(&mut self, thing: HeldThing, picked: Option<(Vec2, f64)>) {
        self.thing = Some(thing);
        if picked.is_some() {
            self.picked = picked;
        }
    }

    /// Lets go without dropping it anywhere: it is where it was.
    pub fn put_back(&mut self) {
        self.thing = None;
    }

    /// The thing as it is now, when its owner hears more of it.
    pub fn keep_up(&mut self, item: &Item) {
        if let Some(thing) = &mut self.thing
            && thing.item != *item
        {
            thing.item = item.clone();
        }
    }
}

/// A held thing was let go: what it was, where, and over what. Its owner
/// decides what that does.
#[derive(Message, Clone, Debug)]
pub struct Dropped {
    pub thing: HeldThing,
    pub at: Option<Vec2>,
    pub over: HeldOver,
}

pub(super) fn add(app: &mut App) {
    app.init_resource::<HeldObject>()
        // (Their own plugins make these too: here for wherever documents
        // are built.)
        .init_resource::<UiPointer>()
        .init_resource::<UiScale>()
        .init_resource::<TouchGesture>()
        .init_resource::<Preferences>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<Dropped>()
        .add_systems(Startup, show::spawn_held)
        .add_systems(PreUpdate, follow::follow_held.in_set(HoldObjects))
        .add_systems(Update, show::show_held);
}

#[cfg(test)]
mod tests;
