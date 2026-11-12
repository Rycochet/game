//! Giving the panels built from markup what their systems know them by:
//! each part is found by its id as it is built.

use bevy::prelude::*;

use super::super::bodies::BodiesToShow;
use super::html_sample::HTML_SAMPLE;
use super::{
    change_creature, create_mob, dev_area, examine_mob, generate, give, learn_skills,
    pictures_spawn, steps, teleport_to_world, vitals, world_time,
};
use crate::dom::{Dom, document};
use crate::widgets::UiId;

pub(in crate::debug_menu) fn bind_panels(
    mut commands: Commands,
    new: Query<(Entity, &UiId), Added<UiId>>,
    mut dom: Dom,
) {
    for (entity, id) in &new {
        let mut bound = commands.entity(entity);
        // Show Widgets' sample is a document of its own, and Suns and Moons
        // is still filled by code: each into its holder.
        match id.parts() {
            ("debug.every_widget", "holder") => {
                bound.with_child(document(&HTML_SAMPLE));
            }
            ("debug.suns_and_moons", "holder") => {
                bound.insert(BodiesToShow);
            }
            _ => {}
        }
        match id.parts().0 {
            "debug.vitals" => vitals::bind(&mut bound, id),
            "debug.change_creature" => change_creature::bind(&mut bound, id, &mut dom),
            "debug.drawing_states" => steps::bind(&mut bound, id),
            "debug.world_time" => world_time::bind(&mut bound, id),
            "debug.examine_mob" => examine_mob::bind(&mut bound, id),
            "debug.create_mob" => create_mob::bind(&mut bound, id),
            "debug.dev_area" => dev_area::bind(&mut bound, id),
            "debug.give_item" => give::bind(&mut bound, id),
            "debug.item_pictures" => pictures_spawn::bind(&mut bound, id),
            learn_skills::FORM => learn_skills::bind(&mut bound, id, &mut dom),
            "debug.generate_poi" => generate::bind(&mut bound, id, &mut dom),
            teleport_to_world::FORM => teleport_to_world::bind(&mut bound, id, &mut dom),
            _ => {}
        }
    }
}
