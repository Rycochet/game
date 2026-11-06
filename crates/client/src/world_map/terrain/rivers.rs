//! The rivers' strips and banks, clipped to the window.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::super::HEIGHT_SCALE;
use super::super::rivers::KnownRivers;
use super::{BANK_ALPHA, Corner, ICE_BOOST, TerrainGrid, ViewSquare, linear};
use crate::surface_style::{SurfaceStyle, hsl_to_rgba};

/// How far rivers are drawn above the ground, so it doesn't poke through them.
pub(super) const RIVER_LIFT: f32 = 0.25;

/// The dark, fading edge each side of a river, which makes it look set into
/// the ground: its width in grid units, and its darkness next to the water.
pub(super) const BANK_WIDTH: f32 = 0.6;

/// A mesh being built out of triangles clipped to the window.
#[derive(Default)]
pub(super) struct ClippedMesh {
    positions: Vec<[f32; 3]>,
    colours: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl ClippedMesh {
    pub(super) fn add(&mut self, triangle: [Corner; 3]) {
        let clipped = clip(&triangle);
        let first = self.positions.len() as u32;
        for c in &clipped {
            self.positions.push(c.position.to_array());
            self.colours.push(c.colour.to_array());
        }
        for k in 1..clipped.len().saturating_sub(1) as u32 {
            self.indices
                .extend_from_slice(&[first, first + k, first + k + 1]);
        }
    }

    pub(super) fn into_mesh(self) -> Mesh {
        let normals = vec![[0.0, 1.0, 0.0]; self.positions.len()];
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colours)
        .with_inserted_indices(Indices::U32(self.indices))
    }
}

/// The rivers' meshes, the strips and their banks: a thin strip along each
/// river, resting on the drawn terrain and cut to the window, with a dark edge
/// fading out each side. Parts under the water aren't drawn, so a river's mouth
/// moves with the tide; frozen parts are white.
pub fn build_rivers(
    grid: &TerrainGrid,
    rivers: &KnownRivers,
    world_width: i64,
    style: &SurfaceStyle,
    water_level: f64,
    square: ViewSquare,
) -> (Mesh, Mesh) {
    let mut flowing = Vec4::from(linear(hsl_to_rgba(style.water.hue - 20.0, 0.9, 0.5)));
    // Moving water, as far as the shader goes.
    flowing.w = 0.0;
    let mut frozen = Vec4::from(linear(hsl_to_rgba(0.0, 0.0, style.ice_lightness)));
    frozen *= Vec4::new(ICE_BOOST, ICE_BOOST, ICE_BOOST, 1.0);
    let water = water_level as f32 * HEIGHT_SCALE;
    let bank_inner = Vec4::new(0.0, 0.0, 0.0, BANK_ALPHA);
    let bank_outer = Vec4::ZERO;
    let mut strips = ClippedMesh::default();
    let mut banks = ClippedMesh::default();

    for river in rivers.rivers() {
        // The way round the planet that is nearest the player.
        let mut east = (river.source.0 - grid.centre.0).rem_euclid(world_width);
        if east > world_width / 2 {
            east -= world_width;
        }
        let offset = Vec2::new(east as f32, (river.source.1 - grid.centre.1) as f32);
        // Whether a point is near enough the window to have heights kept.
        let near = |point: Vec2| square.inside(point).iter().all(|&d| d >= -1.0);

        for i in 0..river.points.len().saturating_sub(1) {
            let (a, b) = (river.points[i] + offset, river.points[i + 1] + offset);
            if !near(a) || !near(b) {
                continue;
            }
            let under_water = |point: Vec2| grid.height_at(&grid.ground, point) <= water;
            if under_water(a) || under_water(b) {
                continue;
            }
            let ground = grid.height_at(&grid.ground, a) as f64 / HEIGHT_SCALE as f64;
            let frozen_here = style.river_is_frozen(river.temperatures[i], ground, water_level);
            let colour = if frozen_here { frozen } else { flowing };
            let side = (b - a).perp().normalize_or_zero();
            // A corner this far across from the river's middle.
            let corner = |point: Vec2, across: f32, colour: Vec4| {
                let at = point + side * across;
                Corner {
                    position: Vec3::new(at.x, grid.height_at(&grid.shown, at) + RIVER_LIFT, at.y),
                    normal: Vec3::Y,
                    colour,
                    cover: Vec4::ZERO,
                    inside: square.inside(at),
                }
            };
            let (wa, wb) = (river.widths[i] / 2.0, river.widths[i + 1] / 2.0);
            let (a0, a1) = (corner(a, -wa, colour), corner(a, wa, colour));
            let (b0, b1) = (corner(b, -wb, colour), corner(b, wb, colour));
            strips.add([a0, a1, b1]);
            strips.add([a0, b1, b0]);
            for sign in [-1.0, 1.0] {
                let inner_a = corner(a, wa * sign, bank_inner);
                let inner_b = corner(b, wb * sign, bank_inner);
                let outer_a = corner(a, (wa + BANK_WIDTH) * sign, bank_outer);
                let outer_b = corner(b, (wb + BANK_WIDTH) * sign, bank_outer);
                banks.add([inner_a, outer_a, outer_b]);
                banks.add([inner_a, outer_b, inner_b]);
            }
        }
    }
    (strips.into_mesh(), banks.into_mesh())
}

/// Cuts a polygon down to the part inside all four sides of the window
/// (Sutherland–Hodgman clipping), keeping its winding.
pub(super) fn clip(polygon: &[Corner]) -> Vec<Corner> {
    let mut polygon = polygon.to_vec();
    for side in 0..4 {
        let mut kept = Vec::with_capacity(polygon.len() + 1);
        for (i, &a) in polygon.iter().enumerate() {
            let b = polygon[(i + 1) % polygon.len()];
            let (da, db) = (a.inside[side], b.inside[side]);
            if da >= 0.0 {
                kept.push(a);
            }
            // The edge crosses this side: add the crossing point.
            if (da >= 0.0) != (db >= 0.0) {
                kept.push(a.lerp(b, da / (da - db)));
            }
        }
        polygon = kept;
        if polygon.is_empty() {
            break;
        }
    }
    polygon
}
