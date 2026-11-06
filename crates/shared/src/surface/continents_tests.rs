use super::*;
use crate::rng::Seed;

fn continents(seed: u64) -> Surface {
    Surface::with_generator(
        &WorldParams::from_seed(Seed::new(seed)),
        SurfaceGenerator::Continents,
    )
}

/// Heights in metres at points spread evenly over the globe's area, by the
/// broad height (as the planet's picture shows it).
fn broad_metres(surface: &Surface) -> Vec<i32> {
    let width = surface.width();
    let mut metres = Vec::new();
    for row in 0..40 {
        let sine = -1.0 + (2.0 * row as f64 + 1.0) / 40.0;
        let y = (libm::asin(sine) / core::f64::consts::FRAC_PI_2 * width as f64 / 4.0) as i64;
        for column in 0..80 {
            let height = surface.broad_height(width * column / 80, y);
            metres.push(surface.metres(height));
        }
    }
    metres
}

#[test]
fn wraps_east_west() {
    let surface = continents(7);
    let width = surface.width();
    assert_eq!(surface.sample(3, 7), surface.sample(3 + width, 7));
    assert_eq!(surface.sample(-1, 7), surface.sample(width - 1, 7));
}

#[test]
fn same_seed_same_planet() {
    assert_eq!(
        continents(99).sample(1234, -567),
        continents(99).sample(1234, -567)
    );
}

#[test]
fn land_share_is_as_chosen() {
    for seed in [1, 2, 3, 4] {
        let metres = broad_metres(&continents(seed));
        let land = metres.iter().filter(|&&m| m > 0).count() as f64 / metres.len() as f64;
        assert!((0.15..0.5).contains(&land), "seed {seed}: land {land}");
    }
}

#[test]
fn most_land_is_low_and_mountains_rare() {
    let mut land = Vec::new();
    for seed in [1, 2, 3, 4, 5, 6] {
        land.extend(
            broad_metres(&continents(seed))
                .into_iter()
                .filter(|&m| m > 0),
        );
    }
    let share = |above: i32| land.iter().filter(|&&m| m > above).count() as f64 / land.len() as f64;
    assert!(share(1000) < 0.35, "over 1 km: {}", share(1000));
    assert!(share(3000) < 0.08, "over 3 km: {}", share(3000));
    assert!(share(3000) > 0.0, "no mountains at all");
}

#[test]
fn oceans_have_shelves_and_depths() {
    let metres = broad_metres(&continents(3));
    let sea: Vec<_> = metres.iter().filter(|&&m| m <= 0).collect();
    let shallow = sea.iter().filter(|&&&m| m > -200).count();
    let deep = sea.iter().filter(|&&&m| m < -3000).count();
    assert!(
        shallow > 0 && deep > sea.len() / 2,
        "shallow {shallow}, deep {deep} of {}",
        sea.len()
    );
}

#[test]
fn desert_is_still_the_default() {
    let world = WorldParams::from_seed(Seed::new(12345));
    assert_eq!(Surface::new(&world).generator(), SurfaceGenerator::Desert);
    assert_eq!(
        Surface::new(&world).sample(5, 5),
        Surface::with_generator(&world, SurfaceGenerator::Desert).sample(5, 5)
    );
}
