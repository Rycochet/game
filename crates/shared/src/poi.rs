//! World map pois: at most one per chunk of the world grid, such as a
//! cave, a house, a castle or a crypt. Each one leads to a map (a dungeon,
//! building or town) generated from the poi's own seed.
//!
//! Everything about a poi comes from its chunk's position alone: whether
//! there is one, what it is, exactly where it stands in the chunk, and the
//! seed its map is generated from. Pois stand only on dry land (above
//! high tide), so a chunk that is all sea has none.

// Split into files (`cargo xtask lint` keeps files small).
mod area;
mod chunk;
mod in_chunk;
mod kind;
mod poi_map;

pub use chunk::{ChunkPos, Poi};
pub use kind::PoiKind;
pub use poi_map::PoiMap;

/// Chunks are this many world grid points (about 1 km each) on a side.
pub const CHUNK_SIZE: i64 = 64;

/// Pois stand at least this far inside their chunk's edges, so two never
/// come closer than twice this (enough for their map tokens not to overlap).
pub const EDGE_MARGIN: i64 = 8;

/// Percentage of chunks that have a poi (where there is land for it).
pub const POI_CHANCE_PERCENT: u64 = 80;

/// Spots tried in a chunk to find dry land before giving up: a chunk that is
/// mostly sea may have no poi.
pub const LAND_TRIES: u32 = 8;

#[cfg(test)]
mod tests;
