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

const GOLDEN_SAMPLES: &str = "[0.9960789998993278, 0.1914533587172627, 0.6100821044296026]";

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
