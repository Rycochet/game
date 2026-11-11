//! Giving the panels built from markup what their systems know them by:
//! each part is found by its id as it is built.

use bevy::prelude::*;

use super::{change_creature, steps, teleport_to_world, vitals, world_time};
use crate::dom::Dom;
use crate::widgets::UiId;

pub(in crate::debug_menu) fn bind_panels(
    mut commands: Commands,
    new: Query<(Entity, &UiId), Added<UiId>>,
    mut dom: Dom,
) {
    for (entity, id) in &new {
        let mut bound = commands.entity(entity);
        match id.parts().0 {
            "debug.vitals" => vitals::bind(&mut bound, id),
            "debug.change_creature" => change_creature::bind(&mut bound, id, &mut dom),
            "debug.drawing_states" => steps::bind(&mut bound, id),
            "debug.world_time" => world_time::bind(&mut bound, id),
            teleport_to_world::FORM => teleport_to_world::bind(&mut bound, id, &mut dom),
            _ => {}
        }
    }
}
