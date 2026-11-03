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

    /// How big it is beside `Medium`, in quarters, to keep to whole numbers.
    pub fn quarters(self) -> u32 {
        match self {
            Self::Tiny => 2,
            Self::Small => 3,
            Self::Medium => 4,
            Self::Large => 5,
            Self::Huge => 6,
            Self::Giant => 8,
        }
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
