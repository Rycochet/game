//! How big a creature is, and what that changes.

use serde::{Deserialize, Serialize};

/// How big a creature is, or the clothes made for one, smallest first.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Size {
    Tiny,
    Small,
    #[default]
    Medium,
    Large,
    Huge,
    Giant,
}

impl Size {
    pub const ALL: [Self; 6] = [
        Self::Tiny,
        Self::Small,
        Self::Medium,
        Self::Large,
        Self::Huge,
        Self::Giant,
    ];

    /// What it is called.
    pub fn name(self) -> &'static str {
        match self {
            Self::Tiny => "Tiny",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::Huge => "Huge",
            Self::Giant => "Giant",
        }
    }

    /// How big it is beside `Medium`, in quarters, to keep to whole numbers: the middle
    /// of its band of heights (or lengths) over a person's 1.75 m, from the creature
    /// research. Tiny is under 0.75 m, Small to 1.5 m, Medium to 2.1 m, Large to 3.2 m,
    /// Huge to 4.5 m and Giant to 7 m (owner, 2026-11-04).
    pub fn quarters(self) -> u32 {
        match self {
            Self::Tiny => 1,
            Self::Small => 3,
            Self::Medium => 4,
            Self::Large => 6,
            Self::Huge => 9,
            Self::Giant => 12,
        }
    }

    /// The radius of its footprint on a level, in units (`spot::CELL_SPAN`, 240 a
    /// cell): the circle it is drawn standing on and blocks with. Medium's is
    /// today's token base, 2/3 of a cell across; the rest follow `quarters`:
    /// Tiny 20, Small 60, Medium 80, Large 120, Huge 180, Giant 240 (owner,
    /// 2026-11-10).
    pub fn radius(self) -> i64 {
        self.quarters() as i64 * 20
    }

    /// `by` sizes up or, negative, down, stopping at the ends.
    pub fn stepped(self, by: i8) -> Self {
        let last = Self::ALL.len() as i8 - 1;
        Self::ALL[(self as i8 + by).clamp(0, last) as usize]
    }

    /// How many sizes apart.
    pub fn apart(self, other: Self) -> u8 {
        (self as i8 - other as i8).unsigned_abs()
    }
}
