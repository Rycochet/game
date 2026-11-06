//! The planet's surface height at any grid position.
//!
//! The surface is wrapped around a cylinder in 3D noise space, so travelling
//! east or west eventually returns to the start. North/south does not wrap.
//! One grid unit is about 1 km. There is more than one way of making it (see
//! `SurfaceGenerator`), chosen per world in `WorldParams::generator`.

mod continents;
mod desert;

use crate::climate::{ClimateSample, SEA_ICE};
use crate::showcase;
use crate::world::WorldParams;
use continents::Continents;
use core::f64::consts::TAU;
use desert::Desert;
use fastnoise_lite::{FastNoiseLite, NoiseType};

/// Metres for each unit of the height scale, measuring up from the sea.
pub const METRES_PER_UNIT: f64 = 100.0;

/// How deep the deepest seas a generator in metres makes are, on the height
/// scale below its sea level (8 km). Water is shaded by depth down to here.
const DEEPEST: f64 = 80.0;

/// Degrees Celsius a point's `jitter` moves where the sea freezes, so the
/// edge of the ice is ragged rather than following the smooth climate.
const ICE_JITTER: f64 = 0.25;

/// Height and texture at one grid position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceSample {
    /// Ground height, roughly `0.5..127.5`, centred on 64. Generators working
    /// in metres may go past either end.
    pub height: f64,
    /// Small-scale variation, `-8..8`, used to roughen the edges between
    /// terrain bands. Not part of the physical height.
    pub jitter: f64,
}

impl SurfaceSample {
    /// Whether this point is ice: under water, where the climate over it
    /// (`Climate::sample` at its height) is colder than `SEA_ICE`. Both
    /// drawing and walking go by this.
    pub fn is_frozen(&self, water_level: f64, climate: ClimateSample) -> bool {
        self.height <= water_level && climate.celsius() + self.jitter * ICE_JITTER < SEA_ICE
    }
}

/// Which plate a point is on, for showing them (`Plates` only).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlateSample {
    /// Which plate: its cell on the plates' coarse grid.
    pub plate: (i64, i64),
    /// Whether it carries a continent.
    pub continental: bool,
    /// How far from its edge with the next plate, in plate cells (one is
    /// about a tenth of the planet's width).
    pub edge: f64,
    /// How fast it and that plate close, `-2..2`: below 0 they part.
    pub closing: f64,
}

/// The ways of making a surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SurfaceGenerator {
    /// Layered noise on one scale: hills everywhere, nothing continent-sized.
    /// The old generator, kept for dry desert worlds.
    #[default]
    Desert,
    /// Continents and oceans with shelves, worked out in metres, with rare
    /// mountain ranges inland.
    Continents,
    /// As `Continents`, but where land is and where ranges rise come from
    /// drifting plates: ranges where they meet, island arcs, trenches, ocean
    /// ridges and rifts.
    Plates,
}

impl SurfaceGenerator {
    pub const ALL: [SurfaceGenerator; 3] = [
        SurfaceGenerator::Desert,
        SurfaceGenerator::Continents,
        SurfaceGenerator::Plates,
    ];

    pub fn name(self) -> &'static str {
        match self {
            SurfaceGenerator::Desert => "Desert",
            SurfaceGenerator::Continents => "Continents",
            SurfaceGenerator::Plates => "Plates",
        }
    }

    /// The generator with this name (any case), as `name` gives it.
    pub fn named(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.name().eq_ignore_ascii_case(name))
    }

    /// The generator every world uses when `HOW_DEBUG_GENERATOR` names one, for
    /// trying one out in the game. A client and its server must both have it
    /// (as the native client's own server does), or they disagree.
    pub fn forced() -> Option<Self> {
        Self::named(&std::env::var("HOW_DEBUG_GENERATOR").ok()?)
    }

    /// Whether it works in metres, with a climate from the planet.
    pub fn in_metres(self) -> bool {
        self != SurfaceGenerator::Desert
    }
}

enum Generator {
    Desert(Desert),
    // Boxed: it is much larger than `Desert`. With plates for `Plates`.
    Continents(Box<Continents>),
}

