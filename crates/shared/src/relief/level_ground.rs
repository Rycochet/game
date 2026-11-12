//! Which kind of ground a level has: the one rule both sides follow, worked
//! out by the server (which has the site) and told with the level.

use serde::{Deserialize, Serialize};

use super::{Relief, cave};
use crate::dungeon::Level;
use crate::movement::{FlatGround, WildHeights, WorldGround};
use crate::poi::Poi;
use crate::rng::Seed;
use crate::site::{Site, SiteLevel};
use crate::wilderness::{Cutout, Wilderness};

/// What a cave's swell is hashed from, with the poi's place and the level:
/// nothing secret (a poi's map seed must never reach the client).
const CAVE_SEED: u64 = 0x5eed_cafe_0fca_7e00;

/// The kind of ground a level has (`relief`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelGround {
    /// Level, at height 0: buildings, the Hall, anything else.
    #[default]
    Flat,
    /// The wilderness: the land's heights over the client's window
    /// (`Relief::of_land`), with the flats told as its cells come.
    Wilderness,
    /// A site's first level set into the wilderness whole: the land's heights
    /// where it is set in (a cell plus `offset` is its wilderness cell), with
    /// its cutout's `flats` (in wilderness cells), so it looks the same from
    /// either.
    Land {
        offset: (i64, i64),
        flats: Vec<FlatGround>,
    },
    /// A cave level's gentle swell, from this number.
    Cave { seed: u64 },
}

impl LevelGround {
    /// The ground of level `index` of `poi`'s `site` (`level` being that
    /// level): its first level is the land's when it is set into the
    /// wilderness whole; a cave's other levels swell; the rest is flat.
    pub fn of_site(
        wild: &Wilderness,
        ground: &WorldGround,
        (poi, site): (Poi, &Site),
        index: u32,
        level: &SiteLevel,
    ) -> Self {
        if index == 0 {
            let cutout = Cutout::new(wild, poi, level);
            if let Some(offset) = cutout.offset() {
                let flats = cutout.flats(ground);
                return Self::Land { offset, flats };
            }
        }
        match site.is_cave() {
            true => Self::Cave {
                seed: Seed::new(CAVE_SEED).at(&[poi.x, poi.y, i64::from(index)]),
            },
            false => Self::Flat,
        }
    }

    /// The heights of `level`'s corners, with `stairs`' cells kept level:
    /// `None` when it is flat, or the wilderness, whose window's heights the
    /// client keeps itself (`Relief::of_land` with no offset).
    pub fn relief(
        &self,
        ground: &WorldGround,
        level: &Level,
        stairs: &[(i64, i64)],
    ) -> Option<Relief> {
        match self {
            Self::Flat | Self::Wilderness => None,
            Self::Land { offset, flats } => {
                let (ox, oy) = *offset;
                let (min_x, min_y, width, height) = level.bounds();
                let mut heights = WildHeights::new(ground, (min_x + ox, min_y + oy, width, height));
                heights.relevel_by(flats, |x, y| !level.cell(x - ox, y - oy).is_outside());
                Some(Relief::of_land(&heights, level.bounds(), *offset, stairs))
            }
            Self::Cave { seed } => {
                let mut relief = cave::cave(level, *seed);
                relief.level_stairs(stairs);
                Some(relief)
            }
        }
    }
}
