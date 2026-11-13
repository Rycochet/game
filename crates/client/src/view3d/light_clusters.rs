//! How far back the camera sorts lights into its depth slices.
//!
//! Bevy lights each part of the screen from a grid of slices, each listing the
//! lights that reach it. By default the last slice ends at the farthest light
//! of a frame or so before, as the graphics card measured it (it comes back a
//! few frames late). Zooming out moves the camera back faster than that
//! catches up, so for those frames a light can lie wholly past the last slice,
//! is listed in none, and the ground it lights goes dark: the glow round a
//! World Gate on the world map vanished while the wheel turned. A fixed end as
//! far as the camera draws can't be outrun.

use bevy::light::cluster::{ClusterConfig, ClusterFarZMode, ClusterZConfig};

/// The camera's light slices: the default grid, ending at `far` (the camera's
/// own drawing distance) whatever the lights do.
pub(super) fn light_clusters(far: f32) -> ClusterConfig {
    match ClusterConfig::default() {
        ClusterConfig::FixedZ {
            total,
            z_slices,
            z_config,
            dynamic_resizing,
        } => ClusterConfig::FixedZ {
            total,
            z_slices,
            z_config: ClusterZConfig {
                far_z_mode: ClusterFarZMode::Constant(far),
                ..z_config
            },
            dynamic_resizing,
        },
        // Not what Bevy starts with; kept as it is.
        other => other,
    }
}
