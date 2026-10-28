//! The terrain around the player, drawn as a square window lined up with the
//! camera, with flat faces down its edges.
//!
//! Two stages, so stepping and turning the view stay cheap:
//! - `TerrainGrid::fill` works out heights, normals and colours for the grid
//!   points the window covers. Points already worked out are kept, so a step
//!   or a turn only adds the newly covered ones.
//! - `build_window` and `build_edges` then cut the window out of that grid:
//!   whole triangles inside it are kept, ones crossing its border are clipped
//!   to it, so its outline is exact.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use shared::surface::{Surface, SurfaceSample};

use super::{HALF_SIZE, HEIGHT_SCALE, SLOPE_SHADING};
use crate::surface_style::{SurfaceStyle, hsl_to_rgba};

/// The furthest grid units from the player `TerrainGrid` may need. The
/// window's corners are `HALF_SIZE × √2` (about 283) away, so this covers it
/// at any angle.
const REACH: i64 = 284;

/// Side length of `TerrainGrid`'s wrapping store: more than the `2 × REACH`
/// it must hold, so moving never evicts points still in view.
const STORE_SIDE: i64 = 640;

/// Bottom of the edge faces, as a fraction of the water level (0 = height 0,
/// as in the prototype; raised to make the faces shorter).
const FRONT_BASE: f32 = 0.3;

/// Undoes the sun's dimming of flat frozen water (lit flat ice measured at
/// ~71% of its colour in linear light).
const ICE_BOOST: f32 = 1.41;

/// Cache side length: larger than `TerrainGrid`'s reach (plus a 1-unit
/// border for normals) so moving never evicts samples still in view.
const CACHE_SIDE: i64 = 640;

// Checked when compiling: the wrapping stores must be big enough.
const _: () = assert!(STORE_SIDE > 2 * REACH + 1);
const _: () = assert!(CACHE_SIDE > 2 * REACH + 3);

/// Surface samples by grid position, stored in a wrapping 2D grid. Moving one
/// step only samples the new edge; everything else is reused.
#[derive(Resource)]
pub struct HeightCache {
    keys: Vec<Option<(i64, i64)>>,
    samples: Vec<SurfaceSample>,
}

impl Default for HeightCache {
    fn default() -> Self {
        let len = (CACHE_SIDE * CACHE_SIDE) as usize;
        Self {
            keys: vec![None; len],
            samples: vec![
                SurfaceSample {
                    height: 0.0,
                    jitter: 0.0
                };
                len
            ],
        }
    }
}

impl HeightCache {
    pub fn get(&mut self, surface: &Surface, x: i64, y: i64) -> SurfaceSample {
        let index = (y.rem_euclid(CACHE_SIDE) * CACHE_SIDE + x.rem_euclid(CACHE_SIDE)) as usize;
        if self.keys[index] != Some((x, y)) {
            self.keys[index] = Some((x, y));
            self.samples[index] = surface.sample(x, y);
        }
        self.samples[index]
    }
}

/// What to draw at each grid point near the player: drawn height, ground
/// height, normal and colour. Like `HeightCache`, it is a wrapping 2D grid
/// keyed by world position, so a step only works out the new edge.
/// Everything is cleared when the hour, weather or style change.
#[derive(Resource)]
pub struct TerrainGrid {
    keys: Vec<Option<(i64, i64)>>,
    /// Drawn height: the water surface where underwater.
    shown: Vec<f32>,
    /// Ground height (below the water where underwater).
    ground: Vec<f32>,
    normals: Vec<[f32; 3]>,
    colours: Vec<[f32; 4]>,
    /// The grid point the window is centred on (the player's).
    centre: (i64, i64),
}

impl Default for TerrainGrid {
    fn default() -> Self {
        let len = (STORE_SIDE * STORE_SIDE) as usize;
        Self {
            keys: vec![None; len],
            shown: vec![0.0; len],
            ground: vec![0.0; len],
            normals: vec![[0.0, 1.0, 0.0]; len],
            colours: vec![[0.0; 4]; len],
            centre: (0, 0),
        }
    }
}

impl TerrainGrid {
    /// Where the point at `(dx, dz)` from the centre is kept.
    fn slot(&self, dx: i64, dz: i64) -> usize {
        let (x, z) = (self.centre.0 + dx, self.centre.1 + dz);
        (z.rem_euclid(STORE_SIDE) * STORE_SIDE + x.rem_euclid(STORE_SIDE)) as usize
    }

