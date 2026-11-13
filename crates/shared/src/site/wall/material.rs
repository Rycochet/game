//! What a wall is made of.

use serde::{Deserialize, Serialize};

use crate::rng::Seed;

/// What a wall is made of, from the roughest to the finest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WallMaterial {
    /// Whole logs standing side by side: a palisade.
    Log,
    /// Boards nailed to posts.
    Plank,
    /// Rough stone.
    Stone,
    /// Dressed stone blocks.
    Block,
}

impl WallMaterial {
    pub const ALL: [Self; 4] = [Self::Log, Self::Plank, Self::Stone, Self::Block];

    /// Whether it is made of wood.
    pub fn is_wood(self) -> bool {
        matches!(self, Self::Log | Self::Plank)
    }

    /// One of `weights` (materials and how likely each is), by a hash of
    /// `seed`: nothing is drawn from any generator's sequence. Stone if there
    /// are none.
    pub fn pick(seed: Seed, weights: &[(Self, u32)]) -> Self {
        let total: u64 = weights.iter().map(|&(_, weight)| u64::from(weight)).sum();
        if total == 0 {
            return Self::Stone;
        }
        let mut roll = seed.derive("wall material").at(&[]) % total;
        for &(material, weight) in weights {
            if roll < u64::from(weight) {
                return material;
            }
            roll -= u64::from(weight);
        }
        Self::Stone
    }
}
