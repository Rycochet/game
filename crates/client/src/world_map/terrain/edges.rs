//! Flat faces down the window's edges.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use shared::climate::Climate;
use shared::surface::Surface;

use super::super::HEIGHT_SCALE;
use super::{FRONT_BASE, HeightCache, TerrainGrid, ViewSquare, linear};
use crate::surface_style::{SurfaceStyle, hsl_to_rgba};

/// Flat faces down the window's four edges (one mesh): water depth (where
/// underwater) above earth, down to `FRONT_BASE` × the water level in the
/// starting view (the depth below the water shrinks with the zoom, so the
/// frame keeps its shape on screen). They face
/// outwards, so only the sides towards the camera are drawn. The top of each
/// face follows the terrain exactly, at every point where the edge crosses a
/// triangle.
#[allow(clippy::too_many_arguments)]
pub fn build_edges(
    grid: &TerrainGrid,
    cache: &mut HeightCache,
    surface: &Surface,
    climate: &Climate,
    style: &SurfaceStyle,
    water_level: f64,
    (cx, cy): (i64, i64),
    square: ViewSquare,
) -> Mesh {
    let water_top = water_level as f32 * HEIGHT_SCALE;
    let base = water_top - water_top * (1.0 - FRONT_BASE) * square.zoom();
    let (h, s, l) = style.earth_face;
    let earth_colour = linear(hsl_to_rgba(h, s, l));
    // Each edge from one corner to the next, walked so its face points
    // outwards: near, far, right, left.
    let corner = |u, v| square.at(u, v);
    let edges = [
        (corner(-1.0, 1.0), corner(1.0, 1.0)),
        (corner(1.0, -1.0), corner(-1.0, -1.0)),
        (corner(1.0, 1.0), corner(1.0, -1.0)),
        (corner(-1.0, -1.0), corner(-1.0, 1.0)),
    ];

    let mut positions = Vec::new();
    let mut colours = Vec::new();
    let mut indices = Vec::new();
    for (start, end) in edges {
        let first = positions.len() as u32;
        let stops = crossings(start, end);
        for &t in &stops {
            let point = start.lerp(end, t);
            let shown = grid.height_at(&grid.shown, point);
            // Earth all the way up on land, including under raised woods.
            let under = grid.height_at(&grid.ground, point);
            let ground = (if under >= water_top { shown } else { under }).max(base);
            let top = shown.max(ground);
            // Half the surface lightness of the nearest point, like the
            // prototype's front slice (this also ices over with the water).
            let (x, z) = (cx + point.x.round() as i64, cy + point.y.round() as i64);
            let sample = cache.get(surface, x, z);
            let west = cache.get(surface, x - 1, z).height;
            let place = cache.climate(climate, surface, x, z);
            // Only the water strip uses this: nearly all water, with a sliver
            // of shore at most.
            let floor = surface.floor();
            let (band, lightness) = style.shade(sample, place, west, water_level, floor, 0.0);
            let water_colour = linear(hsl_to_rgba(band.hue, band.saturation, lightness / 2.0));
            // Earth strip: ground down to the base.
            positions.extend_from_slice(&[[point.x, ground, point.y], [point.x, base, point.y]]);
            colours.extend_from_slice(&[earth_colour, earth_colour]);
            // Water strip: water surface down to the ground (zero height on
            // land).
            positions.extend_from_slice(&[[point.x, top, point.y], [point.x, ground, point.y]]);
            colours.extend_from_slice(&[water_colour, water_colour]);
        }

        for column in 0..stops.len() as u32 - 1 {
            let here = first + column * 4;
            let next = here + 4;
            for strip in [0, 2] {
                let (top, bottom) = (here + strip, here + strip + 1);
                let (next_top, next_bottom) = (next + strip, next + strip + 1);
                // Anticlockwise seen from outside, so the face points out.
                indices.extend_from_slice(&[top, bottom, next_top, next_top, bottom, next_bottom]);
            }
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(indices))
}

/// Fractions (0 to 1, in order) along the line from `start` to `end` where it
/// crosses a grid line or a triangle's diagonal, plus both ends: the points
/// where the terrain under the line changes slope.
pub(super) fn crossings(start: Vec2, end: Vec2) -> Vec<f32> {
    let mut stops = vec![0.0, 1.0];
    // Lines x = k, z = k, and the diagonals x + z = k.
    let measures: [fn(Vec2) -> f32; 3] = [|p| p.x, |p| p.y, |p| p.x + p.y];
    for measure in measures {
        let (a, b) = (measure(start), measure(end));
        if (b - a).abs() < 1e-6 {
            continue;
        }
        let (low, high) = (a.min(b), a.max(b));
        for k in low.ceil() as i64..=high.floor() as i64 {
            stops.push((k as f32 - a) / (b - a));
        }
    }
    stops.sort_by(f32::total_cmp);
    stops.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    stops
}
