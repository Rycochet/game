use bevy::math::{UVec2, Vec3};
use shared::celestial::Celestial;
use shared::rng::Seed;

use super::moon_look::MoonLook;
use super::sky::{Sky, equator, from_ground, moon_distance};
use super::texture::{ground_at, texture_size};

#[test]
fn the_picture_is_twice_as_wide_as_it_is_tall_and_no_wider_than_the_world() {
    assert_eq!(texture_size(40_000), UVec2::new(1024, 512));
    assert_eq!(texture_size(600), UVec2::new(600, 300));
    assert_eq!(texture_size(3), UVec2::new(3, 1));
}

#[test]
fn the_picture_runs_round_the_world_and_from_pole_to_pole() {
    let width = 4000;
    let size = texture_size(width);
    assert_eq!(ground_at(0, 0, size, width), (1, -998));
    assert_eq!(ground_at(size.x - 1, size.y - 1, size, width), (3998, 998));
    // The equator is between the middle rows.
    let (_, north) = ground_at(0, size.y / 2 - 1, size, width);
    let (_, south) = ground_at(0, size.y / 2, size, width);
    assert!(north < 0 && south > 0, "{north} {south}");
}

#[test]
fn up_and_east_on_the_ground_match_the_globe() {
    for angle in [0.0, 1.0, -2.5] {
        let up = from_ground([0.0, 1.0, 0.0], angle);
        assert!(up.distance(equator(angle)) < 1e-5);
        // A little east along the ground is a little further round.
        let east = from_ground([1.0, 0.0, 0.0], angle);
        let ahead = equator(angle + 0.01) - equator(angle);
        assert!(east.dot(ahead) > 0.0);
        let south = from_ground([0.0, 0.0, 1.0], angle);
        assert!(south.distance(Vec3::NEG_Y) < 1e-5);
    }
}

#[test]
fn at_noon_the_ground_at_x_0_faces_the_main_sun() {
    for seed in 0..50 {
        let celestial = Celestial::from_seed(Seed::new(seed));
        let sky = Sky::new(&celestial);
        let main = celestial.at(36.0)[0];
        let up = equator(sky.ground_angle(36.0));
        let sun = sky.sun_direction(0, &main, 36.0);
        // Straight overhead but for the sun's lean north or south.
        let flat = Vec3::new(sun.x, 0.0, sun.z).normalize();
        assert!(flat.distance(up) < 1e-4, "seed {seed}");
    }
}

#[test]
fn suns_are_seen_in_the_morning_where_they_rise() {
    for seed in 0..50 {
        let celestial = Celestial::from_seed(Seed::new(seed));
        let sky = Sky::new(&celestial);
        let rises_east = celestial.at(9.0)[0].direction[0] > 0.0;
        let angle = sky.ground_angle(9.0);
        let main = celestial.at(9.0)[0];
        let east = from_ground([1.0, 0.0, 0.0], angle);
        let seen_east = sky.sun_direction(0, &main, 9.0).dot(east) > 0.0;
        assert_eq!(seen_east, rises_east, "seed {seed}");
    }
}

#[test]
fn slower_moons_are_further_out() {
    let distances: Vec<f32> = [7.0, 20.0, 50.0, 90.0].map(moon_distance).to_vec();
    assert!(distances.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(distances[0] > 1.5 && distances[3] <= 4.0);
}

#[test]
fn moons_look_the_same_every_time() {
    let seed = Seed::new(99);
    let first = MoonLook::of(seed, 0);
    let again = MoonLook::of(seed, 0);
    assert_eq!(first.style, again.style);
    assert_eq!(first.turn(10.0), again.turn(10.0));
    assert_eq!(first.image().data, again.image().data);
}

#[test]
fn every_style_of_moon_turns_up() {
    let mut styles = Vec::new();
    for number in 0..40 {
        let style = MoonLook::of(Seed::new(5), number).style;
        if !styles.contains(&style) {
            styles.push(style);
        }
    }
    assert_eq!(styles.len(), 4);
}
