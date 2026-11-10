//! What grows at any world map position: how much tree cover and low growth.
//!
//! Follows the climate, with slow patchy noise deciding which parts of a
//! region are actually wooded, so forests have clearings and ragged edges.
//! Only something to look at for now. See `docs/context/shared/vegetation.md`.

use crate::climate::ClimateSample;
use crate::world::WorldParams;
use core::f64::consts::TAU;
use fastnoise_lite::{FastNoiseLite, NoiseType};

/// Grid units per unit of noise space: woods are tens of points across.
const ZOOM: f64 = 45.0;

/// Noise layers, each double the detail and half the strength.
const OCTAVES: u32 = 3;

/// Spreads the noise, which bunches around 0, more evenly over `0..=1`.
const STRETCH: f64 = 1.3;

/// How soft a wood's edge is, as a share of the patch noise each way.
const EDGE: f64 = 0.06;

/// Trees thin out to nothing between these heights, a little below where the
/// map's colours turn to tundra.
const TREE_LINE_FROM: f64 = 68.0;
const TREE_LINE_TO: f64 = 80.0;

/// What grows at one place, each `0..=1`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cover {
    /// How wooded it is.
    pub trees: f32,
    /// How much low growth (grass, scrub) there is under and between them.
    pub growth: f32,
    /// How narrow its trees are: 0 broad, 1 conifers.
    pub narrow: f32,
    /// How tall and uneven the canopy is: 1 in rainforest.
    pub lush: f32,
}

/// Works out the vegetation for one world.
pub struct Vegetation {
    patches: FastNoiseLite,
    /// Grid units around the equator, where `x` wraps.
    width: i64,
}

impl Vegetation {
    pub fn new(world: &WorldParams) -> Self {
        let mut patches = FastNoiseLite::with_seed(world.seed.derive("vegetation").noise_seed());
        patches.set_noise_type(Some(NoiseType::OpenSimplex2));
        // We scale coordinates ourselves.
        patches.set_frequency(Some(1.0));
        Self {
            patches,
            width: (world.circumference / 1000).max(1) as i64,
        }
    }

    /// What grows at a position with this climate and ground height, ignoring
    /// whether it is land: callers leave out water, shore and snow.
    pub fn cover(&self, x: i64, y: i64, height: f64, climate: ClimateSample) -> Cover {
        let ClimateSample {
            temperature,
            moisture,
        } = climate;
        let warm_enough = smoothstep(0.15, 0.3, temperature);
        let tree_line = 1.0 - smoothstep(TREE_LINE_FROM, TREE_LINE_TO, height);
        // Hot places need more rain for the same trees (savanna against forest).
        let thirst = 0.25 + 0.2 * smoothstep(0.6, 0.8, temperature);
        let wooded = smoothstep(thirst, thirst + 0.35, moisture) * warm_enough * tree_line;

        // The wetter and warmer, the more of the region the woods fill.
        let patch = self.patch(x, y);
        let trees = smoothstep(1.0 - wooded - EDGE, 1.0 - wooded + EDGE, patch) * wooded.sqrt();

        Cover {
            trees: trees as f32,
            growth: (smoothstep(0.08, 0.35, moisture) * smoothstep(0.08, 0.2, temperature)) as f32,
            narrow: (1.0 - smoothstep(0.32, 0.5, temperature)) as f32,
            lush: (smoothstep(0.55, 0.75, temperature) * smoothstep(0.55, 0.85, moisture)) as f32,
        }
    }

    /// Patchy noise at a position, `0..=1`.
    fn patch(&self, x: i64, y: i64) -> f64 {
        // As in `climate`: on a cylinder, one grid unit east the same noise
        // distance as one south, wrapped first to keep float precision.
        let angle = TAU * x.rem_euclid(self.width) as f64 / self.width as f64;
        let radius = self.width as f64 / (ZOOM * TAU);
        let (nx, ny, nz) = (
            libm::cos(angle) * radius,
            libm::sin(angle) * radius,
            y as f64 / ZOOM,
        );
        let mut total = 0.0;
        for octave in 0..OCTAVES {
            let scale = (1u32 << octave) as f64;
            total += self
                .patches
                .get_noise_3d(nx * scale, ny * scale, nz * scale) as f64
                / scale;
        }
        (0.5 + 0.5 * total * STRETCH).clamp(0.0, 1.0)
    }
}

/// 0 below `from`, 1 above `to`, an even S-curve between.
fn smoothstep(from: f64, to: f64, value: f64) -> f64 {
    let t = ((value - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests;
