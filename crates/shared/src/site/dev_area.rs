//! Developer tools: a walled area made to order to try things out in (the
//! debug menu's Dev Area). The player arrives on an up staircase whose way out
//! takes them back to wherever they were; the server keeps where that is.

use serde::{Deserialize, Serialize};

use crate::creature::CreatureKind;
use crate::poi::PoiKind;
use crate::rng::Seed;

mod layout;
mod mobs;
pub(super) use layout::Plan;

/// The sides an area may have, in cells inside its walls.
pub const DEV_AREA_SIZES: [u16; 4] = [16, 32, 64, 128];
/// The most mobs of one kind an area may be made with.
pub const MOST_DEV_MOBS: u8 = 32;
/// The kind of place it is drawn as (stone walls).
pub const DEV_AREA_KIND: PoiKind = PoiKind::Castle;

/// What fills an area inside its walls.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum DevLayout {
    /// Open ground.
    #[default]
    Empty,
    /// Deep water up the middle, two cells wide, with a gap at each end: it
    /// can be seen across but not walked.
    Moat,
    /// The same of wall: nothing can be seen through it.
    Cover,
}

impl DevLayout {
    pub const ALL: [Self; 3] = [Self::Empty, Self::Moat, Self::Cover];

    pub fn name(self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Moat => "Moat",
            Self::Cover => "Cover",
        }
    }

    /// The one named so, in any case.
    pub fn named(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|layout| layout.name().eq_ignore_ascii_case(name))
    }
}

/// Who is put in an area.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum DevMobs {
    /// Whoever its layout would have (nobody, for the plain layouts).
    #[default]
    Default,
    None,
    /// `count` of one kind, anywhere at least a little way from the stairs.
    Kind {
        kind: CreatureKind,
        count: u8,
    },
}

/// Everything an area is made from: the same choices always make the same area.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DevArea {
    /// Its side inside the walls, one of `DEV_AREA_SIZES`.
    pub size: u16,
    pub layout: DevLayout,
    pub mobs: DevMobs,
    pub seed: u64,
}

impl DevArea {
    /// The same, kept to what can be made: the nearest size allowed (up to the
    /// largest) and 1 to `MOST_DEV_MOBS` of a kind. Not trusted from a client.
    pub fn checked(self) -> Self {
        let size = DEV_AREA_SIZES
            .into_iter()
            .find(|&size| size >= self.size)
            .unwrap_or(DEV_AREA_SIZES[DEV_AREA_SIZES.len() - 1]);
        let mobs = match self.mobs {
            DevMobs::Kind { kind, count } => DevMobs::Kind {
                kind,
                count: count.clamp(1, MOST_DEV_MOBS),
            },
            other => other,
        };
        Self { size, mobs, ..self }
    }

    /// One number standing for all its choices (for a file's name).
    pub fn key(&self) -> u64 {
        let mobs = match self.mobs {
            DevMobs::Default => 0,
            DevMobs::None => 1,
            DevMobs::Kind { kind, count } => 2 + (kind as i64) * 256 + i64::from(count),
        };
        let choices = [i64::from(self.size), self.layout as i64, mobs];
        Seed::new(self.seed).at(&choices)
    }

    /// The seed for what was typed: a number as it is, any other words hashed.
    pub fn seed_from_text(text: &str) -> u64 {
        let text = text.trim();
        text.parse()
            .unwrap_or_else(|_| Seed::new(0).derive(text).value())
    }
}

#[cfg(test)]
mod tests;
