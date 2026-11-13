//! Making a character: the screen the server asks for when the player's character
//! is new. A name, a kind and its looks, and three triangles turning together to
//! choose the nine attributes, each with a radar chart behind it (the kind's
//! bonus its inner layer). See `docs/context/client/character_create.md`.

use bevy::prelude::*;

use crate::common::MakingCharacter;

mod buttons;
mod difficulty;
mod draft;
mod form;
mod gender;
mod geometry;
mod input;
mod looks;
mod name;
mod picture;
mod screen;
mod show;

pub struct CharacterCreatePlugin;

impl Plugin for CharacterCreatePlugin {
    fn build(&self, app: &mut App) {
        let making = |making: Res<MakingCharacter>| making.0;
        app.init_resource::<draft::Draft>()
            .init_resource::<input::Holding>()
            .init_resource::<name::NameRolls>()
            .init_resource::<looks::PatternPictures>()
            .add_systems(Startup, screen::spawn_screen)
            .add_systems(Update, (screen::bind_screen, form::show_screen).chain())
            .add_systems(
                Update,
                (
                    form::read_choices,
                    gender::choose_gender,
                    form::fill_looks,
                    difficulty::choose_difficulty,
                    difficulty::fill_difficulty,
                    name::suggest_names,
                    input::point_at_triangles,
                    input::keys_for_triangles,
                    buttons::press_triangle_buttons,
                    form::press_begin,
                    show::draw_triangles,
                    show::place_labels,
                    show::show_values,
                    looks::show_look_labels,
                )
                    .chain()
                    .after(form::show_screen)
                    .run_if(making),
            );
    }
}
