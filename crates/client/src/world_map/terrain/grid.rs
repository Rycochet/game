//! What to draw at each grid point, and heights on the drawn triangles.

use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use shared::climate::Climate;
use shared::surface::Surface;
use shared::vegetation::Vegetation;

use super::super::rivers::KnownRivers;
use super::super::{HEIGHT_SCALE, SLOPE_SHADING};
use super::{
    CANOPY_SHADING, CHUNK, GridArea, HeightCache, ICE_BOOST, STORE_SIDE, canopy_lift, linear,
};
use crate::surface_style::{SurfaceKind, SurfaceStyle, hsl_to_rgba};

/// What to draw at each grid point near the player: drawn height, ground
/// height, normal and colour. Like `HeightCache`, it is a wrapping 2D grid
/// keyed by world position, so a step only works out the new edge.
/// Everything is cleared when the hour, weather or style change.
#[derive(Resource)]
pub struct TerrainGrid {
    keys: Vec<Option<(i64, i64)>>,
    /// Drawn height: the water surface where underwater.
    pub(super) shown: Vec<f32>,
    /// Ground height (below the water where underwater).
    pub(super) ground: Vec<f32>,
    pub(super) normals: Vec<[f32; 3]>,
    pub(super) colours: Vec<[f32; 4]>,
    /// What the shader grows here: trees, low growth, how narrow the trees
    /// are and how lush. No trees or growth except on open land.
    pub(super) covers: Vec<Vec4>,
    /// Whether it shows snow, which covers small roads.
    pub(super) snowy: Vec<bool>,
    /// The grid point the window is centred on (the player's).
    pub(super) centre: (i64, i64),
    /// Terrain chunks (`CHUNK` points a side, by world position) with a point
    /// worked out since `take_changed`: their meshes are out of date.
    changed: HashSet<(i64, i64)>,
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
            snowy: vec![false; len],
            centre: (0, 0),
            changed: HashSet::new(),
        }
    }
}

impl TerrainGrid {
    /// The lowest drawn height of the points worked out (the water where there
    /// is any), which the window's frame hangs from.
    pub(super) fn lowest_shown(&self) -> f32 {
        self.keys
            .iter()
            .zip(&self.shown)
            .filter(|(key, _)| key.is_some())
            .map(|(_, &shown)| shown)
            .reduce(f32::min)
            .unwrap_or(0.0)
    }

    /// Where the point at `(dx, dz)` from the centre is kept.
    pub(super) fn slot(&self, dx: i64, dz: i64) -> usize {
        let (x, z) = (self.centre.0 + dx, self.centre.1 + dz);
        (z.rem_euclid(STORE_SIDE) * STORE_SIDE + x.rem_euclid(STORE_SIDE)) as usize
    }

    /// Whether a terrain chunk's corners are all worked out (and so, as whole
    /// chunks are filled, all of it).
    pub fn has_chunk(&self, (cx, cz): (i64, i64)) -> bool {
        let (x, z) = (cx * CHUNK, cz * CHUNK);
        [
            (x, z),
            (x + CHUNK, z),
            (x, z + CHUNK),
            (x + CHUNK, z + CHUNK),
        ]
        .iter()
        .all(|&(x, z)| self.keys[self.slot(x - self.centre.0, z - self.centre.1)] == Some((x, z)))
    }

    /// The terrain chunks with points worked out since this was last asked.
    pub fn take_changed(&mut self) -> HashSet<(i64, i64)> {
        std::mem::take(&mut self.changed)
    }

    /// Makes sure every point in `areas` (world positions, both corners
    /// included, within `REACH + CHUNK` of `centre`) is worked out.
    /// `redraw_all` throws away what is kept first (for a new hour or style).
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
        floor: f64,
        light_from: (f64, f64),
        centre: (i64, i64),
        areas: &[GridArea],
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
                (x, z),
            )
        };
        let shown_at = |cache: &mut HeightCache, x: i64, z: i64| {
            style.shown_height(cache.get(surface, x, z), water_level) as f32 * HEIGHT_SCALE
                + lift_at(cache, x, z)
        };

        for &((from_x, from_z), (to_x, to_z)) in areas {
            for z in from_z..=to_z {
                for x in from_x..=to_x {
                    let i = self.slot(x - centre.0, z - centre.1);
                    if self.keys[i] == Some((x, z)) {
                        continue;
                    }
                    self.keys[i] = Some((x, z));
                    self.changed.extend(chunks_touching(x, z));
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
                        style.shade(sample, place, away, water_level, floor, SLOPE_SHADING);
                    let mut colour = linear(hsl_to_rgba(band.hue, band.saturation, lightness));
                    if style.is_frozen_water(sample, place, water_level) {
                        // Brighter than white, so after the sun dims this flat
                        // surface it shows at the style's ice colour.
                        for channel in &mut colour[..3] {
                            *channel *= ICE_BOOST;
                        }
                    }
                    // Alpha below 1 tells the shader this is moving water.
                    if sample.height <= water_level
                        && !style.is_frozen_water(sample, place, water_level)
                    {
                        colour[3] = 0.0;
                    }
                    self.colours[i] = colour;

                    // Snow, shore and water change with the weather and tide, so
                    // what shows is decided here rather than in `shared`.
                    let cover = cache.cover(vegetation, climate, surface, x, z);
                    let kind = style.kind(sample, place, water_level);
                    let open_land = kind == SurfaceKind::Land;
                    self.snowy[i] = kind == SurfaceKind::Snow;
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
    }

    /// The height of the drawn triangles at a point (relative to the centre),
    /// reading `values` (`shown` or `ground`). Each grid square is split into
    /// two triangles along the diagonal from its north-east to its south-west
    /// corner, as `build_chunk` draws it.
    pub(super) fn height_at(&self, values: &[f32], point: Vec2) -> f32 {
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
}

/// The terrain chunks a point is a corner of: one, or on a chunk's edge two or
/// four, as neighbouring chunks share their edge points.
fn chunks_touching(x: i64, z: i64) -> impl Iterator<Item = (i64, i64)> {
    let along = |at: i64| {
        let chunk = at.div_euclid(CHUNK);
        let before = (at.rem_euclid(CHUNK) == 0).then_some(chunk - 1);
        std::iter::once(chunk).chain(before)
    };
    along(x).flat_map(move |cx| along(z).map(move |cz| (cx, cz)))
}
