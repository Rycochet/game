//! `Points`: an amount of Health or Mana in thousandths of a point, so harm and
//! refilling never lose or gain by rounding. Players only ever see whole points.

use serde::{Deserialize, Serialize};

/// How many thousandths make a point.
pub const PER_POINT: u32 = 1000;

/// An amount of Health or Mana, in thousandths of a point. A newtype (a struct
/// wrapping one value), so thousandths and whole points can't be mixed up by
/// accident: the compiler treats them as different types.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Points(pub u32);

impl Points {
    pub const ZERO: Self = Self(0);

    /// This many whole points.
    pub const fn whole(points: u32) -> Self {
        Self(points.saturating_mul(PER_POINT))
    }

    /// The thousandths.
    pub const fn milli(self) -> u32 {
        self.0
    }

    /// What a player is shown: whole points, rounded up, so anything still
    /// alive never shows 0.
    pub const fn shown(self) -> u32 {
        self.0.div_ceil(PER_POINT)
    }

    /// As a fraction of a point, for drawing.
    pub fn as_f32(self) -> f32 {
        self.0 as f32 / PER_POINT as f32
    }
}
