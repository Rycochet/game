//! How hard the game is for one character: Easy, Normal, Hard or Ironman. Each
//! changes the harm they take, how fast they refill and what dying costs them.
//! A server allows some of them (`Difficulties`); a character keeps theirs.
//! See `docs/context/shared/difficulty.md`.

use serde::{Deserialize, Serialize};

use crate::vitals::{Points, REFILL_TICKS};

mod allowed;

pub use allowed::Difficulties;

/// How hard the game is for one character, easiest first (closeness and ties
/// go by this order).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Difficulty {
    /// A fifth less harm taken, nothing lost on dying, and Health and Mana
    /// refill twice as fast.
    Easy,
    /// As the game is meant to be played.
    #[default]
    Normal,
    /// A fifth more harm taken, and dying drops all but what is worn against
    /// the skin.
    Hard,
    /// A fifth more harm taken, and dying drops everything and deletes the
    /// character. Only chosen when making a character.
    Ironman,
}

impl Difficulty {
    pub const ALL: [Self; 4] = [Self::Easy, Self::Normal, Self::Hard, Self::Ironman];

    /// Its name, as players see it and `HOW_DIFFICULTIES` takes it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
            Self::Ironman => "Ironman",
        }
    }

    /// The difficulty with this name, whatever its case and spaces round it.
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.trim();
        Self::ALL
            .into_iter()
            .find(|difficulty| difficulty.name().eq_ignore_ascii_case(name))
    }

    /// Its place in `ALL`.
    pub(crate) fn index(self) -> usize {
        self as usize
    }

    /// How much of the harm a creature does them they take: a fifth less on
    /// Easy, a fifth more on Hard and Ironman.
    pub fn harm_taken(self, harm: Points) -> Points {
        let (times, by) = match self {
            Self::Easy => (4, 5),
            Self::Normal => (1, 1),
            Self::Hard | Self::Ironman => (6, 5),
        };
        let scaled = u64::from(harm.milli()) * times / by;
        Points(scaled.min(u64::from(u32::MAX)) as u32)
    }

    /// Action ticks for Health and Mana to refill from nothing to full: twice
    /// as fast on Easy.
    pub fn refill_ticks(self) -> u64 {
        match self {
            Self::Easy => REFILL_TICKS / 2,
            _ => REFILL_TICKS,
        }
    }

    /// Whether dying drops what they carry and wear (Hard keeps what is worn
    /// against the skin; Ironman nothing).
    pub fn drops_on_death(self) -> bool {
        matches!(self, Self::Hard | Self::Ironman)
    }

    /// Hours of rust every skill takes on dying (`Skills::rust_for_hours`):
    /// Normal 8, Hard 24, mild enough to be a "doh" rather than a reason to
    /// quit (owner, 2026-11-08). Easy loses none, and Ironman is deleted.
    pub fn rust_on_death(self) -> u32 {
        match self {
            Self::Normal => 8,
            Self::Hard => 24,
            Self::Easy | Self::Ironman => 0,
        }
    }

    /// Whether dying deletes the character.
    pub fn permanent_death(self) -> bool {
        self == Self::Ironman
    }

    /// Whether a player may change from this difficulty to `to` themselves:
    /// any of the others, but never to or from Ironman, which is only chosen
    /// when making a character (a server's rules can still move them).
    pub fn may_change_to(self, to: Self) -> bool {
        self != Self::Ironman && to != Self::Ironman
    }

    /// What an existing character with no difficulty saved starts on: the
    /// player's own default, but not Ironman, which is only chosen when making
    /// a character (Hard instead).
    pub fn for_existing(self) -> Self {
        match self {
            Self::Ironman => Self::Hard,
            other => other,
        }
    }
}

#[cfg(test)]
mod tests;
