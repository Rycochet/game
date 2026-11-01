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
use shared::climate::{Climate, ClimateSample};
use shared::surface::{Surface, SurfaceSample};
use shared::vegetation::{Cover, Vegetation};

use super::rivers::KnownRivers;
use super::{HALF_SIZE, HEIGHT_SCALE, SLOPE_SHADING};
use crate::surface_style::{SurfaceKind, SurfaceStyle, hsl_to_rgba};

/// The furthest grid units from the player `TerrainGrid` may need. The
/// window's corners are `HALF_SIZE × √2` (about 283) away, so this covers it
/// at any angle, with a little to spare.
pub const REACH: i64 = 287;

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
    climate_keys: Vec<Option<(i64, i64)>>,
    climates: Vec<ClimateSample>,
    cover_keys: Vec<Option<(i64, i64)>>,
    covers: Vec<Cover>,
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
            climate_keys: vec![None; len],
            climates: vec![ClimateSample::default(); len],
            cover_keys: vec![None; len],
            covers: vec![Cover::default(); len],
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

    /// The climate at a point, kept so a new hour needn't work it out again.
    pub fn climate(
        &mut self,
        climate: &Climate,
        surface: &Surface,
        x: i64,
        y: i64,
    ) -> ClimateSample {
        let index = (y.rem_euclid(CACHE_SIDE) * CACHE_SIDE + x.rem_euclid(CACHE_SIDE)) as usize;
        if self.climate_keys[index] != Some((x, y)) {
            let height = self.get(surface, x, y).height;
            self.climate_keys[index] = Some((x, y));
            self.climates[index] = climate.sample(x, y, height);
        }
        self.climates[index]
    }

    /// What grows at a point, kept like the climate.
    pub fn cover(
        &mut self,
        vegetation: &Vegetation,
        climate: &Climate,
        surface: &Surface,
        x: i64,
        y: i64,
    ) -> Cover {
        let index = (y.rem_euclid(CACHE_SIDE) * CACHE_SIDE + x.rem_euclid(CACHE_SIDE)) as usize;
        if self.cover_keys[index] != Some((x, y)) {
            let place = self.climate(climate, surface, x, y);
            let height = self.get(surface, x, y).height;
            self.cover_keys[index] = Some((x, y));
            self.covers[index] = vegetation.cover(x, y, height, place);
        }
        self.covers[index]
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
    /// What the shader grows here: trees, low growth, how narrow the trees
    /// are and how lush. No trees or growth except on open land.
    covers: Vec<Vec4>,
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
            covers: vec![Vec4::ZERO; len],
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

    /// Makes sure every point the window `square` around `centre` needs is
    /// worked out. `redraw_all` throws away what is kept
    /// first (for a new hour, weather or style).
    #[allow(clippy::too_many_arguments)]
    pub fn fill(
        &mut self,
        cache: &mut HeightCache,
        surface: &Surface,
        climate: &Climate,
        vegetation: &Vegetation,
        rivers: &KnownRivers,
        style: &SurfaceStyle,
        water_level: f64,
        snow_line: f64,
        light_from: (f64, f64),
        centre: (i64, i64),
        square: ViewSquare,
        redraw_all: bool,
    ) {
        if redraw_all {
            self.keys.fill(None);
        }
        self.centre = centre;
        // Water is part of the terrain: underwater points sit at the surface.
        // (The cache is passed in rather than captured, so the loop below can
        // use it too: Rust allows only one mutable borrow at a time.)
        let lift_at = |cache: &mut HeightCache, x: i64, z: i64| {
            canopy_lift(
                cache,
                surface,
                climate,
                vegetation,
                rivers,
                style,
                water_level,
                snow_line,
                (x, z),
            )
        };
        let shown_at = |cache: &mut HeightCache, x: i64, z: i64| {
            style.shown_height(cache.get(surface, x, z), water_level) as f32 * HEIGHT_SCALE
                + lift_at(cache, x, z)
        };

        let (reach_x, reach_z) = square.reach();
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
                // Woods count as raised ground, so their sunlit edges show.
                // It is shaded by the height a step away from the light: a step
                // along each axis, in the amounts the light comes from that way.
                let (from_east, from_south) = light_from;
                let lift_here = lift_at(cache, x, z);
                let mut height_at = |x: i64, z: i64| {
                    cache.get(surface, x, z).height
                        + ((lift_at(cache, x, z) - lift_here) * CANOPY_SHADING) as f64
                };
                let across = height_at(x - from_east.signum() as i64, z) - sample.height;
                let along = height_at(x, z - from_south.signum() as i64) - sample.height;
                let away = sample.height + across * from_east.abs() + along * from_south.abs();
                let place = cache.climate(climate, surface, x, z);
                let (band, lightness) =
                    style.shade(sample, place, away, water_level, snow_line, SLOPE_SHADING);
                let mut colour = linear(hsl_to_rgba(band.hue, band.saturation, lightness));
                if style.is_frozen_water(sample, water_level, snow_line) {
                    // Brighter than white, so after the sun dims this flat
                    // surface it shows at the style's ice colour.
                    for channel in &mut colour[..3] {
                        *channel *= ICE_BOOST;
                    }
                }
                // Alpha below 1 tells the shader this is moving water.
                if sample.height <= water_level
                    && !style.is_frozen_water(sample, water_level, snow_line)
                {
                    colour[3] = 0.0;
                }
                self.colours[i] = colour;

                // Snow, shore and water change with the weather and tide, so
                // what shows is decided here rather than in `shared`.
                let cover = cache.cover(vegetation, climate, surface, x, z);
                let open_land =
                    style.kind(sample, place, water_level, snow_line) == SurfaceKind::Land;
                self.covers[i] = Vec4::new(
                    if open_land {
                        cover.trees * rivers.open_at(x, z)
                    } else {
                        0.0
                    },
                    if open_land { cover.growth } else { 0.0 },
                    cover.narrow,
                    cover.lush,
                );
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
    fn corner(&self, dx: i64, dz: i64, square: ViewSquare) -> Corner {
        let i = self.slot(dx, dz);
        Corner {
            position: Vec3::new(dx as f32, self.shown[i], dz as f32),
            normal: Vec3::from(self.normals[i]),
            colour: Vec4::from(self.colours[i]),
            cover: self.covers[i],
            inside: square.inside(Vec2::new(dx as f32, dz as f32)),
        }
    }
}

/// A corner of a triangle being clipped to the window.
#[derive(Clone, Copy)]
struct Corner {
    position: Vec3,
    normal: Vec3,
    colour: Vec4,
    cover: Vec4,
    /// Distance inside each of the window's four sides (negative outside).
    inside: [f32; 4],
}

impl Corner {
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            normal: self.normal.lerp(other.normal, t),
            colour: self.colour.lerp(other.colour, t),
            cover: self.cover.lerp(other.cover, t),
            inside: std::array::from_fn(|k| {
                self.inside[k] + (other.inside[k] - self.inside[k]) * t
            }),
        }
    }
}

