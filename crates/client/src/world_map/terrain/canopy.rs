//! How far woods raise the drawn surface, the drawn height at a point, and linear colours.

use bevy::prelude::*;
use shared::climate::Climate;
use shared::surface::Surface;
use shared::vegetation::Vegetation;

use super::super::HEIGHT_SCALE;
use super::super::rivers::KnownRivers;
use super::{CANOPY_FULL, CANOPY_LUSH, HeightCache};
use crate::surface_style::{SurfaceKind, SurfaceStyle};

/// How far the drawn surface is raised at a point for the trees on it: taller
/// in rainforest. None except on open land.
#[allow(clippy::too_many_arguments)]
pub fn canopy_lift(
    cache: &mut HeightCache,
    surface: &Surface,
    climate: &Climate,
    vegetation: &Vegetation,
    rivers: &KnownRivers,
    style: &SurfaceStyle,
    water_level: f64,
    snow_line: f64,
    (x, z): (i64, i64),
) -> f32 {
    let cover = cache.cover(vegetation, climate, surface, x, z);
    if cover.trees <= 0.0 {
        return 0.0;
    }
    let sample = cache.get(surface, x, z);
    let place = cache.climate(climate, surface, x, z);
    if style.kind(sample, place, water_level, snow_line) != SurfaceKind::Land {
        return 0.0;
    }
    let t = ((cover.trees - CANOPY_FROM) / (CANOPY_FULL - CANOPY_FROM)).clamp(0.0, 1.0);
    // Down to the ground towards the shore, or sand and water stretch up a wall.
    let shore = ((sample.height - water_level) as f32 / CANOPY_SHORE).clamp(0.0, 1.0);
    let height = CANOPY_HEIGHT + CANOPY_LUSH * cover.lush;
    t * t * (3.0 - 2.0 * t) * height * shore * rivers.open_at(x, z)
}

/// Height above the water at which the canopy reaches its full height.
pub(super) const CANOPY_SHORE: f32 = 4.0;
/// How much of the canopy's height counts in the baked slope shading, which is
/// strong enough to turn woods' edges white at full height.
pub(super) const CANOPY_SHADING: f32 = 0.2;

/// How high woods stand above the ground, plus how much more in rainforest.
pub(super) const CANOPY_HEIGHT: f32 = 2.5;

/// How much tree cover it takes for the canopy to start rising, and to be full
/// height.
pub(super) const CANOPY_FROM: f32 = 0.2;

/// The height of the drawn surface at a fractional position: the grid
/// points' drawn heights (woods included) interpolated across the triangle
/// the point is in, split north-east to south-west as `build_window` draws
/// them. Tokens rest with their centre here, so a poi's base meets the ground
/// at the poi itself (owner).
#[allow(clippy::too_many_arguments)]
pub fn shown_height_at(
    cache: &mut HeightCache,
    surface: &Surface,
    climate: &Climate,
    vegetation: &Vegetation,
    rivers: &KnownRivers,
    style: &SurfaceStyle,
    water_level: f64,
    snow_line: f64,
    (x, y): (f64, f64),
) -> f32 {
    let mut at = |px: i64, py: i64| {
        let sample = cache.get(surface, px, py);
        let lift = canopy_lift(
            cache,
            surface,
            climate,
            vegetation,
            rivers,
            style,
            water_level,
            snow_line,
            (px, py),
        );
        style.shown_height(sample, water_level) as f32 * HEIGHT_SCALE + lift
    };
    let (gx, gy) = (x.floor(), y.floor());
    let (fx, fy) = ((x - gx) as f32, (y - gy) as f32);
    let (gx, gy) = (gx as i64, gy as i64);
    let north_west = at(gx, gy);
    let north_east = at(gx + 1, gy);
    let south_west = at(gx, gy + 1);
    if fx + fy <= 1.0 {
        north_west + fx * (north_east - north_west) + fy * (south_west - north_west)
    } else {
        let south_east = at(gx + 1, gy + 1);
        south_east + (1.0 - fx) * (south_west - south_east) + (1.0 - fy) * (north_east - south_east)
    }
}

/// sRGB bytes to the linear floats meshes use.
pub(super) fn linear([r, g, b, _]: [u8; 4]) -> [f32; 4] {
    Color::srgb_u8(r, g, b).to_linear().to_f32_array()
}
