//! `Difficulties`: which difficulties a server allows.

use serde::{Deserialize, Serialize};

use super::Difficulty;

/// A set of difficulties, as bits by `Difficulty::index`: the ones a server
/// allows. Never empty once made by `parse` or `ALL`; `without` refuses to take
/// the last.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Difficulties(u8);

impl Default for Difficulties {
    fn default() -> Self {
        Self::ALL
    }
}

impl Difficulties {
    /// Every difficulty.
    pub const ALL: Self = Self(0b1111);

    /// Just this one.
    pub fn only(difficulty: Difficulty) -> Self {
        Self(1 << difficulty.index())
    }

    pub fn contains(self, difficulty: Difficulty) -> bool {
        self.0 & (1 << difficulty.index()) != 0
    }

    /// These and `difficulty` too.
    pub fn with(self, difficulty: Difficulty) -> Self {
        Self(self.0 | (1 << difficulty.index()))
    }

    /// These without `difficulty`, unless it is the last: one is always
    /// allowed.
    pub fn without(self, difficulty: Difficulty) -> Self {
        let left = Self(self.0 & !(1 << difficulty.index()));
        if left.is_empty() { self } else { left }
    }

    /// Whether there are none (only from bits read from somewhere: a server
    /// treats that as every one, `or_all`).
    pub fn is_empty(self) -> bool {
        self.0 & Self::ALL.0 == 0
    }

    /// These, or every one if there are none.
    pub fn or_all(self) -> Self {
        if self.is_empty() {
            Self::ALL
        } else {
            Self(self.0 & Self::ALL.0)
        }
    }

    /// The allowed ones, easiest first.
    pub fn iter(self) -> impl Iterator<Item = Difficulty> {
        Difficulty::ALL
            .into_iter()
            .filter(move |&difficulty| self.contains(difficulty))
    }

    /// How many are allowed.
    pub fn len(self) -> usize {
        self.iter().count()
    }

    /// The allowed difficulty nearest `wanted` (itself if allowed), the easier
    /// of two as near.
    pub fn closest(self, wanted: Difficulty) -> Difficulty {
        self.or_all()
            .iter()
            .min_by_key(|difficulty| difficulty.index().abs_diff(wanted.index()))
            .unwrap_or(wanted)
    }

    /// Reads a list of names separated by commas (`HOW_DIFFICULTIES`): blank
    /// means every one. Names it doesn't know are given back, to warn about,
    /// and left out; if none is known, every one is allowed.
    pub fn parse(text: &str) -> (Self, Vec<String>) {
        let mut allowed = Self(0);
        let mut unknown = Vec::new();
        for name in text
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            match Difficulty::from_name(name) {
                Some(difficulty) => allowed = allowed.with(difficulty),
                None => unknown.push(name.to_owned()),
            }
        }
        (allowed.or_all(), unknown)
    }

    /// The names, separated by commas, as `parse` reads them.
    pub fn names(self) -> String {
        self.iter()
            .map(Difficulty::name)
            .collect::<Vec<_>>()
            .join(", ")
    }
}
