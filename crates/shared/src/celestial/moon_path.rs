//! Where a moon is in the sky, and how much of it is lit.

use core::f64::consts::TAU;

use super::{BodyNow, MOONLIGHT, Moon, WAXING_LOOK_AHEAD, cross, dot, fade, unit};

/// The way a moon is going at the top of its orbit.
pub(super) fn moon_way_over(moon: &Moon) -> [f64; 3] {
    let axis = unit(cross(moon.across[0], moon.across[1]));
    let up = axis[1];
    let top = unit([-axis[0] * up, 1.0 - up * up, -axis[2] * up]);
    cross(axis, top)
}

pub(super) fn moon_direction(moon: &Moon, hours: f64) -> [f64; 3] {
    let angle = TAU * (hours / moon.period + moon.start);
    let (sine, cosine) = (libm::sin(angle), libm::cos(angle));
    let [a, b] = moon.across;
    [0, 1, 2].map(|i| a[i] * cosine + b[i] * sine)
}

/// How far a moon's face is lit: by the sun lighting most of it.
pub(super) fn lit_by(direction: [f64; 3], suns: &[BodyNow]) -> f64 {
    suns.iter()
        .map(|sun| (1.0 - dot(direction, sun.direction)) / 2.0)
        .fold(0.0, f64::max)
}

pub(super) fn moon_at(
    moon: &Moon,
    suns: &[BodyNow],
    suns_later: &[BodyNow],
    hours: f64,
) -> BodyNow {
    let direction = moon_direction(moon, hours);
    let later = moon_direction(moon, hours + WAXING_LOOK_AHEAD);

    // Each sun lights the side of the moon facing it: all of what we see when
    // the sun is behind us, none when it is behind the moon. Suns light it
    // whether or not they are above our horizon.
    let reflected = moon.body.size * moon.body.size * MOONLIGHT * fade(direction);
    let mut light = [0.0; 3];
    let lit = lit_by(direction, suns);
    for sun in suns {
        let facing = (1.0 - dot(direction, sun.direction)) / 2.0;
        for (i, channel) in light.iter_mut().enumerate() {
            *channel += sun.body.colour[i] * moon.body.colour[i] * facing * reflected;
        }
    }
    BodyNow {
        body: moon.body,
        direction,
        lit,
        waxing: lit_by(later, suns_later) > lit,
        along: dot(direction, moon_way_over(moon)),
        light,
    }
}
