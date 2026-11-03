//! One sun or moon: what it is, and where it is and the light it gives now.

use super::{ColourName, NEW_MOON_BELOW, QUARTER_WITHIN};

/// A sun or a moon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyKind {
    Sun,
    Moon,
}

/// Where a moon is in its phases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoonPhase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

/// One of a world's suns or moons, whatever the time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub kind: BodyKind,
    /// Its colour, linear RGB with the strongest channel at 1.
    pub colour: [f64; 3],
    /// Its colour's name, `None` if it is too near white to have one.
    pub colour_name: Option<ColourName>,
    /// How big it is: suns about `0.5..2`, moons `0.3..1`.
    pub size: f64,
}

/// One of a world's suns or moons at one time. `Celestial::at` gives these in
/// the order of `Celestial::bodies`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyNow {
    pub body: Body,
    /// A unit vector towards it, `[east, up, south]`.
    pub direction: [f64; 3],
    /// How far a moon's face we see is lit, `0..=1`. Suns are always 1.
    pub lit: f64,
    /// Whether a moon's lit part is growing. Never for suns.
    pub waxing: bool,
    /// How far along its way across the sky it is, whichever way that goes on the
    /// world: negative before its highest point, positive after, always rising.
    pub along: f64,
    /// The light it gives now, linear RGB, faded near the horizon: zero below
    /// it. Noon's main sun gives its colour on flat ground, so more than 1 here.
    pub light: [f64; 3],
}

impl BodyNow {
    /// Whether it is above the horizon.
    pub fn is_up(&self) -> bool {
        self.direction[1] > 0.0
    }

    /// A moon's phase; `None` for a sun.
    pub fn phase(&self) -> Option<MoonPhase> {
        if self.body.kind == BodyKind::Sun {
            return None;
        }
        Some(if self.lit < NEW_MOON_BELOW {
            MoonPhase::New
        } else if self.lit > 1.0 - NEW_MOON_BELOW {
            MoonPhase::Full
        } else {
            let quarter = (self.lit - 0.5).abs() < QUARTER_WITHIN;
            match (self.waxing, quarter, self.lit < 0.5) {
                (true, true, _) => MoonPhase::FirstQuarter,
                (false, true, _) => MoonPhase::LastQuarter,
                (true, _, true) => MoonPhase::WaxingCrescent,
                (true, _, false) => MoonPhase::WaxingGibbous,
                (false, _, false) => MoonPhase::WaningGibbous,
                (false, _, true) => MoonPhase::WaningCrescent,
            }
        })
    }

    /// The strongest channel of its light on flat ground.
    pub fn flat_light(&self) -> f64 {
        let [red, green, blue] = self.light;
        red.max(green).max(blue) * self.direction[1].max(0.0)
    }
}
