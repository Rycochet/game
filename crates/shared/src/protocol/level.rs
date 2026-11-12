//! Site levels as the server sends them: what a level is, then its cells in
//! 8×8 chunks, only as the player comes to see them (or is about to).

use serde::{Deserialize, Serialize};

use crate::door::DoorLook;
use crate::dungeon::Cell;
use crate::poi::PoiKind;
use crate::site::{StairsDirection, StairsStyle};

/// Level chunks are this many cells on a side: 64 cells, one bit each of a
/// `u64` mask.
pub const LEVEL_CHUNK_SIZE: i64 = 8;

/// How close (each way, in world map grid points) the player must be to a
/// poi to enter it.
pub const ENTER_REACH: i64 = 2;

/// The chunk a cell is in, and its bit in that chunk's mask.
pub fn level_chunk(x: i64, y: i64) -> ((i64, i64), u32) {
    let chunk = (
        x.div_euclid(LEVEL_CHUNK_SIZE),
        y.div_euclid(LEVEL_CHUNK_SIZE),
    );
    let bit = y.rem_euclid(LEVEL_CHUNK_SIZE) * LEVEL_CHUNK_SIZE + x.rem_euclid(LEVEL_CHUNK_SIZE);
    (chunk, bit as u32)
}

/// What kind of place a level is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelKind {
    /// A site behind a world map poi of this kind.
    Poi(PoiKind),
    /// The Hall of Worlds (`shared::hall`): never sent, the client makes it.
    /// Its `x` count from `origin` (`ServerMessage::EnterHall`).
    Hall { origin: i64 },
    /// The wilderness (`shared::wilderness`): its level a window round the
    /// player (`ServerMessage::WildernessWindow`), in its own cells, `x`
    /// wrapping at its width.
    Wilderness,
}

impl LevelKind {
    /// The poi it is behind, if any.
    pub fn poi(self) -> Option<PoiKind> {
        match self {
            Self::Poi(kind) => Some(kind),
            Self::Hall { .. } | Self::Wilderness => None,
        }
    }

    pub fn is_town(self) -> bool {
        self.poi().is_some_and(PoiKind::is_town)
    }
}

/// What a player is told on arriving on a site level: everything except its
/// cells, which follow as `LevelChunk`s.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelInfo {
    /// The kind of place (for its look and name).
    pub kind: LevelKind,
    /// Whether it is a ruined building (for its name).
    pub ruined: bool,
    /// Whether its levels are a building's floors (going up) rather than a
    /// cave's depths.
    pub building: bool,
    /// Which level this is, of how many.
    pub index: u32,
    pub levels: u32,
    /// Whether the site's last level is a basement.
    pub basement: bool,
    /// The level's rectangle: `(min_x, min_y, width, height)`.
    pub bounds: (i64, i64, i64, i64),
    /// Where the player arrives.
    pub arrival: (i64, i64),
    /// Whether they have just come into the site (or back to it, on joining)
    /// rather than taken its stairs: what a client kept of its other levels
    /// is out of date, and `RememberedLevel`s follow.
    pub entering_site: bool,
    /// What the place is called, if it has a name of its own: a town's,
    /// castle's or fort's (`place_name`), worked out by the server, which keeps
    /// the seed. The same on every level, and kept by a ruin.
    pub place_name: Option<String>,
    /// The kind of ground it has (`relief`): what its heights are made from.
    #[serde(default)]
    pub ground: crate::relief::LevelGround,
}

impl LevelInfo {
    /// How high level `index` of this site is: its index, but -1 for a basement.
    pub fn storey(&self, index: u32) -> i32 {
        if self.basement && index + 1 == self.levels {
            -1
        } else {
            index as i32
        }
    }
}

/// What the player remembers of another level of the site they are on, for
/// the maps. Only what they have seen, and the neighbours that shape it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RememberedLevel {
    /// Which level it is.
    pub index: u32,
    /// Its rectangle: `(min_x, min_y, width, height)`.
    pub bounds: (i64, i64, i64, i64),
    pub chunks: Vec<LevelChunk>,
}

/// Some cells of one chunk of the current level.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelChunk {
    /// Which chunk (cell `x, y` is in chunk `x / 8, y / 8`, rounding down).
    pub chunk: (i64, i64),
    /// Which of its cells are included: bit `row × 8 + column`.
    pub mask: u64,
    /// Those cells, in bit order. What the player would see: a secret door
    /// is a wall, unless they have truesight.
    pub cells: Vec<Cell>,
    /// Stairs on those cells.
    pub stairs: Vec<SeenStairs>,
    /// The doors among those cells, as the player sees them now (or saw them
    /// last, if `remembered`). A secret door only once found.
    pub doors: Vec<DoorLook>,
    /// Whether these are cells the player saw on an earlier visit (sent on
    /// arriving), rather than ones they are about to see.
    pub remembered: bool,
}

impl LevelChunk {
    /// The positions of the cells the mask includes, in bit order.
    pub fn positions(&self) -> impl Iterator<Item = (i64, i64)> + '_ {
        let (cx, cy) = self.chunk;
        (0..64i64)
            .filter(|bit| self.mask & (1 << bit) != 0)
            .map(move |bit| {
                (
                    cx * LEVEL_CHUNK_SIZE + bit % LEVEL_CHUNK_SIZE,
                    cy * LEVEL_CHUNK_SIZE + bit / LEVEL_CHUNK_SIZE,
                )
            })
    }

    /// The included cells with their positions, in bit order.
    pub fn cells(&self) -> impl Iterator<Item = (i64, i64, Cell)> + '_ {
        self.positions()
            .zip(&self.cells)
            .map(|((x, y), &cell)| (x, y, cell))
    }
}

/// A staircase as the player sees it: where it leads stays with the server.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenStairs {
    pub x: i64,
    pub y: i64,
    pub direction: StairsDirection,
    /// The way it goes down, as a grid step from its top end to its bottom
    /// end (`Site::stairs_descend`), the same on both levels it joins.
    pub descends: (i64, i64),
    /// What it is built as (`Site::stairs_style`), the same at both ends.
    pub style: StairsStyle,
    /// Its top end is out in the open, so daylight comes down it.
    pub open_to_sky: bool,
}

#[cfg(test)]
mod tests;