/// The square window of terrain on show, centred on the player and lined up
/// with the camera, in grid units relative to the centre.
#[derive(Clone, Copy, Debug)]
pub struct ViewSquare {
    /// The camera's right and towards-the-camera directions along the ground,
    /// as (x, z).
    pub right: Vec2,
    pub towards: Vec2,
    /// From the centre to each side: `HALF_SIZE` in the starting view, less
    /// when zoomed in.
    pub half: f32,
}

impl ViewSquare {
    /// How far `point` is inside each side: right, left, near and far
    /// (negative outside).
    fn inside(&self, point: Vec2) -> [f32; 4] {
        let (u, v) = (point.dot(self.right), point.dot(self.towards));
        [self.half - u, self.half + u, self.half - v, self.half + v]
    }

    /// Whether `point` is on or inside the square.
    pub fn contains(&self, point: Vec2) -> bool {
        self.inside(point).iter().all(|&d| d >= 0.0)
    }

    /// The point `u` halves to the right and `v` halves towards the camera.
    fn at(&self, u: f32, v: f32) -> Vec2 {
        (self.right * u + self.towards * v) * self.half
    }

    /// How far the square reaches from the centre along x and z (its bounding
    /// box), plus a little for the grid squares around its edge and rivers
    /// crossing it. Only grid points within this are drawn.
    fn reach(&self) -> (i64, i64) {
        let reach =
            |a: f32, b: f32| ((self.half * (a.abs() + b.abs())).ceil() as i64 + 3).min(REACH);
        (
            reach(self.right.x, self.towards.x),
            reach(self.right.y, self.towards.y),
        )
    }

    /// How much of the starting view's width is on show (1 = all of it).
    fn zoom(&self) -> f32 {
        self.half / HALF_SIZE as f32
    }
}

