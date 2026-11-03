//! `Family`: the families of blade, and the kind of item each is.

use serde::{Deserialize, Serialize};

use crate::item::ItemKind;
use crate::rng::Rng;

/// The groups of weapon, each with its generic names and specific ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Family {
    Knife,
    Dagger,
    ShortSword,
    Sword,
    LongSword,
    Axe,
    Spear,
    Polearm,
}

impl Family {
    pub const ALL: [Self; 8] = [
        Self::Knife,
        Self::Dagger,
        Self::ShortSword,
        Self::Sword,
        Self::LongSword,
        Self::Axe,
        Self::Spear,
        Self::Polearm,
    ];

    /// The kind of item this family is made as.
    pub const fn kind(self) -> ItemKind {
        match self {
            Self::Knife => ItemKind::Knife,
            Self::Dagger => ItemKind::Dagger,
            Self::ShortSword => ItemKind::ShortSword,
            Self::Sword => ItemKind::Sword,
            Self::LongSword => ItemKind::Longsword,
            Self::Axe => ItemKind::Axe,
            Self::Spear => ItemKind::Spear,
            Self::Polearm => ItemKind::Polearm,
        }
    }

    /// The other families to try if this one can't be made: first the smaller ones in
    /// any order, as those are likely to fit, then the larger, nearest first.
    pub(super) fn others(self, rng: &mut Rng) -> Vec<Self> {
        let mut smaller: Vec<(Self, u64)> = Self::ALL[..self as usize]
            .iter()
            .map(|&family| (family, 1))
            .collect();
        let mut others = Vec::new();
        while !smaller.is_empty() {
            let family = rng.pick_weighted(&smaller);
            smaller.retain(|&(other, _)| other != family);
            others.push(family);
        }
        others.extend_from_slice(&Self::ALL[self as usize + 1..]);
        others
    }
}
