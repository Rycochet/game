//! Which elements the pointer is on: the target and everything it is in, and
//! nothing else, so what is covered never lights up.

use bevy::prelude::*;

use super::events::{PointerEnter, PointerLeave};
use super::hit::Hit;
use crate::dom::parts::{Element, Hovered};

/// Keeps `Hovered` on the elements of the chain, saying when the pointer
/// comes onto one and when it leaves.
pub(super) fn follow_hover(
    mut commands: Commands,
    hit: Res<Hit>,
    elements: Query<(), With<Element>>,
    mut before: Local<Vec<Entity>>,
) {
    if !hit.is_changed() {
        return;
    }
    let now: Vec<Entity> = hit.first_elements(&elements);
    for left in before.iter().filter(|entity| !now.contains(entity)) {
        // One that has gone needs nothing taken off it.
        if elements.contains(*left) {
            commands.entity(*left).try_remove::<Hovered>();
            commands.trigger(PointerLeave { entity: *left });
        }
    }
    for entered in now.iter().filter(|entity| !before.contains(entity)) {
        commands.entity(*entered).try_insert(Hovered);
        commands.trigger(PointerEnter { entity: *entered });
    }
    *before = now;
}

impl Hit {
    /// The elements of the chain.
    fn first_elements(&self, elements: &Query<(), With<Element>>) -> Vec<Entity> {
        let is_element = |entity: &&Entity| elements.contains(**entity);
        self.chain.iter().filter(is_element).copied().collect()
    }
}
