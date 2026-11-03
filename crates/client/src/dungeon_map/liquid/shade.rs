//! How liquid hides what is under it, and how sides and beds are shaded by depth.

use super::super::mesh::Surface;
use super::{BED_DARKENING, FULL_DEPTH, RIM_DARKENING, SIDE_DARKENING};

/// How much of what is under a liquid's surface can't be seen through it.
pub fn opacity(surface: Surface) -> f32 {
    match surface {
        Surface::Water => 0.35,
        _ => 1.0,
    }
}

/// How a side (of a channel, or a wall) is shaded, `depth` below the floor.
pub fn depth_shade(depth: f32) -> f32 {
    1.0 - RIM_DARKENING - SIDE_DARKENING * depth / FULL_DEPTH
}

/// How the bed is shaded, `depth` below the floor.
pub(super) fn bed_shade(depth: f32) -> f32 {
    depth_shade(depth) - BED_DARKENING
}
