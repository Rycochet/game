//! Where things are round the globe: its turn, the suns' light and the moons'
//! orbits, from `shared::celestial`. Pure maths, so it can be tested alone.
//!
//! The globe's axis is +Y, with north up. A point of the equator `angle`
//! radians round from +X is `equator(angle)`, and angles grow towards the
//! east, as the world map's `x` does. The camera looks from +Z.

use std::f32::consts::{PI, TAU};

use bevy::math::{Quat, Vec3};
use shared::celestial::{BodyNow, Celestial};

/// Where round the equator the noon sun is: about 60 degrees right of the
/// side facing the camera, so most of what is seen is lit, and night shows.
const SUN_ANGLE: f32 = -PI / 2.0 + 1.05;

/// Moons' distances from the globe's centre, in its radii, from the quickest
/// orbit there can be to the slowest.
const NEAREST_MOON: f32 = 1.7;
const FURTHEST_MOON: f32 = 4.0;

/// The quickest and slowest moon orbits there can be, in hours
/// (`shared::celestial`'s range).
const FASTEST_ORBIT: f64 = 7.0;
const SLOWEST_ORBIT: f64 = 90.0;

/// The point of the equator `angle` radians east of +X.
pub(super) fn equator(angle: f32) -> Vec3 {
    Vec3::new(angle.cos(), 0.0, -angle.sin())
}

/// East along the ground at the point of the equator at `angle`.
fn eastward(angle: f32) -> Vec3 {
    Vec3::new(-angle.sin(), 0.0, -angle.cos())
}

/// A direction seen from the ground (`[east, up, south]`) at the point of the
/// equator at `angle`, as a direction round the globe.
pub(super) fn from_ground(direction: [f64; 3], angle: f32) -> Vec3 {
    let [east, up, south] = direction.map(|part| part as f32);
    eastward(angle) * east + equator(angle) * up - Vec3::Y * south
}

/// A world's globe and sky.
#[derive(Clone, Copy, Debug)]
pub(super) struct Sky {
    /// 1 if the globe turns towards the east (suns rise in the east), else -1.
    spin: f32,
    /// Towards the main sun, which stays put while the globe turns under it.
    main_sun: Vec3,
}

impl Sky {
    pub(super) fn new(celestial: &Celestial) -> Self {
        let main_sun = |hours| celestial.at(hours)[0].direction;
        // An hour before noon a sun rising in the east is still in the east.
        let spin = if main_sun(11.0)[0] > 0.0 { 1.0 } else { -1.0 };
        Self {
            spin,
            main_sun: from_ground(main_sun(12.0), SUN_ANGLE).normalize(),
        }
    }

    /// How far round the globe is turned at an hour: where `x = 0` on the
    /// equator is, which faces the main sun at noon. The world has no time
    /// zones, so this is where its clock's time of day is true.
    pub(super) fn ground_angle(&self, hours: f64) -> f32 {
        let day = ((hours - 12.0) / 24.0).rem_euclid(1.0) as f32;
        SUN_ANGLE + self.spin * TAU * day
    }

    /// The globe's turn at an hour.
    pub(super) fn globe_turn(&self, hours: f64) -> Quat {
        Quat::from_rotation_y(self.ground_angle(hours))
    }

    /// Towards a sun (`number` in `Celestial::bodies`' order). The main sun
    /// stays put; others are where they are seen from `x = 0`.
    pub(super) fn sun_direction(&self, number: usize, sun: &BodyNow, hours: f64) -> Vec3 {
        if number == 0 {
            self.main_sun
        } else {
            from_ground(sun.direction, self.ground_angle(hours)).normalize()
        }
    }

    /// Where a moon is. Moons' orbits ignore the planet's turning, so they
    /// stay put in space while the globe turns.
    pub(super) fn moon_place(&self, moon: &BodyNow, period: f64) -> Vec3 {
        from_ground(moon.direction, SUN_ANGLE) * moon_distance(period)
    }
}

/// How far out a moon is, in the globe's radii: by Kepler's rule the distance
/// grows with the period to the power 2/3, here squeezed between the nearest
/// and furthest, so every moon fits the view.
pub(super) fn moon_distance(period: f64) -> f32 {
    let kepler = |hours: f64| hours.powf(2.0 / 3.0);
    let along =
        (kepler(period) - kepler(FASTEST_ORBIT)) / (kepler(SLOWEST_ORBIT) - kepler(FASTEST_ORBIT));
    NEAREST_MOON + (FURTHEST_MOON - NEAREST_MOON) * along.clamp(0.0, 1.0) as f32
}
