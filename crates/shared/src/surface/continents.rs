//! Continents: land and sea from noise on the scale of the whole planet,
//! worked out in whole metres from the sea. Each world has its own share of
//! land, met exactly by choosing where the coast falls. With plates (the
//! `Plates` generator), they decide where land is and where ranges rise, and
//! the noise adds the detail.

mod layers;
mod plates;
mod shape;

use fastnoise_lite::FastNoiseLite;

use super::{cylinder, new_noise};
use crate::world::WorldParams;
use layers::{Point, layered, ridged, scaled, warped};
use plates::{Plates, Tectonics};
use shape::smoothstep;

/// The largest features: about this many across the planet's width.
const FEATURES_AROUND: f64 = 4.0;
/// Layers are added until their features are this small, in grid units; for
/// the broad height (and so the planet's picture) only down to the second.
const SMALLEST: f64 = 2.0;
const BROAD_SMALLEST: f64 = 32.0;
/// The most layers of any one noise.
const MOST_LAYERS: u32 = 16;
/// The layer near this size, in grid units, also gives `jitter`.
const JITTER_SIZE: f64 = 8.0;
/// How far coasts are pushed about, in noise units.
const WARP: f64 = 0.35;
/// The share of the planet that is land: from this to that, by the seed.
const LAND_SHARE: (f64, f64) = (0.2, 0.45);
/// How many points across and down the planet are tried to find where the
/// coast falls for that share.
const SHARE_SAMPLES: (u32, u32) = (64, 48);
/// Ranges run along the lines where noise this many times finer than the
/// continents' crosses 0: belts from `BELT_FROM` of the way to the line, at full
/// strength by `BELT_FULL`. Only some lines have them: where other noise, this
/// many times finer, is above `RANGES_FROM` (full by `RANGES_FULL`).
const BELTS_SCALE: f64 = 2.5;
const BELT_FROM: f64 = 0.75;
const BELT_FULL: f64 = 0.95;
const RANGES_SCALE: f64 = 1.5;
const RANGES_FROM: f64 = -0.1;
const RANGES_FULL: f64 = 0.25;
/// Plains and plateaus: noise this many times finer than the continents',
/// raising the land from `LOWEST_UPLIFT` of the most to all of it.
const UPLIFT_SCALE: f64 = 3.0;
const LOWEST_UPLIFT: f64 = 0.15;
/// The ridges of ranges, this many times finer than the continents.
const RIDGES_SCALE: f64 = 8.0;
/// With plates: how much a plate's kind (`-1..1`) moves the land noise, and
/// a range raises it, so ranges and island arcs stand out of the sea.
const PLATE_LAND: f64 = 0.45;
const RANGE_LAND: f64 = 0.15;

pub(super) struct Continents {
    land: FastNoiseLite,
    warp: FastNoiseLite,
    ranges: FastNoiseLite,
    ridges: FastNoiseLite,
    uplift: FastNoiseLite,
    /// For the `Plates` generator.
    plates: Option<Plates>,
    width: i64,
    /// Grid units to a unit of noise space.
    zoom: f64,
    /// Where the coast is on the land noise.
    coast: f64,
}

impl Continents {
    /// `with_plates`: the `Plates` generator rather than `Continents`.
    pub(super) fn new(world: &WorldParams, width: i64, with_plates: bool) -> Self {
        let mut continents = Self {
            land: new_noise(world, "continents"),
            warp: new_noise(world, "continents warp"),
            ranges: new_noise(world, "continents ranges"),
            ridges: new_noise(world, "continents ridges"),
            uplift: new_noise(world, "continents uplift"),
            plates: with_plates.then(|| Plates::new(world, width)),
            width,
            zoom: (width as f64 / FEATURES_AROUND).max(1.0),
            coast: 0.0,
        };
        let (least, most) = LAND_SHARE;
        let share = world.seed.derive("land share").rng().range_f64(least, most);
        continents.coast = continents.coast_for(share);
        continents
    }

    /// Metres from the sea, and the jitter (`-8..8`, or 0 for broad). Broad
    /// leaves out the layers smaller than `BROAD_SMALLEST`.
    pub(super) fn metres(&self, x: i64, y: i64, broad: bool) -> (i32, f64) {
        let smallest = if broad { BROAD_SMALLEST } else { SMALLEST };
        let point = warped(&self.warp, cylinder(x, y, self.width, self.zoom), WARP);
        let tectonics = self.plates.as_ref().map(|plates| plates.at((x, y), point));
        let (land, jitter) = self.land_at(point, smallest, tectonics);
        let land = land - self.coast;
        let (ridge, uplift) = if land > 0.0 {
            (
                self.ridge_at(point, smallest, tectonics),
                self.uplift_at(point),
            )
        } else {
            (0.0, 0.0)
        };
        let moved = tectonics.map_or(0.0, |found| found.boundary.metres);
        let metres = (shape::metres(land, ridge, uplift) + moved).round() as i32;
        let jitter = if broad { 0.0 } else { jitter * 8.0 };
        (metres, jitter)
    }

