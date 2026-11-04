//! The first generator: seven layers of noise on one scale, the largest about
//! 256 grid units across. Good for hills and randomness, but nothing on it is
//! continent-sized.

use fastnoise_lite::FastNoiseLite;

use super::{SurfaceSample, cylinder, new_noise};
use crate::world::WorldParams;

/// Grid units per unit of noise space: higher gives larger, smoother pois.
const ZOOM: f64 = 256.0;

/// Number of noise layers, each double the detail and half the strength.
const OCTAVES: u32 = 7;

/// How many of the largest layers make up `broad_height`.
const BROAD_OCTAVES: u32 = 3;

/// The layer also used for `jitter`.
const JITTER_OCTAVE: u32 = 5;

pub(super) struct Classic {
    noise: FastNoiseLite,
}

impl Classic {
    pub(super) fn new(world: &WorldParams) -> Self {
        Self {
            noise: new_noise(world, "land"),
        }
    }

    pub(super) fn broad_height(&self, x: i64, y: i64, width: i64) -> f64 {
        let (nx, ny, nz) = cylinder(x, y, width, ZOOM);
        let mut height = 128.0;
        for octave in 0..BROAD_OCTAVES {
            let scale = (1u32 << octave) as f64;
            let value = self.noise.get_noise_3d(nx * scale, ny * scale, nz * scale) as f64;
            height += value * 64.0 / scale;
        }
        height / 2.0
    }

    pub(super) fn sample(&self, x: i64, y: i64, width: i64) -> SurfaceSample {
        let (nx, ny, nz) = cylinder(x, y, width, ZOOM);
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
