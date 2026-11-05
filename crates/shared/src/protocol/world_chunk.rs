//! World map chunks as sent, with the pois a player has seen.

use serde::{Deserialize, Serialize};

use crate::poi::{ChunkPos, Poi, PoiKind};
use crate::rng::Seed;
use crate::site::PoiLook;
use crate::world::WorldId;

/// What a player knows of one world map chunk. The terrain itself isn't
/// sent: the client generates it from the terrain seed. Later this also
/// carries the chunk's changes and per-player state (visited, attitude).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldChunk {
    pub chunk: ChunkPos,
    /// Its poi, if it has one.
    pub poi: Option<SeenPoi>,
}

/// A world map poi as a player knows it: what and where it is, not
/// the seed its site comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenPoi {
    pub kind: PoiKind,
    pub look: PoiLook,
    pub chunk: ChunkPos,
    pub x: i64,
    pub y: i64,
}

impl From<Poi> for SeenPoi {
    fn from(poi: Poi) -> Self {
        Self {
            kind: poi.kind,
            look: PoiLook::of(poi.kind, poi.seed),
            chunk: poi.chunk,
            x: poi.x,
            y: poi.y,
        }
    }
}

/// What a client is told about a world when it joins.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldInfo {
    pub id: WorldId,
    /// Public: the client generates the world map terrain from it.
    pub terrain_seed: Seed,
    /// Which world map generator the world was made with (`WORLD_MAP_VERSION`).
    pub world_map_version: u32,
    /// Its World Gates, in the order `WorldGates::find` gives (public: they
    /// come from the terrain, and lead to the Hall). Only the Hall's planets
    /// have any: other worlds are reached by other means.
    pub gates: Vec<GateInfo>,
}

/// One of a world's World Gates, on its world map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateInfo {
    /// Which of the world's gates (`hall::GateAddress::entrance`).
    pub entrance: u8,
    pub x: i64,
    pub y: i64,
    /// The Hall gate it leads to (`hall::GatePlace::number`), which looks
    /// just the same: its design and runes.
    pub place: u32,
}