    /// Which plate a point is on, if this has plates.
    pub(super) fn plate_at(&self, x: i64, y: i64) -> Option<crate::surface::PlateSample> {
        let plates = self.plates.as_ref()?;
        let point = warped(&self.warp, cylinder(x, y, self.width, self.zoom), WARP);
        Some(plates.sample((x, y), point))
    }

    /// How many layers until features are `smallest` grid units, starting
    /// from features `zoom / scale` across.
    fn layers(&self, scale: f64, smallest: f64) -> u32 {
        let mut layers = 1;
        while layers < MOST_LAYERS && self.zoom / scale / (1u64 << layers) as f64 >= smallest {
            layers += 1;
        }
        layers
    }

    /// The land at a warped point, before the coast is taken off (the noise,
    /// moved by the plates if there are any), and the jitter layer.
    fn land_at(&self, point: Point, smallest: f64, tectonics: Option<Tectonics>) -> (f64, f64) {
        let jitter_layer = libm::round(libm::log2(self.zoom / JITTER_SIZE)).max(0.0) as u32;
        let (noise, jitter) = layered(&self.land, point, self.layers(1.0, smallest), jitter_layer);
        let moved = tectonics.map_or(0.0, |found| {
            PLATE_LAND * found.base + RANGE_LAND * found.boundary.range
        });
        (noise + moved, jitter)
    }

    /// How strong a mountain range is here, `0..=1`: where plates meet if
    /// there are any, otherwise long winding belts.
    fn ridge_at(&self, point: Point, smallest: f64, tectonics: Option<Tectonics>) -> f64 {
        let layers = self.layers(RIDGES_SCALE, smallest).min(8);
        if let Some(found) = tectonics {
            let range = found.boundary.range;
            if range <= 0.0 {
                return 0.0;
            }
            // A solid range with ridges along it, rather than ridges alone.
            let ridges = ridged(&self.ridges, scaled(point, RIDGES_SCALE), layers);
            return range * (0.5 + 0.5 * ridges);
        }
        let line = 1.0
            - layered(&self.ranges, scaled(point, BELTS_SCALE), 2, u32::MAX)
                .0
                .abs();
        let belt = smoothstep(BELT_FROM, BELT_FULL, line);
        if belt <= 0.0 {
            return 0.0;
        }
        // The same noise far off, so it doesn't follow the belts.
        let elsewhere = scaled(
            (point.0 + 31.7, point.1 - 12.9, point.2 + 7.3),
            RANGES_SCALE,
        );
        let range = belt
            * smoothstep(
                RANGES_FROM,
                RANGES_FULL,
                layered(&self.ranges, elsewhere, 2, u32::MAX).0,
            );
        if range <= 0.0 {
            return 0.0;
        }
        range * ridged(&self.ridges, scaled(point, RIDGES_SCALE), layers)
    }

    /// How raised the land is here, `LOWEST_UPLIFT..=1`.
    fn uplift_at(&self, point: Point) -> f64 {
        let value = layered(&self.uplift, scaled(point, UPLIFT_SCALE), 3, u32::MAX).0;
        LOWEST_UPLIFT + (1.0 - LOWEST_UPLIFT) * smoothstep(-0.4, 0.4, value)
    }

    /// The land noise's value with `share` of the planet above it, from points
    /// spread evenly over the globe's area (rows closer together near the
    /// equator, where there is more of it).
    fn coast_for(&self, share: f64) -> f64 {
        let (across, down) = SHARE_SAMPLES;
        let mut values = Vec::with_capacity((across * down) as usize);
        for row in 0..down {
            let sine = -1.0 + (2.0 * row as f64 + 1.0) / down as f64;
            let latitude = libm::asin(sine) / core::f64::consts::FRAC_PI_2;
            let y = libm::round(latitude * self.width as f64 / 4.0) as i64;
            for column in 0..across {
                let x = self.width * column as i64 / across as i64;
                let point = warped(&self.warp, cylinder(x, y, self.width, self.zoom), WARP);
                let tectonics = self.plates.as_ref().map(|plates| plates.at((x, y), point));
                values.push(self.land_at(point, BROAD_SMALLEST, tectonics).0);
            }
        }
        values.sort_by(f64::total_cmp);
        let index = ((1.0 - share) * values.len() as f64) as usize;
        values[index.min(values.len() - 1)]
    }
}
