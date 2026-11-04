use super::*;
use crate::rng::Seed;

fn plates(seed: u64) -> Surface {
    Surface::with_generator(
        &WorldParams::from_seed(Seed::new(seed)),
        SurfaceGenerator::Plates,
    )
}

/// Points spread evenly over the globe's area.
fn spread(surface: &Surface) -> Vec<(i64, i64)> {
    let width = surface.width();
    let mut points = Vec::new();
    for row in 0..40 {
        let sine = -1.0 + (2.0 * row as f64 + 1.0) / 40.0;
        let y = (libm::asin(sine) / core::f64::consts::FRAC_PI_2 * width as f64 / 4.0) as i64;
        for column in 0..80 {
            points.push((width * column / 80, y));
        }
    }
    points
}

#[test]
fn wraps_east_west() {
    let surface = plates(7);
    let width = surface.width();
    assert_eq!(surface.sample(3, 7), surface.sample(3 + width, 7));
    assert_eq!(surface.sample(-1, 7), surface.sample(width - 1, 7));
    assert_eq!(surface.plate_at(-1, 7), surface.plate_at(width - 1, 7));
}

#[test]
fn same_seed_same_planet() {
    assert_eq!(plates(99).sample(1234, -567), plates(99).sample(1234, -567));
}

#[test]
fn land_share_is_as_chosen() {
    for seed in [1, 2, 3, 4] {
        let surface = plates(seed);
        let points = spread(&surface);
        let land = points
            .iter()
            .filter(|&&(x, y)| surface.metres(surface.broad_height(x, y)) > 0)
            .count() as f64
            / points.len() as f64;
        assert!((0.15..0.5).contains(&land), "seed {seed}: land {land}");
    }
}

#[test]
fn plates_meet_and_raise_ranges() {
    let mut high = 0;
    let mut plates_seen = std::collections::HashSet::new();
    for seed in [1, 2, 3] {
        let surface = plates(seed);
        for (x, y) in spread(&surface) {
            plates_seen.insert((seed, surface.plate_at(x, y).expect("has plates").plate));
            if surface.metres(surface.broad_height(x, y)) > 3000 {
                high += 1;
            }
        }
    }
    assert!(plates_seen.len() > 30, "only {} plates", plates_seen.len());
    assert!(high > 0, "no ranges at all");
}

#[test]
fn only_plates_has_plates() {
    let world = WorldParams::from_seed(Seed::new(3));
    for kind in [SurfaceGenerator::Classic, SurfaceGenerator::Continents] {
        assert_eq!(Surface::with_generator(&world, kind).plate_at(10, 10), None);
    }
}
