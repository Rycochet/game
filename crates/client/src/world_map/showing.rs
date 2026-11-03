//! Showing and hiding the scene as the view changes.

use bevy::prelude::*;

use super::{HEIGHT_SCALE, WorldMapScene};
use crate::common::CurrentWorld;
use crate::view3d::{TokenView, ViewGrid};

/// Becoming the current view: shows the scene, adds the ambient fill (`lights`
/// keeps it up to date), and tells `view3d` the grid is 1 unit per point with the camera looking at
/// sea level, so it doesn't bob over hills.
pub(super) fn show_scene(
    mut commands: Commands,
    world: Res<CurrentWorld>,
    mut scene: Query<&mut Visibility, With<WorldMapScene>>,
) {
    for mut visibility in &mut scene {
        *visibility = Visibility::Inherited;
    }
    commands.insert_resource(GlobalAmbientLight::default());
    commands.insert_resource(ViewGrid {
        cell_size: 1.0,
        floor: world.params.sea_level as f32 * HEIGHT_SCALE,
    });
}

/// Leaving the view: hides the scene, and shows every token again (other
/// views draw all of them).
pub(super) fn hide_scene(
    mut scene: Query<&mut Visibility, (With<WorldMapScene>, Without<TokenView>)>,
    mut tokens: Query<&mut Visibility, With<TokenView>>,
) {
    for mut visibility in &mut scene {
        *visibility = Visibility::Hidden;
    }
    for mut visibility in &mut tokens {
        *visibility = Visibility::Inherited;
    }
}