/// Generates surface samples for one world.
pub struct Surface {
    kind: SurfaceGenerator,
    generator: Generator,
    /// Grid units around the equator.
    width: i64,
    /// The world's sea level on the height scale, where 0 m is.
    sea_level: f64,
}

impl Surface {
    /// The world's own surface, by its `generator`.
    pub fn new(world: &WorldParams) -> Self {
        Self::with_generator(world, world.generator)
    }

    /// The surface another generator would make for this world.
    pub fn with_generator(world: &WorldParams, kind: SurfaceGenerator) -> Self {
        let width = (world.circumference / 1000).max(1) as i64;
        let generator = match kind {
            SurfaceGenerator::Desert => Generator::Desert(Desert::new(world)),
            SurfaceGenerator::Continents | SurfaceGenerator::Plates => {
                let with_plates = kind == SurfaceGenerator::Plates;
                Generator::Continents(Box::new(Continents::new(world, width, with_plates)))
            }
        };
        Self {
            kind,
            generator,
            width,
            sea_level: world.sea_level,
        }
    }

    pub fn generator(&self) -> SurfaceGenerator {
        self.kind
    }

    /// Grid units around the equator; `x` wraps at this value.
    pub fn width(&self) -> i64 {
        self.width
    }

    /// The height from only the largest noise layers: hills and mountains
    /// without the small bumps on them, on the same scale as `sample`'s
    /// height. It is what walking counts as climbing.
    pub fn broad_height(&self, x: i64, y: i64) -> f64 {
        if showcase::in_plain(y) {
            return showcase::FLAT_HEIGHT;
        }
        match &self.generator {
            Generator::Desert(desert) => desert.broad_height(x, y, self.width),
            Generator::Continents(continents) => self.on_scale(continents.metres(x, y, true).0),
        }
    }

    pub fn sample(&self, x: i64, y: i64) -> SurfaceSample {
        if showcase::in_plain(y) {
            return SurfaceSample {
                height: showcase::FLAT_HEIGHT,
                jitter: 0.0,
            };
        }
        match &self.generator {
            Generator::Desert(desert) => desert.sample(x, y, self.width),
            Generator::Continents(continents) => {
                let (metres, jitter) = continents.metres(x, y, false);
                SurfaceSample {
                    height: self.on_scale(metres),
                    jitter,
                }
            }
        }
    }

    /// Which plate a point is on, for generators with plates.
    pub fn plate_at(&self, x: i64, y: i64) -> Option<PlateSample> {
        match &self.generator {
            Generator::Continents(continents) => continents.plate_at(x, y),
            Generator::Desert(_) => None,
        }
    }

    /// The lowest height water is shaded down to: the deepest it gets.
    pub fn floor(&self) -> f64 {
        if self.kind.in_metres() {
            self.sea_level - DEEPEST
        } else {
            0.0
        }
    }

    /// How far a height on the height scale is above the sea, in whole metres
    /// (below it if negative).
    pub fn metres(&self, height: f64) -> i32 {
        ((height - self.sea_level) * METRES_PER_UNIT).round() as i32
    }

    /// A height in metres from the sea, on the height scale.
    fn on_scale(&self, metres: i32) -> f64 {
        self.sea_level + metres as f64 / METRES_PER_UNIT
    }
}

/// A noise generator for one purpose of the world's surface, at frequency 1,
/// as positions are scaled here.
fn new_noise(world: &WorldParams, purpose: &str) -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(world.seed.derive(purpose).noise_seed());
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise.set_frequency(Some(1.0));
    noise
}

/// Where a grid position is in noise space, `zoom` grid units to a unit.
fn cylinder(x: i64, y: i64, width: i64, zoom: f64) -> (f64, f64, f64) {
    // Wrap first so huge `x` values don't lose float precision.
    let angle = TAU * x.rem_euclid(width) as f64 / width as f64;
    // Radius chosen so one grid unit east is the same noise distance as one
    // unit south.
    let radius = width as f64 / (zoom * TAU);
    (
        libm::cos(angle) * radius,
        libm::sin(angle) * radius,
        y as f64 / zoom,
    )
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod continents_tests;

#[cfg(test)]
mod plates_tests;
