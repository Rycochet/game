//! Closing the view: everything it made goes, its pictures with it.

use bevy::prelude::*;

use super::showing::PlanetView;
use crate::common::CurrentWorld;

/// Closes the view, if open.
pub(super) fn close(view: &mut PlanetView, commands: &mut Commands) {
    if let Some(showing) = view.showing.take() {
        commands.entity(showing.layer).despawn();
        commands.entity(showing.globe.scene.root).despawn();
    }
}

/// Without a world (the connection gone) there is nothing to show.
pub(super) fn close_without_world(
    mut view: ResMut<PlanetView>,
    world: Option<Res<CurrentWorld>>,
    mut commands: Commands,
) {
    if world.is_none() {
        close(&mut view, &mut commands);
    }
}
