//! How far woods raise the drawn surface, the highest shown height, and linear colours.

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

/// The highest drawn surface within `radius` of a fractional position, so
/// something that wide can rest on top without sinking into a slope.
#[allow(clippy::too_many_arguments)]
pub fn highest_shown_height(
    cache: &mut HeightCache,
    surface: &Surface,
    climate: &Climate,
    vegetation: &Vegetation,
    rivers: &KnownRivers,
    style: &SurfaceStyle,
    water_level: f64,
    snow_line: f64,
    (x, y): (f64, f64),
    radius: f64,
) -> f32 {
    let mut highest = f64::MIN;
    let reach = radius.ceil() as i64;
    for dy in -reach..=reach {
        for dx in -reach..=reach {
            let (px, py) = (x.round() as i64 + dx, y.round() as i64 + dy);
            if (px as f64 - x).hypot(py as f64 - y) <= radius {
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
                highest = highest.max(style.shown_height(sample, water_level) + lift as f64);
            }
        }
    }
    highest as f32 * HEIGHT_SCALE
}

/// sRGB bytes to the linear floats meshes use.
pub(super) fn linear([r, g, b, _]: [u8; 4]) -> [f32; 4] {
    Color::srgb_u8(r, g, b).to_linear().to_f32_array()
}