    /// Makes sure every point the window around `centre`, lined up with
    /// `axes`, needs is worked out. `redraw_all` throws away what is kept
    /// first (for a new hour, weather or style).
    #[allow(clippy::too_many_arguments)]
    pub fn fill(
        &mut self,
        cache: &mut HeightCache,
        surface: &Surface,
        style: &SurfaceStyle,
        water_level: f64,
        snow_line: f64,
        centre: (i64, i64),
        axes: (Vec2, Vec2),
        redraw_all: bool,
    ) {
        if redraw_all {
            self.keys.fill(None);
        }
        self.centre = centre;
        // Water is part of the terrain: underwater points sit at the surface.
        // (The cache is passed in rather than captured, so the loop below can
        // use it too: Rust allows only one mutable borrow at a time.)
        let shown_at = |cache: &mut HeightCache, x: i64, z: i64| {
            style.shown_height(cache.get(surface, x, z), water_level) as f32 * HEIGHT_SCALE
        };

        let (reach_x, reach_z) = window_reach(axes);
        for dz in -reach_z..=reach_z {
            for dx in -reach_x..=reach_x {
                let i = self.slot(dx, dz);
                let (x, z) = (centre.0 + dx, centre.1 + dz);
                if self.keys[i] == Some((x, z)) {
                    continue;
                }
                self.keys[i] = Some((x, z));
                self.shown[i] = shown_at(cache, x, z);

                // Slope from the neighbours either side (central differences).
                let slope_x = shown_at(cache, x + 1, z) - shown_at(cache, x - 1, z);
                let slope_z = shown_at(cache, x, z + 1) - shown_at(cache, x, z - 1);
                self.normals[i] = Vec3::new(-slope_x, 2.0, -slope_z).normalize().to_array();

                let sample = cache.get(surface, x, z);
                self.ground[i] = sample.height as f32 * HEIGHT_SCALE;
                // The prototype's strong slope shading, baked in per point
                // because real lighting alone is much smoother.
                let west = cache.get(surface, x - 1, z).height;
                let (band, lightness) =
                    style.shade(sample, west, water_level, snow_line, SLOPE_SHADING);
                let mut colour = linear(hsl_to_rgba(band.hue, band.saturation, lightness));
                if style.is_frozen_water(sample, water_level, snow_line) {
                    // Brighter than white, so after the sun dims this flat
                    // surface it shows at the style's ice colour.
                    for channel in &mut colour[..3] {
                        *channel *= ICE_BOOST;
                    }
                }
                self.colours[i] = colour;
            }
        }
    }

    /// The height of the drawn triangles at a point (relative to the centre),
    /// reading `values` (`shown` or `ground`). Each grid square is split into
    /// two triangles along the diagonal from its north-east to its south-west
    /// corner, as `build_window` draws it.
    fn height_at(&self, values: &[f32], point: Vec2) -> f32 {
        let (x, z) = (point.x.floor(), point.y.floor());
        let (fx, fz) = (point.x - x, point.y - z);
        let (x, z) = (x as i64, z as i64);
        let north_west = values[self.slot(x, z)];
        let north_east = values[self.slot(x + 1, z)];
        let south_west = values[self.slot(x, z + 1)];
        let south_east = values[self.slot(x + 1, z + 1)];
        if fx + fz <= 1.0 {
            north_west + fx * (north_east - north_west) + fz * (south_west - north_west)
        } else {
            south_east
                + (1.0 - fx) * (south_west - south_east)
                + (1.0 - fz) * (north_east - south_east)
        }
    }

    /// The point at `(dx, dz)` from the centre, ready for clipping.
    fn corner(&self, dx: i64, dz: i64, axes: (Vec2, Vec2)) -> Corner {
        let i = self.slot(dx, dz);
        Corner {
            position: Vec3::new(dx as f32, self.shown[i], dz as f32),
            normal: Vec3::from(self.normals[i]),
            colour: Vec4::from(self.colours[i]),
            inside: inside_window(Vec2::new(dx as f32, dz as f32), axes),
        }
    }
}

/// A corner of a triangle being clipped to the window.
#[derive(Clone, Copy)]
struct Corner {
    position: Vec3,
    normal: Vec3,
    colour: Vec4,
    /// Distance inside each of the window's four sides (negative outside).
    inside: [f32; 4],
}

