//! A heading: a whole-number angle, a full turn in `TURN` steps.

use serde::{Deserialize, Serialize};

/// Steps in a full turn: a heading is a `u16`, so turning wraps by itself.
pub const TURN: u32 = 1 << 16;

/// An eighth of a turn (45°): a tap of Turn Left or Turn Right.
pub const EIGHTH: u16 = (TURN / 8) as u16;

/// Half a turn, in steps.
const HALF: i64 = (TURN / 2) as i64;

/// The length of `Heading::unit`'s vector.
pub const UNIT: i64 = 4096;

/// Which way something faces, on a level's map: 0 is east (`+x`), and it grows
/// towards south (`+y`), as the client's `atan2(y, x)` does. A full turn is
/// `TURN` steps.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Heading(pub u16);

impl Heading {
    pub const EAST: Self = Self(0);
    pub const SOUTH: Self = Self((TURN / 4) as u16);
    pub const WEST: Self = Self((TURN / 2) as u16);
    pub const NORTH: Self = Self((TURN * 3 / 4) as u16);

    /// The way from the origin to `(dx, dy)`; `None` for no way at all.
    pub fn of(dx: i64, dy: i64) -> Option<Self> {
        if dx == 0 && dy == 0 {
            return None;
        }
        // Into the first quarter, then searched for by the sine.
        let (ax, ay) = (dx.unsigned_abs() as i128, dy.unsigned_abs() as i128);
        let (mut low, mut high) = (0i64, HALF / 2);
        while low < high {
            let middle = (low + high + 1) / 2;
            // At or past `middle` while ay / ax ≥ sin / cos there.
            if ay * sin_by(middle + HALF / 2, FINE) >= ax * sin_by(middle, FINE) {
                low = middle;
            } else {
                high = middle - 1;
            }
        }
        let angle = match (dx >= 0, dy >= 0) {
            (true, true) => low,
            (false, true) => HALF - low,
            (false, false) => HALF + low,
            (true, false) => 2 * HALF - low,
        };
        Some(Self(angle.rem_euclid(2 * HALF) as u16))
    }

    /// The way from `from` to `to` (units), if they differ.
    pub fn towards(from: crate::spot::Spot, to: crate::spot::Spot) -> Option<Self> {
        Self::of(to.x - from.x, to.y - from.y)
    }

    /// Turned by `steps` (positive towards south from east: clockwise on a map
    /// with north up).
    pub fn turned(self, steps: i32) -> Self {
        Self(self.0.wrapping_add(steps as u16))
    }

    /// How far apart two headings are, the short way round, in steps (at most
    /// half a turn).
    pub fn apart(self, other: Self) -> u16 {
        let gap = self.0.wrapping_sub(other.0);
        gap.min(gap.wrapping_neg())
    }

    /// A vector of length about `UNIT` pointing this way.
    pub fn unit(self) -> (i64, i64) {
        let angle = i64::from(self.0);
        (cos(angle), sin(angle))
    }

    /// In radians, for drawing (the client); never for rules.
    pub fn radians(self) -> f32 {
        f32::from(self.0) / TURN as f32 * std::f32::consts::TAU
    }

    /// From radians (the client's input), rounded to the nearest step.
    pub fn from_radians(radians: f32) -> Self {
        let turns = radians / std::f32::consts::TAU;
        let steps = (turns * TURN as f32).round() as i64;
        Self(steps.rem_euclid(i64::from(TURN)) as u16)
    }
}

/// The sine of `angle` steps, times `UNIT`: Bhaskara's whole-number formula,
/// within 0.2% everywhere and the same on every machine.
fn sin(angle: i64) -> i64 {
    sin_by(angle, UNIT) as i64
}

fn cos(angle: i64) -> i64 {
    sin(angle + HALF / 2)
}

/// The sine of `angle` steps, times `unit`.
fn sin_by(angle: i64, unit: i64) -> i128 {
    let angle = angle.rem_euclid(2 * HALF);
    let (angle, sign) = if angle > HALF {
        (angle - HALF, -1)
    } else {
        (angle, 1)
    };
    let part = i128::from(angle * (HALF - angle));
    let half = i128::from(HALF);
    sign * 16 * part * i128::from(unit) / (5 * half * half - 4 * part)
}

/// Finer than `UNIT`, for finding a heading.
const FINE: i64 = 1 << 30;
