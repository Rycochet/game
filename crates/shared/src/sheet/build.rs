//! The three triangles attributes are chosen with: a dot in each, and how far
//! they are turned. Integers only, so client and server always agree. See
//! `docs/context/abilities-skills.md`.

use serde::{Deserialize, Serialize};

use super::Row;
use crate::rng::Rng;

/// What a dot's three weights add up to: a dot at a corner has all of it there.
pub const WEIGHT_TOTAL: u16 = 1000;

/// How many steps the triangles turn through in a whole turn: degrees.
pub const TURN_STEPS: u16 = 360;

/// What every row's points add up to, in thousandths: each row has 1, and the
/// last is shared by the rows above the middle.
pub const POINTS_TOTAL: u32 = 4000;

/// Where a dot sits in its triangle: a weight for each corner, in the row's
/// column order (power, finesse, resistance), adding up to `WEIGHT_TOTAL`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dot(pub [u16; 3]);

impl Dot {
    /// In the middle: an even share. The 1 left over goes to the first corner.
    pub const CENTRE: Self = Self([334, 333, 333]);

    pub fn is_valid(self) -> bool {
        self.0.iter().map(|&weight| weight as u32).sum::<u32>() == WEIGHT_TOTAL as u32
    }
}

/// The three dots (Body, Mind, Spirit) and the turn: everything a player chooses
/// about their attributes, and all the client sends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Build {
    pub dots: [Dot; 3],
    /// Steps of `TURN_STEPS` the triangles are turned by. At 0 Body is at the top.
    pub turn: u16,
}

impl Default for Build {
    fn default() -> Self {
        Self::CENTRED
    }
}

impl Build {
    /// Every dot in the middle, Body at the top.
    pub const CENTRED: Self = Self {
        dots: [Dot::CENTRE; 3],
        turn: 0,
    };

    /// Whether a client's build can be used: every dot inside its triangle and the
    /// turn within one turn.
    pub fn is_valid(&self) -> bool {
        self.dots.iter().all(|dot| dot.is_valid()) && self.turn < TURN_STEPS
    }

    pub fn dot(&self, row: Row) -> Dot {
        self.dots[row as usize]
    }

    /// Each row's points, in thousandths: 1 each, plus a share of the last 1 by
    /// how high the row sits. Rows below the middle get none. Body at the top
    /// gets 2, 1, 1; two rows equally high 1.5, 1.5, 1.
    pub fn points(&self) -> [u32; 3] {
        let heights = Row::ALL.map(|row| {
            let angle = self.turn as u32 + row as u32 * TURN_STEPS as u32 / 3;
            cosine(angle).max(0) as u32
        });
        let high: u32 = heights.iter().sum();
        let mut points = heights.map(|height| 1000 + height * 1000 / high);
        // Rounding may leave a thousandth over: it goes to the highest row.
        let short = POINTS_TOTAL - points.iter().sum::<u32>();
        let highest = (0..3).max_by_key(|&at| heights[at]).unwrap_or(0);
        points[highest] += short;
        points
    }

    /// A build near the centre, for a mob: dots a little off the middle, turned
    /// any way.
    pub fn rolled(rng: &mut Rng) -> Self {
        const OFF: u64 = 100;
        let mut dot = || {
            let raw = [0; 3].map(|_: u16| 333 - OFF + rng.range_u64(0, OFF * 2 + 1));
            let total: u64 = raw.iter().sum();
            let first = (raw[0] * WEIGHT_TOTAL as u64 / total) as u16;
            let second = (raw[1] * WEIGHT_TOTAL as u64 / total) as u16;
            Dot([first, second, WEIGHT_TOTAL - first - second])
        };
        let dots = [dot(), dot(), dot()];
        Self {
            dots,
            turn: rng.range_u64(0, TURN_STEPS as u64) as u16,
        }
    }
}

/// The cosine of a whole number of degrees, in thousandths, from a fixed table
/// rather than floating point, so every machine gets the same.
fn cosine(degrees: u32) -> i32 {
    const QUARTER: [i16; 91] = [
        1000, 1000, 999, 999, 998, 996, 995, 993, 990, 988, 985, 982, 978, 974, 970, 966, 961, 956,
        951, 946, 940, 934, 927, 921, 914, 906, 899, 891, 883, 875, 866, 857, 848, 839, 829, 819,
        809, 799, 788, 777, 766, 755, 743, 731, 719, 707, 695, 682, 669, 656, 643, 629, 616, 602,
        588, 574, 559, 545, 530, 515, 500, 485, 469, 454, 438, 423, 407, 391, 375, 358, 342, 326,
        309, 292, 276, 259, 242, 225, 208, 191, 174, 156, 139, 122, 105, 87, 70, 52, 35, 17, 0,
    ];
    let degrees = degrees % 360;
    let value = match degrees {
        0..=90 => QUARTER[degrees as usize],
        91..=180 => -QUARTER[180 - degrees as usize],
        181..=270 => -QUARTER[degrees as usize - 180],
        _ => QUARTER[360 - degrees as usize],
    };
    value as i32
}

#[cfg(test)]
mod tests;
