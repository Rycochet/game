use super::*;
use crate::rng::Seed;
use std::collections::HashSet;

fn climate() -> Climate {
    Climate::new(&WorldParams::from_seed(Seed::new(12345)))
}

// Golden values: if these change, every world's biomes change.
#[test]
fn golden_samples() {
    let climate = climate();
    let samples: Vec<_> = [(0, 0, 50.0), (1234, -5678, 60.0), (1_000_000, 42, 80.0)]
        .iter()
        .map(|&(x, y, height)| climate.sample(x, y, height))
        .collect();
    assert_eq!(format!("{samples:?}"), GOLDEN_SAMPLES);
}

const GOLDEN_SAMPLES: &str = "[ClimateSample { temperature: 0.7207018950599211, moisture: 0.05794992866699206 }, ClimateSample { temperature: 0.08091579423648787, moisture: 0.5796224217932354 }, ClimateSample { temperature: 0.13252501553257556, moisture: 0.6713264243747723 }]";

/// Snow is about as common as when it was a fixed height of 90.
#[test]
fn snow_covers_about_as_much_as_a_fixed_height_did() {
    let (mut snow, mut high, mut land) = (0, 0, 0);
    for seed in 1..=6 {
        let params = WorldParams::from_seed(Seed::new(seed));
        let climate = Climate::new(&params);
        let surface = crate::surface::Surface::new(&params);
        let quarter = (params.circumference / 4000) as i64;
        for y in 0..60 {
            for x in 0..60 {
                let (x, y) = (x * 211, (y - 30) * quarter / 30);
                let height = surface.sample(x, y).height;
                if height > params.high_water() {
                    land += 1;
                    high += usize::from(height > 90.0);
                    let cold = climate.sample(x, y, height).temperature;
                    snow += usize::from(cold < SNOW_TEMPERATURE);
                }
            }
        }
    }
    assert!(
        snow > high / 2 && snow < high * 2,
        "{snow} {high} of {land}"
    );
}

#[test]
fn wraps_east_west() {
    let climate = climate();
    let width = climate.width;
    assert_eq!(
        climate.sample(3, 7, 50.0),
        climate.sample(3 + width, 7, 50.0)
    );
}

#[test]
fn higher_is_colder() {
    let climate = climate();
    assert!(climate.sample(9, 9, 90.0).temperature <= climate.sample(9, 9, 40.0).temperature);
}

/// The lookup covers everything, and each biome has a place on it.
#[test]
fn every_biome_is_on_the_diagram() {
    let mut found = HashSet::new();
    for t in 0..=20 {
        for m in 0..=20 {
            found.insert(Biome::of(t as f64 / 20.0, m as f64 / 20.0));
        }
    }
    assert_eq!(found.len(), Biome::ALL.len());
}

/// Walking the near-equator world finds a good spread of biomes, so the
/// noise isn't bunched into a few of them.
#[test]
fn real_terrain_has_variety() {
    let params = WorldParams::from_seed(Seed::new(12345));
    let climate = Climate::new(&params);
    let surface = crate::surface::Surface::new(&params);
    let mut found = HashSet::new();
    for y in (-6000..6000).step_by(40) {
        for x in (-6000..6000).step_by(40) {
            let height = surface.sample(x, y).height;
            found.insert(climate.sample(x, y, height).biome());
        }
    }
    assert!(found.len() >= 7, "only {found:?}");
}

#[test]
fn air_cools_at_earths_rate_above_the_sea() {
    let climate = Climate::new(&WorldParams::from_seed(Seed::new(5)));
    let low = climate.sample_metres(100, 50, 0);
    let high = climate.sample_metres(100, 50, 1000);
    let cooled = low.celsius() - high.celsius();
    if low.temperature < 1.0 && high.temperature > 0.0 {
        assert!((cooled - 6.5).abs() < 1e-9, "{cooled}");
    }
    // Under the sea counts as at it.
    assert_eq!(climate.sample_metres(100, 50, -3000), low);
}
