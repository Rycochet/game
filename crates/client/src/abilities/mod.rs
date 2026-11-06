//! Abilities: the Character panel's Abilities page, every ability the character
//! knows, and (next) the action bar. See `docs/context/client/abilities.md`.

use bevy::prelude::*;

mod icons;
mod page;

pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, page::fill_pages);
    }
}

#[cfg(test)]
mod tests;
