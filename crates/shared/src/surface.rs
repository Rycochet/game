//! The planet's surface height at any grid position.
//!
//! The surface is wrapped around a cylinder in 3D noise space, so travelling
//! east or west eventually returns to the start. North/south does not wrap.
//! One grid unit is about 1 km.

use crate::world::WorldParams;
use core::f64::consts::TAU;
use fastnoise_lite::{FastNoiseLite, NoiseType};

/// Grid units per unit of noise space: higher gives larger, smoother features.
const ZOOM: f64 = 256.0;

/// Number of noise layers, each double the detail and half the strength.
const OCTAVES: u32 = 7;

/// The layer also used for `jitter`.
const JITTER_OCTAVE: u32 = 5;

/// Height and texture at one grid position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceSample {
    /// Ground height, roughly `0.5..127.5`, centred on 64.
    pub height: f64,
    /// Small-scale variation, `-8..8`, used to roughen the edges between
    /// terrain bands. Not part of the physical height.
    pub jitter: f64,
}

/// Generates surface samples for one world.
pub struct Surface {
    noise: FastNoiseLite,
    /// Grid units around the equator.
    width: i64,
}

impl Surface {
    pub fn new(world: &WorldParams) -> Self {
        let mut noise = FastNoiseLite::with_seed(world.seed.derive("land").noise_seed());
        noise.set_noise_type(Some(NoiseType::OpenSimplex2));
        // We scale coordinates ourselves.
        noise.set_frequency(Some(1.0));
        Self {
            noise,
            width: (world.circumference / 1000).max(1) as i64,
        }
    }

    /// Grid units around the equator; `x` wraps at this value.
    pub fn width(&self) -> i64 {
        self.width
    }

    pub fn sample(&self, x: i64, y: i64) -> SurfaceSample {
        // Wrap first so huge `x` values don't lose float precision.
        let angle = TAU * x.rem_euclid(self.width) as f64 / self.width as f64;
        // Radius chosen so one grid unit east is the same noise distance as
        // one unit south.
        let radius = self.width as f64 / (ZOOM * TAU);
        let nx = libm::cos(angle) * radius;
        let ny = libm::sin(angle) * radius;
        let nz = y as f64 / ZOOM;

        let mut height = 128.0;
        let mut jitter = 0.0;
        for octave in 0..OCTAVES {
            let scale = (1u32 << octave) as f64;
            let value = self.noise.get_noise_3d(nx * scale, ny * scale, nz * scale) as f64;
            height += value * 64.0 / scale;
            if octave == JITTER_OCTAVE {
                jitter = value * 8.0;
            }
        }

        SurfaceSample {
            height: height / 2.0,
            jitter,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Seed;

    fn surface() -> Surface {
        Surface::new(&WorldParams::from_seed(Seed::new(12345)))
    }

    // Golden values: if these change, every generated world changes.
    #[test]
    fn golden_samples() {
        let surface = surface();
        let samples: Vec<_> = [(0, 0), (1, 0), (-1, 5), (1234, -5678), (1_000_000, 42)]
            .iter()
            .map(|&(x, y)| surface.sample(x, y))
            .collect();
        assert_eq!(format!("{samples:?}"), GOLDEN_SAMPLES);
    }

    const GOLDEN_SAMPLES: &str = "[SurfaceSample { height: 66.09790093125775, jitter: -6.349335193634033 }, SurfaceSample { height: 66.56663356721401, jitter: -5.502878665924072 }, SurfaceSample { height: 69.92800272256136, jitter: -0.8336078524589539 }, SurfaceSample { height: 47.342495096847415, jitter: -0.28205934166908264 }, SurfaceSample { height: 46.198867592960596, jitter: -7.5050225257873535 }]";

    #[test]
    fn wraps_east_west() {
        let surface = surface();
        let width = surface.width();
        assert_eq!(surface.sample(3, 7), surface.sample(3 + width, 7));
        assert_eq!(surface.sample(-1, 7), surface.sample(width - 1, 7));
    }
}
