//! The planet's surface height at any grid position.
//!
//! The surface is wrapped around a cylinder in 3D noise space, so travelling
//! east or west eventually returns to the start. North/south does not wrap.
//! One grid unit is about 1 km. There is more than one way of making it (see
//! `SurfaceGenerator`); the worlds themselves all use `Classic` for now.

mod classic;
mod continents;

use crate::showcase;
use crate::world::WorldParams;
use classic::Classic;
use continents::Continents;
use core::f64::consts::TAU;
use fastnoise_lite::{FastNoiseLite, NoiseType};

/// Metres for each unit of the height scale, measuring up from the sea.
pub const METRES_PER_UNIT: f64 = 100.0;

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
    /// Whether this point is ice: under water, with the snow line below it.
    /// Freezing spreads from the shallows as the snow line falls.
    pub fn is_frozen(&self, water_level: f64, snow_line: f64) -> bool {
        self.height <= water_level && self.height + self.jitter > snow_line
    }
}

/// The ways of making a surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SurfaceGenerator {
    /// Layered noise on one scale: what every world uses for now.
    #[default]
    Classic,
    /// Continents and oceans with shelves, worked out in metres, with rare
    /// mountain ranges inland.
    Continents,
}

impl SurfaceGenerator {
    pub const ALL: [SurfaceGenerator; 2] =
        [SurfaceGenerator::Classic, SurfaceGenerator::Continents];

    pub fn name(self) -> &'static str {
        match self {
            SurfaceGenerator::Classic => "Classic",
            SurfaceGenerator::Continents => "Continents",
        }
    }
}

enum Generator {
    Classic(Classic),
    // Boxed: it is much larger than `Classic`.
    Continents(Box<Continents>),
}

/// Generates surface samples for one world.
pub struct Surface {
    generator: Generator,
    /// Grid units around the equator.
    width: i64,
    /// The world's sea level on the height scale, where 0 m is.
    sea_level: f64,
}

impl Surface {
    /// The world's own surface.
    pub fn new(world: &WorldParams) -> Self {
        Self::with_generator(world, SurfaceGenerator::Classic)
    }

    /// The surface another generator would make for this world.
    pub fn with_generator(world: &WorldParams, kind: SurfaceGenerator) -> Self {
        let width = (world.circumference / 1000).max(1) as i64;
        let generator = match kind {
            SurfaceGenerator::Classic => Generator::Classic(Classic::new(world)),
            SurfaceGenerator::Continents => {
                Generator::Continents(Box::new(Continents::new(world, width)))
            }
        };
        Self {
            generator,
            width,
            sea_level: world.sea_level,
        }
    }

    pub fn generator(&self) -> SurfaceGenerator {
        match self.generator {
            Generator::Classic(_) => SurfaceGenerator::Classic,
            Generator::Continents(_) => SurfaceGenerator::Continents,
        }
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
            Generator::Classic(classic) => classic.broad_height(x, y, self.width),
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
            Generator::Classic(classic) => classic.sample(x, y, self.width),
            Generator::Continents(continents) => {
                let (metres, jitter) = continents.metres(x, y, false);
                SurfaceSample {
                    height: self.on_scale(metres),
                    jitter,
                }
            }
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
