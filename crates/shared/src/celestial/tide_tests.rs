use super::*;
use crate::rng::Seed;

/// A moon of `size` going round in 24 hours, `start` of the way round at hour 0,
/// whose orbit climbs to `top` (the sine of its highest point's height).
fn moon(size: f64, start: f64, top: f64) -> Moon {
    let flat = (1.0 - top * top).sqrt();
    Moon {
        body: Body {
            kind: BodyKind::Moon,
            colour: [1.0; 3],
            colour_name: None,
            size,
        },
        across: [[0.0, top, flat], [1.0, 0.0, 0.0]],
        period: 24.0,
        start,
    }
}

fn sky_with(moons: Vec<Moon>) -> Celestial {
    let mut sky = Celestial::from_seed(Seed::new(12345));
    sky.moons = moons;
    sky
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn no_moons_keep_the_sea_at_mid_tide() {
    let sky = sky_with(Vec::new());
    assert!((0..48).all(|hour| sky.tide_pull(hour as f64) == 0.0));
}

#[test]
fn one_moon_pulls_most_at_the_zenith_and_least_at_the_nadir() {
    let sky = sky_with(vec![moon(0.5, 0.0, 1.0)]);
    assert!(near(sky.tide_pull(0.0), 1.0));
    assert!(near(sky.tide_pull(6.0), 0.0));
    assert!(near(sky.tide_pull(12.0), -1.0));
}

#[test]
fn a_moon_that_stays_low_makes_gentle_tides() {
    let top = libm::sin(20f64.to_radians());
    let sky = sky_with(vec![moon(1.0, 0.0, top)]);
    let highest = (0..240)
        .map(|tenth| sky.tide_pull(tenth as f64 / 10.0))
        .fold(f64::MIN, f64::max);
    assert!(near(highest, top));
}

#[test]
fn moons_together_add_up_and_opposite_ones_cancel() {
    let together = sky_with(vec![moon(0.6, 0.0, 1.0), moon(0.6, 0.0, 1.0)]);
    assert!(near(together.tide_pull(0.0), 1.0));
    let opposite = sky_with(vec![moon(0.6, 0.0, 1.0), moon(0.6, 0.5, 1.0)]);
    assert!((0..24).all(|hour| near(opposite.tide_pull(hour as f64), 0.0)));
}

#[test]
fn heavier_moons_pull_harder() {
    // One of size 1 at the zenith, one of size 0.5 (an eighth of the mass) at
    // the nadir.
    let sky = sky_with(vec![moon(1.0, 0.0, 1.0), moon(0.5, 0.5, 1.0)]);
    assert!(near(sky.tide_pull(0.0), (1.0 - 0.125) / 1.125));
}

#[test]
fn the_pull_never_goes_past_full() {
    for seed in 0..200 {
        let sky = Celestial::from_seed(Seed::new(seed));
        for hour in 0..200 {
            let pull = sky.tide_pull(hour as f64 * 0.7);
            assert!((-1.0..=1.0).contains(&pull), "{seed} {hour}: {pull}");
        }
    }
}
