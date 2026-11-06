//! The box near the bottom of the window naming what the pointer is over: a
//! creature, a poi, a door, stairs, a bag on the ground, or a sun or moon by
//! the minimap. See `docs/context/client/hover.md`.

mod name;
mod spawn;

use bevy::prelude::*;

use crate::common::GameView;

/// The hover box.
pub struct HoverPlugin;

impl Plugin for HoverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn::spawn_box).add_systems(
            PostUpdate,
            name::show_name.run_if(not(in_state(GameView::Connecting))),
        );
    }
}

#[cfg(test)]
mod tests;
