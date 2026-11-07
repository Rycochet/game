//! A Dev Area grown as a biome: ground, grass and trees rolled as the
//! wilderness rolls them, from the vegetation of a typical place of that biome.

use super::DevArea;
use crate::climate::{Biome, ClimateSample};
use crate::dungeon::{Cell, Level};
use crate::rng::Seed;
use crate::vegetation::Vegetation;
use crate::wilderness::MOST_TREES_PERCENT;
use crate::world::WorldParams;

/// A ground height well inside the land, below the tree line.
const HEIGHT: f64 = 45.0;
/// Rolls are out of this many.
const ROLL_OUT_OF: u64 = 1_000_000;
/// How far apart, in world map points, different seeds' areas are taken from.
const SPREAD: u64 = 10_000;

/// A temperature and moisture in the middle of each biome's part of the
/// Whittaker diagram (`Biome::of`).
pub(super) fn typical(biome: Biome) -> ClimateSample {
    let (temperature, moisture) = match biome {
        Biome::Tundra => (0.15, 0.4),
        Biome::BorealForest => (0.3, 0.65),
        Biome::Grassland => (0.55, 0.15),
        Biome::Woodland => (0.55, 0.4),
        Biome::SeasonalForest => (0.55, 0.65),
        Biome::TemperateRainforest => (0.55, 0.9),
        Biome::Desert => (0.85, 0.1),
        Biome::Savanna => (0.85, 0.45),
        Biome::TropicalRainforest => (0.85, 0.85),
    };
    ClimateSample {
        temperature,
        moisture,
    }
}

/// Grows the biome on the level's open ground. Each cell stands for a world map
/// point (closer than the wilderness's 16 cells), so woods and clearings fit
/// in a small area; the seed picks where on the patch noise it is taken from.
pub(super) fn grow(level: &mut Level, area: &DevArea, biome: Biome) {
    let seed = Seed::new(area.seed);
    let vegetation = Vegetation::new(&WorldParams::from_seed(seed));
    let climate = typical(biome);
    let from = (seed.at(&[0]) % SPREAD) as i64;
    let trees_roll = seed.derive("dev area trees");
    let growth_roll = seed.derive("dev area growth");
    let roll =
        |seed: Seed, x: i64, y: i64| (seed.at(&[x, y]) % ROLL_OUT_OF) as f64 / ROLL_OUT_OF as f64;
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if level.cell(x, y) != Cell::Ground {
                continue;
            }
            let cover = vegetation.cover(from + x, y, HEIGHT, climate);
            let trees = f64::from(cover.trees) * MOST_TREES_PERCENT as f64 / 100.0;
            let cell = if roll(trees_roll, x, y) < trees {
                Cell::Tree
            } else if roll(growth_roll, x, y) < f64::from(cover.growth) {
                Cell::Grass
            } else {
                Cell::Ground
            };
            level.set(x, y, cell);
        }
    }
}
