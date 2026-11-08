//! Keeping the lit water's flow clock (`mesh::flow`): seconds, wrapping every
//! `PERIOD`, so the picture of water on raised wilderness ground moves
//! downhill. Still with Reduce Motion (owner) and off the wilderness.

use bevy::prelude::*;

use super::mesh::Surface;
use super::parts::SurfaceMaterials;
use super::sight_lines::SurfaceMaterial;
use crate::common::{Preferences, WildCellHeights};

/// How often the flow clock wraps, in seconds: every band's speed is a whole
/// number of pictures in it (`SLOWEST_FLOW` in `sight_mask.wesl`), so the
/// picture never jumps.
const PERIOD: f32 = 8.0;

/// Sets the lit water's flow clock each frame.
pub(super) fn run_flow(
    time: Res<Time>,
    preferences: Res<Preferences>,
    heights: Res<WildCellHeights>,
    surfaces: Res<SurfaceMaterials>,
    mut materials: ResMut<Assets<SurfaceMaterial>>,
) {
    let flowing = heights.heights.is_some() && !preferences.video.reduce_motion;
    let seconds = match flowing {
        // Never quite 0, which means still.
        true => (time.elapsed_secs() % PERIOD).max(1.0e-3),
        false => 0.0,
    };
    let water = &surfaces.solid[Surface::Water as usize];
    let current = materials.get(water).map(|m| m.extension.flow_time());
    if current.is_some_and(|now| now != seconds)
        && let Some(mut material) = materials.get_mut(water)
    {
        material.extension.set_flow_time(seconds);
    }
}
