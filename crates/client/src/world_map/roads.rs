//! What all the drawing of roads shares: their widths and smoothed courses.
//! `terrain::build_roads` makes the mesh, and `rivers::KnownRivers` clears the
//! woods round them.

use bevy::prelude::*;
use shared::road::{Road, RoadKind};

use super::rivers::UNITS_PER_PIXEL;

/// Points either side averaged to take the grid's corners out of a road.
const SMOOTHING: usize = 3;

/// How wide a road is, in grid units: 1 to 4 pixels at the starting zoom,
/// smallest to largest (like rivers, so they scale with the map).
pub(super) fn width(kind: RoadKind) -> f32 {
    let pixels = match kind {
        RoadKind::Path => 1.0,
        RoadKind::Track => 2.0,
        RoadKind::Road => 3.0,
        RoadKind::Highway => 4.0,
    };
    pixels * UNITS_PER_PIXEL
}

/// A road's course with the grid's corners taken out, relative to its first
/// point (`road.points[0]`).
pub(super) fn smoothed(road: &Road) -> Vec<Vec2> {
    let start = road.points[0];
    let last = road.points.len() - 1;
    let relative = |i: usize| {
        let (x, y) = road.points[i];
        Vec2::new((x - start.0) as f32, (y - start.1) as f32)
    };
    (0..=last)
        .map(|i| {
            // A shorter window near the ends, so they stay at the towns.
            let reach = SMOOTHING.min(i).min(last - i);
            let total: Vec2 = (i - reach..=i + reach).map(relative).sum();
            total / (2 * reach + 1) as f32
        })
        .collect()
}

/// How far east of `centre` a world `x` is, the way round the planet that is
/// nearest.
pub(super) fn east_of(x: i64, centre: i64, world_width: i64) -> i64 {
    let mut east = (x - centre).rem_euclid(world_width);
    if east > world_width / 2 {
        east -= world_width;
    }
    east
}
