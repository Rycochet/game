//! Light from the bodies in the sky: colours along a body's path, flat light, the brightest.

use super::{BodyKind, BodyNow, MOON_RICHEST, from_hue, sun_colour};

/// A colour for this kind of body, `0..=1` along every one it can pick: suns
/// from blue to orange, moons round the hues. Generated bodies can be paler.
pub fn colour_along(kind: BodyKind, along: f64) -> [f64; 3] {
    match kind {
        BodyKind::Sun => sun_colour(along, 0.0),
        BodyKind::Moon => from_hue(along * 6.0, MOON_RICHEST),
    }
}

/// The light on flat ground from all of these together, linear RGB.
pub fn flat_light(bodies: &[BodyNow]) -> [f64; 3] {
    let mut total = [0.0; 3];
    for now in bodies {
        let height = now.direction[1].max(0.0);
        for (channel, light) in total.iter_mut().zip(now.light) {
            *channel += light * height;
        }
    }
    total
}

/// Which of these lights flat ground the most: the first (the main sun) if
/// nothing does.
pub fn brightest(bodies: &[BodyNow]) -> usize {
    let mut best = 0;
    for (i, now) in bodies.iter().enumerate() {
        if now.flat_light() > bodies[best].flat_light() {
            best = i;
        }
    }
    best
}