/// The terrain mesh: the window `square`, cut out of the grid.
pub fn build_window(grid: &TerrainGrid, square: ViewSquare) -> Mesh {
    let (reach_x, reach_z) = square.reach();
    let width = (2 * reach_x + 1) as usize;
    let local = |dx: i64, dz: i64| (dz + reach_z) as usize * width + (dx + reach_x) as usize;

    // Which points in the box are inside the window.
    let mut inside = Vec::with_capacity(width * (2 * reach_z + 1) as usize);
    for dz in -reach_z..=reach_z {
        for dx in -reach_x..=reach_x {
            let point = Vec2::new(dx as f32, dz as f32);
            inside.push(square.contains(point));
        }
    }
    // Squares near the window's corners need clipping even when none of
    // their points are inside (a corner can poke into a triangle).
    let corners =
        [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)].map(|(u, v)| square.at(u, v));
    let near_corner = |x: i64, z: i64| {
        corners
            .iter()
            .any(|c| (c.x - x as f32).abs() <= 2.0 && (c.y - z as f32).abs() <= 2.0)
    };

    let expected = (4.0 * square.half * square.half) as usize;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(expected);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(expected);
    let mut colours: Vec<[f32; 4]> = Vec::with_capacity(expected);
    // Two pairs of numbers, in the mesh's texture coordinate slots, as those
    // reach the shader without a vertex shader of our own.
    let mut growing: Vec<[f32; 2]> = Vec::with_capacity(expected);
    let mut kinds: Vec<[f32; 2]> = Vec::with_capacity(expected);
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
                            growing.push(grid.covers[i].xy().to_array());
                            kinds.push(grid.covers[i].zw().to_array());
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
                let clipped = clip(&triangle.map(|(dx, dz)| grid.corner(dx, dz, square)));
                let first = positions.len() as u32;
                for c in &clipped {
                    positions.push(c.position.to_array());
                    normals.push(c.normal.normalize().to_array());
                    colours.push(c.colour.to_array());
                    growing.push(c.cover.xy().to_array());
                    kinds.push(c.cover.zw().to_array());
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
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, growing)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, kinds)
    .with_inserted_indices(Indices::U32(indices))
}

/// How far rivers are drawn above the ground, so it doesn't poke through them.
const RIVER_LIFT: f32 = 0.25;

/// The dark, fading edge each side of a river, which makes it look set into
/// the ground: its width in grid units, and its darkness next to the water.
const BANK_WIDTH: f32 = 0.6;
const BANK_ALPHA: f32 = 0.6;

/// A mesh being built out of triangles clipped to the window.
#[derive(Default)]
struct ClippedMesh {
    positions: Vec<[f32; 3]>,
    colours: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl ClippedMesh {
    fn add(&mut self, triangle: [Corner; 3]) {
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

    fn into_mesh(self) -> Mesh {
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
    snow_line: f64,
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
            let frozen_here =
                style.river_is_frozen(river.temperatures[i], ground, water_level, snow_line);
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
    style: &SurfaceStyle,
    water_level: f64,
    snow_line: f64,
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
            // Only the water strip uses this: nearly all water, which has no
            // biome colour, with a sliver of shore at most.
            let (band, lightness) = style.shade(
                sample,
                ClimateSample::default(),
                west,
                water_level,
                snow_line,
                0.0,
            );
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
const CANOPY_SHORE: f32 = 4.0;
/// How much of the canopy's height counts in the baked slope shading, which is
/// strong enough to turn woods' edges white at full height.
const CANOPY_SHADING: f32 = 0.2;

/// How high woods stand above the ground, plus how much more in rainforest.
const CANOPY_HEIGHT: f32 = 2.5;
const CANOPY_LUSH: f32 = 1.5;
/// How much tree cover it takes for the canopy to start rising, and to be full
/// height.
const CANOPY_FROM: f32 = 0.2;
const CANOPY_FULL: f32 = 0.7;

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
        let half = HALF_SIZE as f32;
        let square = ViewSquare {
            right: Vec2::X,
            towards: Vec2::Y,
            half,
        };
        let at = |x: f32, z: f32| Corner {
            position: Vec3::new(x, 0.0, z),
            normal: Vec3::Y,
            colour: Vec4::ONE,
            cover: Vec4::ZERO,
            inside: square.inside(Vec2::new(x, z)),
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