impl Corner {
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            normal: self.normal.lerp(other.normal, t),
            colour: self.colour.lerp(other.colour, t),
            inside: std::array::from_fn(|k| {
                self.inside[k] + (other.inside[k] - self.inside[k]) * t
            }),
        }
    }
}

/// How far `point` (relative to the centre) is inside each side of the
/// window: right, left, near and far. `axes` are the camera's right and
/// towards-the-camera directions along the ground.
fn inside_window(point: Vec2, (right, towards): (Vec2, Vec2)) -> [f32; 4] {
    let half = HALF_SIZE as f32;
    let (u, v) = (point.dot(right), point.dot(towards));
    [half - u, half + u, half - v, half + v]
}

/// How far the window lined up with `axes` reaches from the centre along x
/// and z (its bounding box), plus one for the squares around its edge. Only
/// grid points within this are drawn.
fn window_reach((right, towards): (Vec2, Vec2)) -> (i64, i64) {
    let half = HALF_SIZE as f32;
    let reach = |a: f32, b: f32| ((half * (a.abs() + b.abs())).ceil() as i64 + 1).min(REACH);
    (reach(right.x, towards.x), reach(right.y, towards.y))
}

/// The terrain mesh: the `HALF_SIZE` square window lined up with `axes`
/// (see `inside_window`), cut out of the grid.
pub fn build_window(grid: &TerrainGrid, axes: (Vec2, Vec2)) -> Mesh {
    let (right, towards) = axes;
    let half = HALF_SIZE as f32;
    let (reach_x, reach_z) = window_reach(axes);
    let width = (2 * reach_x + 1) as usize;
    let local = |dx: i64, dz: i64| (dz + reach_z) as usize * width + (dx + reach_x) as usize;

    // Which points in the box are inside the window.
    let mut inside = Vec::with_capacity(width * (2 * reach_z + 1) as usize);
    for dz in -reach_z..=reach_z {
        for dx in -reach_x..=reach_x {
            let point = Vec2::new(dx as f32, dz as f32);
            inside.push(inside_window(point, axes).iter().all(|&d| d >= 0.0));
        }
    }
    // Squares near the window's corners need clipping even when none of
    // their points are inside (a corner can poke into a triangle).
    let corners = [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)]
        .map(|(u, v)| right * u * half + towards * v * half);
    let near_corner = |x: i64, z: i64| {
        corners
            .iter()
            .any(|c| (c.x - x as f32).abs() <= 2.0 && (c.y - z as f32).abs() <= 2.0)
    };

    let expected = (4 * HALF_SIZE * HALF_SIZE) as usize;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(expected);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(expected);
    let mut colours: Vec<[f32; 4]> = Vec::with_capacity(expected);
    let mut indices: Vec<u32> = Vec::with_capacity(expected * 6);
    // Points already added to the mesh (by box position), so whole
    // triangles share them.
    let mut added = vec![u32::MAX; inside.len()];

    for z in -reach_z..reach_z {
        for x in -reach_x..reach_x {
            // Two triangles per square, wound anticlockwise seen from above.
            for triangle in [
                [(x, z), (x, z + 1), (x + 1, z)],
                [(x + 1, z), (x, z + 1), (x + 1, z + 1)],
            ] {
                let count = triangle
                    .iter()
                    .filter(|&&(dx, dz)| inside[local(dx, dz)])
                    .count();
                if count == 3 {
                    for (dx, dz) in triangle {
                        let j = local(dx, dz);
                        if added[j] == u32::MAX {
                            let i = grid.slot(dx, dz);
                            positions.push([dx as f32, grid.shown[i], dz as f32]);
                            normals.push(grid.normals[i]);
                            colours.push(grid.colours[i]);
                            added[j] = positions.len() as u32 - 1;
                        }
                        indices.push(added[j]);
                    }
                    continue;
                }
                if count == 0 && !near_corner(x, z) {
                    continue;
                }
                // Crosses the border: cut off the parts outside each side in
                // turn, then fan the remaining polygon into triangles.
                let clipped = clip(&triangle.map(|(dx, dz)| grid.corner(dx, dz, axes)));
                let first = positions.len() as u32;
                for c in &clipped {
                    positions.push(c.position.to_array());
                    normals.push(c.normal.normalize().to_array());
                    colours.push(c.colour.to_array());
                }
                for k in 1..clipped.len().saturating_sub(1) as u32 {
                    indices.extend_from_slice(&[first, first + k, first + k + 1]);
                }
            }
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(indices))
}

