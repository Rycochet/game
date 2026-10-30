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
    /// whether it is land: callers leave out water, sand and snow.
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
mod tests {
    use super::*;
    use crate::climate::Climate;
    use crate::rng::Seed;
    use crate::surface::Surface;

    fn vegetation() -> Vegetation {
        Vegetation::new(&WorldParams::from_seed(Seed::new(12345)))
    }

    fn place(temperature: f64, moisture: f64) -> ClimateSample {
        ClimateSample {
            temperature,
            moisture,
        }
    }

    /// The share of 1600 places around here with trees on them.
    fn wooded_share(climate: ClimateSample) -> f64 {
        let vegetation = vegetation();
        let mut wooded = 0;
        for y in 0..40 {
            for x in 0..40 {
                let cover = vegetation.cover(x * 13, y * 13, 45.0, climate);
                wooded += usize::from(cover.trees > 0.5);
            }
        }
        wooded as f64 / 1600.0
    }

    // Golden values: if these change, every world's vegetation changes.
    #[test]
    fn golden_samples() {
        let vegetation = vegetation();
        let samples: Vec<_> = [(0, 0), (1234, -5678), (1_000_000, 42)]
            .iter()
            .map(|&(x, y)| vegetation.patch(x, y))
            .collect();
        assert_eq!(format!("{samples:?}"), GOLDEN_SAMPLES);
    }

    const GOLDEN_SAMPLES: &str = "[0.3625948317348957, 0.9510458312928677, 0.36863902360200884]";

    #[test]
    fn wraps_east_west() {
        let vegetation = vegetation();
        let width = vegetation.width;
        let climate = place(0.6, 0.6);
        assert_eq!(
            vegetation.cover(3, 7, 45.0, climate),
            vegetation.cover(3 + width, 7, 45.0, climate)
        );
    }

    #[test]
    fn rainforest_is_mostly_trees_and_desert_has_none() {
        assert!(wooded_share(place(0.8, 0.95)) > 0.7);
        assert_eq!(wooded_share(place(0.8, 0.1)), 0.0);
    }

    #[test]
    fn wet_woods_cover_more_than_dry_ones_but_leave_clearings() {
        let (dry, wet) = (
            wooded_share(place(0.55, 0.42)),
            wooded_share(place(0.55, 0.6)),
        );
        assert!(dry > 0.0 && dry < wet && wet < 1.0, "{dry} {wet}");
    }

    #[test]
    fn no_trees_above_the_tree_line() {
        let cover = vegetation().cover(5, 5, TREE_LINE_TO, place(0.6, 0.9));
        assert_eq!(cover.trees, 0.0);
    }

    /// On real terrain some land is wooded and some is open.
    #[test]
    fn real_terrain_has_woods_and_open_land() {
        let params = WorldParams::from_seed(Seed::new(12345));
        let (climate, surface) = (Climate::new(&params), Surface::new(&params));
        let vegetation = Vegetation::new(&params);
        let (mut wooded, mut land) = (0, 0);
        for y in (-6000..6000).step_by(40) {
            for x in (-6000..6000).step_by(40) {
                let height = surface.sample(x, y).height;
                if height > params.high_water() {
                    land += 1;
                    let here = climate.sample(x, y, height);
                    wooded += usize::from(vegetation.cover(x, y, height, here).trees > 0.5);
                }
            }
        }
        let share = wooded as f64 / land as f64;
        assert!((0.15..0.7).contains(&share), "{share}");
    }
}
