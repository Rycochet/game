//! The roads' strips, clipped to the window, with bridges where they cross
//! rivers.

use std::collections::BTreeSet;

use bevy::mesh::Mesh;
use bevy::prelude::*;
use shared::road::RoadKind;

use super::super::rivers::KnownRivers;
use super::super::roads::{east_of, smoothed, width};
use super::rivers::{ClippedMesh, RIVER_LIFT};
use super::{Corner, TerrainGrid, ViewSquare, linear};
use crate::common::KnownRoads;
use crate::surface_style::hsl_to_rgba;

/// How far the smallest road is drawn above the ground, and how much higher
/// each larger kind is, so where roads overlap the larger is on top (owner's
/// rule). All below rivers, which cross over them on bridges.
const ROAD_LIFT: f32 = 0.08;
const LIFT_PER_KIND: f32 = 0.05;

/// How much of a road's own colour shows through snow: paths not at all (they
/// are the land's colour), larger roads as darker snow.
const SHOWS_THROUGH_SNOW: f32 = 0.3;

/// How much lighter than the land a path is, worn by feet: enough to see
/// among the slope shading, still the land's colour (its hue kept).
const PATH_WORN: f32 = 1.8;

/// How much of a highway's width is its pale middle, the rest dark edges.
const HIGHWAY_MIDDLE: f32 = 0.65;

/// How much wider a bridge is than its road, in grid units, and how far above
/// the river it is drawn.
const BRIDGE_EXTRA: f32 = 0.3;
const BRIDGE_LIFT: f32 = 0.3;

/// The roads' mesh: a strip along each, resting on the drawn terrain and cut
/// to the window. Paths take the land's colour, a little worn (they show
/// mostly because the woods are cleared round them), tracks are brown, roads grey and
/// highways pale stone with dark edges; where one crosses a river it is on a
/// bridge, of planks or of stone.
pub fn build_roads(
    grid: &TerrainGrid,
    roads: &KnownRoads,
    rivers: &KnownRivers,
    world_width: i64,
    square: ViewSquare,
) -> Mesh {
    let near = |point: Vec2| square.inside(point).iter().all(|&d| d >= -1.0);
    let wet = river_points(grid, rivers, world_width, near);
    let crosses_river = |point: Vec2| {
        let (x, z) = (point.x.round() as i64, point.y.round() as i64);
        (-1..=1).any(|dz| (-1..=1).any(|dx| wet.contains(&(x + dx, z + dz))))
    };
    let mut sorted: Vec<_> = roads.iter().collect();
    sorted.sort_by_key(|road| road.kind);
    let mut mesh = ClippedMesh::default();
    for road in sorted {
        let start = road.points[0];
        let offset = Vec2::new(
            east_of(start.0, grid.centre.0, world_width) as f32,
            (start.1 - grid.centre.1) as f32,
        );
        let points = smoothed(road);
        let lift = ROAD_LIFT + LIFT_PER_KIND * road.kind as u8 as f32;
        let half = width(road.kind) / 2.0;
        for pair in points.windows(2) {
            let (a, b) = (pair[0] + offset, pair[1] + offset);
            if !near(a) || !near(b) {
                continue;
            }
            let strip = Strip { grid, square, a, b };
            if crosses_river(a) || crosses_river(b) {
                let colour = bridge_colour(road.kind);
                let half = half + BRIDGE_EXTRA;
                strip.add(&mut mesh, (-half, half), RIVER_LIFT + BRIDGE_LIFT, colour);
                continue;
            }
            let (ground, snowy) = strip.ground();
            let mut colour = match road.kind {
                RoadKind::Path => (ground * PATH_WORN).min(Vec4::ONE),
                kind => kind_colour(kind),
            };
            if snowy {
                colour = ground.lerp(colour, SHOWS_THROUGH_SNOW);
            }
            colour.w = 1.0;
            if road.kind == RoadKind::Highway {
                let mut edge = ground.lerp(rgb(0.0, 0.0, 0.3), 0.8);
                edge.w = 1.0;
                // Side by side rather than one over the other, which flickers.
                let middle = half * HIGHWAY_MIDDLE;
                strip.add(&mut mesh, (-half, -middle), lift, edge);
                strip.add(&mut mesh, (-middle, middle), lift, colour);
                strip.add(&mut mesh, (middle, half), lift, edge);
            } else {
                strip.add(&mut mesh, (-half, half), lift, colour);
            }
        }
    }
    mesh.into_mesh()
}

/// One stretch of road, from `a` to `b` (relative to the grid's centre).
struct Strip<'a> {
    grid: &'a TerrainGrid,
    square: ViewSquare,
    a: Vec2,
    b: Vec2,
}

impl Strip<'_> {
    /// The land's colour at its start, and whether it is snow there.
    fn ground(&self) -> (Vec4, bool) {
        let slot = self
            .grid
            .slot(self.a.x.round() as i64, self.a.y.round() as i64);
        (Vec4::from(self.grid.colours[slot]), self.grid.snowy[slot])
    }

    /// Adds the part of it from `left` to `right` across its middle (in grid
    /// units, left negative) to `mesh`, `lift` above the drawn ground.
    fn add(&self, mesh: &mut ClippedMesh, (left, right): (f32, f32), lift: f32, colour: Vec4) {
        let side = (self.b - self.a).perp().normalize_or_zero();
        let corner = |point: Vec2, across: f32| {
            let at = point + side * across;
            Corner {
                position: Vec3::new(at.x, self.grid.height_at(&self.grid.shown, at) + lift, at.y),
                normal: Vec3::Y,
                colour,
                cover: Vec4::ZERO,
                inside: self.square.inside(at),
            }
        };
        let (a0, a1) = (corner(self.a, left), corner(self.a, right));
        let (b0, b1) = (corner(self.b, left), corner(self.b, right));
        mesh.add([a0, a1, b1]);
        mesh.add([a0, b1, b0]);
    }
}

/// The grid points (relative to the centre) rivers near the window run through.
fn river_points(
    grid: &TerrainGrid,
    rivers: &KnownRivers,
    world_width: i64,
    near: impl Fn(Vec2) -> bool,
) -> BTreeSet<(i64, i64)> {
    let mut wet = BTreeSet::new();
    for river in rivers.rivers() {
        let east = east_of(river.source.0, grid.centre.0, world_width);
        let offset = Vec2::new(east as f32, (river.source.1 - grid.centre.1) as f32);
        for &point in &river.points {
            let at = point + offset;
            if near(at) {
                wet.insert((at.x.round() as i64, at.y.round() as i64));
            }
        }
    }
    wet
}

/// Tracks are dirt, roads grey stone and highways pale paving.
fn kind_colour(kind: RoadKind) -> Vec4 {
    match kind {
        RoadKind::Path | RoadKind::Track => rgb(30.0, 0.5, 0.42),
        RoadKind::Road => rgb(0.0, 0.0, 0.62),
        RoadKind::Highway => rgb(40.0, 0.15, 0.8),
    }
}

/// Planks for the small roads, stone for the large.
fn bridge_colour(kind: RoadKind) -> Vec4 {
    match kind {
        RoadKind::Path | RoadKind::Track => rgb(25.0, 0.5, 0.22),
        RoadKind::Road | RoadKind::Highway => rgb(0.0, 0.0, 0.65),
    }
}

/// A colour by hue, saturation and lightness, as the shader takes it.
fn rgb(hue: f64, saturation: f64, lightness: f64) -> Vec4 {
    Vec4::from(linear(hsl_to_rgba(hue, saturation, lightness)))
}