/// Cuts a polygon down to the part inside all four sides of the window
/// (Sutherland–Hodgman clipping), keeping its winding.
fn clip(polygon: &[Corner]) -> Vec<Corner> {
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

/// Flat faces down the window's four edges (one mesh): water depth (where
/// underwater) above earth, down to `FRONT_BASE` × the water level. They face
/// outwards, so only the sides towards the camera are drawn. The top of each
/// face follows the terrain exactly, at every point where the edge crosses a
/// triangle.
#[allow(clippy::too_many_arguments)]
pub fn build_edges(
    grid: &TerrainGrid,
    cache: &mut HeightCache,
    surface: &Surface,
    style: &SurfaceStyle,
    water_level: f64,
    snow_line: f64,
    (cx, cy): (i64, i64),
    (right, towards): (Vec2, Vec2),
) -> Mesh {
    let water_top = water_level as f32 * HEIGHT_SCALE;
    let base = water_top * FRONT_BASE;
    let (h, s, l) = style.earth_face;
    let earth_colour = linear(hsl_to_rgba(h, s, l));
    let half = HALF_SIZE as f32;
    let corner = |u: f32, v: f32| right * u + towards * v;
    // Each edge from one corner to the next, walked so its face points
    // outwards: near, far, right, left.
    let edges = [
        (corner(-half, half), corner(half, half)),
        (corner(half, -half), corner(-half, -half)),
        (corner(half, half), corner(half, -half)),
        (corner(-half, -half), corner(-half, half)),
    ];

    let mut positions = Vec::new();
    let mut colours = Vec::new();
    let mut indices = Vec::new();
    for (start, end) in edges {
        let first = positions.len() as u32;
        let stops = crossings(start, end);
        for &t in &stops {
            let point = start.lerp(end, t);
            let ground = grid.height_at(&grid.ground, point).max(base);
            let top = grid.height_at(&grid.shown, point).max(ground);
            // Half the surface lightness of the nearest point, like the
            // prototype's front slice (this also ices over with the water).
            let (x, z) = (cx + point.x.round() as i64, cy + point.y.round() as i64);
            let sample = cache.get(surface, x, z);
            let west = cache.get(surface, x - 1, z).height;
            let (band, lightness) = style.shade(sample, west, water_level, snow_line, 0.0);
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
fn crossings(start: Vec2, end: Vec2) -> Vec<f32> {
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

/// The highest drawn surface within `radius` of a fractional position, so
/// something that wide can rest on top without sinking into a slope.
pub fn highest_shown_height(
    cache: &mut HeightCache,
    surface: &Surface,
    style: &SurfaceStyle,
    water_level: f64,
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
                highest = highest.max(style.shown_height(sample, water_level));
            }
        }
    }
    highest as f32 * HEIGHT_SCALE
}

/// sRGB bytes to the linear floats meshes use.
fn linear([r, g, b, _]: [u8; 4]) -> [f32; 4] {
    Color::srgb_u8(r, g, b).to_linear().to_f32_array()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_reaches_the_window_corners() {
        let corner = HALF_SIZE as f32 * std::f32::consts::SQRT_2;
        // The corner plus one square for the triangles around it.
        assert!(REACH as f32 >= corner + 1.0);
    }

    #[test]
    fn crossings_include_ends_and_grid_lines() {
        let stops = crossings(Vec2::new(0.5, 0.5), Vec2::new(2.5, 0.5));
        // x = 1, x = 2 and the diagonals x + z = 1, 2, 3 (z is 0.5): the
        // diagonals fall at x = 0.5 (the start), 1.5 and 2.5 (the end).
        assert_eq!(stops, [0.0, 0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn clipping_keeps_the_inside_part() {
        let axes = (Vec2::X, Vec2::Y);
        let half = HALF_SIZE as f32;
        let at = |x: f32, z: f32| Corner {
            position: Vec3::new(x, 0.0, z),
            normal: Vec3::Y,
            colour: Vec4::ONE,
            inside: inside_window(Vec2::new(x, z), axes),
        };
        // A triangle sticking out of the right side by half its width.
        let clipped = clip(&[
            at(half - 1.0, 0.0),
            at(half - 1.0, 1.0),
            at(half + 1.0, 0.0),
        ]);
        assert!(clipped.len() >= 3);
        assert!(clipped.iter().all(|c| c.position.x <= half + 1e-4));
    }
}
