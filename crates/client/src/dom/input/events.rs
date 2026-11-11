//! What happens to elements, as in a browser: sent to the element in front
//! and then up through everything it is in, until an observer stops it
//! (`event.propagate(false)`). Watch one on an element with
//! `commands.entity(element).observe(|click: On<Click>| …)`.

// What each event carries is read by the controls, which come next.
#![allow(dead_code)]

use bevy::prelude::*;

/// The left button went down on this.
#[derive(EntityEvent, Clone, Copy, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Press {
    pub entity: Entity,
    /// Where, in interface pixels.
    pub at: Vec2,
}

/// The left button was let go over this.
#[derive(EntityEvent, Clone, Copy, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Release {
    pub entity: Entity,
    pub at: Vec2,
}

/// A press and its letting go were both on this (or on things in it). A
/// finger's tap is one too.
#[derive(EntityEvent, Clone, Copy, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Click {
    pub entity: Entity,
    pub at: Vec2,
    /// 2 for the second click of a double click, otherwise 1.
    pub count: u8,
}

/// The right button went down on this.
#[derive(EntityEvent, Clone, Copy, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct ContextMenu {
    pub entity: Entity,
    pub at: Vec2,
}

/// The wheel turned over this.
#[derive(EntityEvent, Clone, Copy, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Wheel {
    pub entity: Entity,
    /// Notches; positive is up.
    pub lines: f32,
}

/// The pointer came onto this element or something in it. Not passed up.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct PointerEnter {
    pub entity: Entity,
}

/// The pointer left this element and everything in it. Not passed up.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct PointerLeave {
    pub entity: Entity,
}
