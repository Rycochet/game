//! Where the suns and moons are at a time of day, and the light they give.

use core::f64::consts::{PI, TAU};

use super::{
    Body, BodyNow, Celestial, DAY_HOURS, FADE_HEIGHT, Sun, WAXING_LOOK_AHEAD, moon_at, scaled, unit,
};

impl Celestial {
    /// Every sun, then every moon.
    pub fn bodies(&self) -> Vec<Body> {
        let suns = self.suns.iter().map(|sun| sun.body);
        suns.chain(self.moons.iter().map(|moon| moon.body))
            .collect()
    }

    /// Where every sun and moon is at a time, in (fractional) game hours, and
    /// the light it gives. Everything that wants to know about them should
    /// come through here, so what is said matches what is seen.
    pub fn at(&self, hours: f64) -> Vec<BodyNow> {
        let suns_at = |hours: f64| -> Vec<BodyNow> {
            self.suns
                .iter()
                .map(|sun| self.sun_at(sun, hours))
                .collect()
        };
        let suns = suns_at(hours);
        let suns_later = suns_at(hours + WAXING_LOOK_AHEAD);
        let moons: Vec<BodyNow> = self
            .moons
            .iter()
            .map(|moon| moon_at(moon, &suns, &suns_later, hours))
            .collect();
        suns.into_iter().chain(moons).collect()
    }

    fn sun_at(&self, sun: &Sun, hours: f64) -> BodyNow {
        let angle = TAU * (hours - sun.noon) / sun.period;
        let across = libm::sin(angle);
        let east = if self.rises_in_east { -across } else { across };
        // A quarter turn of a sine over the time it is up, as how far east it is
        // comes back a little before a long day's end.
        let from_noon = (angle + PI).rem_euclid(TAU) - PI;
        let half_day = PI * self.daylight_hours / DAY_HOURS;
        let along = libm::sin((from_noon / half_day).clamp(-1.0, 1.0) * PI / 2.0);
        let up = libm::cos(angle) + rise_of(self.daylight_hours);
        let direction = unit([east, up, self.lean]);
        BodyNow {
            body: sun.body,
            direction,
            lit: 1.0,
            waxing: false,
            along,
            light: scaled(sun.body.colour, sun.strength * fade(direction)),
        }
    }
}

/// How far above its horizon-crossing level a sun's path is raised for the
/// day's length: 0 for half a day of light, more for longer days.
pub(super) fn rise_of(daylight_hours: f64) -> f64 {
    -libm::cos(PI * daylight_hours / DAY_HOURS)
}

pub(super) fn fade(direction: [f64; 3]) -> f64 {
    let t = (direction[1] / FADE_HEIGHT).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
