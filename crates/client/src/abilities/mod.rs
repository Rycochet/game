//! Abilities: the Character panel's Abilities page, every ability the character
//! knows, and the action bar either side of the Health and Mana circle. See
//! `docs/context/client/abilities.md`.

use bevy::prelude::*;

use crate::common::PointerSystems;
use crate::widgets::WidgetSystems;

mod bar;
mod drag;
mod held;
mod icons;
mod own_bar;
mod page;
mod spots;

pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<own_bar::OwnBar>()
            .init_resource::<drag::Drag>()
            .init_resource::<drag::Aim>()
            .add_systems(Startup, (bar::spawn_bar, held::spawn_held))
            // Before widgets take presses on panels, and the map clicks.
            .add_systems(
                PreUpdate,
                drag::drag_abilities
                    .after(PointerSystems)
                    .before(WidgetSystems::Presses),
            )
            .add_systems(
                Update,
                (
                    page::fill_pages,
                    page::open_skills,
                    own_bar::take_bar,
                    held::use_by_key,
                    (bar::place_bar, bar::show_bar, held::show_held),
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests;
