//! The Hall of Worlds: a marble walkway hovering in a void, running east and
//! west for billions of tiles and coming round again, with a World Gate every
//! few tiles on both sides. Not made by any generator: everything here is a
//! fixed rule of the place, the same for everyone.
//!
//! Each gate leads to one of the 16 World Gates of one planet, and that pair
//! (the planet's seed and which of its gates) is the gate's address. Places
//! along the Hall and addresses map to each other both ways, so a player
//! coming through any World Gate can be put at the right gate here. Players
//! are shown a gate's place as a code of runes. See
//! `docs/context/shared/hall.md`.

mod address;
mod code;
mod design;
mod level;
mod place;
mod shuffle;
mod tile;

pub use address::GateAddress;
pub use code::{CODE_LENGTH, GateCode, RUNES, SEPARATOR};
pub use design::GateDesign;
pub use level::{LEVEL_ROWS, LEVEL_TOP, cell, level};
pub use place::{GatePlace, Side};
pub use tile::{Tile, hue, tile, wrap_x};

/// The Hall's length east to west, in tiles; east of the last tile is the
/// first again.
pub const HALL_LENGTH: i64 = 1 << 33;
/// How wide the walkway is, north to south: rows `0..WALKWAY_ROWS`.
pub const WALKWAY_ROWS: i64 = 4;
/// Tiles from one gate to the next along each side.
pub const GATE_SPACING: i64 = 4;
/// Bits of a planet's seed in an address: every seed below `1 << PLANET_BITS`
/// is a planet with gates in the Hall.
pub const PLANET_BITS: u32 = 28;
/// Planets reachable from the Hall.
pub const PLANETS: u32 = 1 << PLANET_BITS;

#[cfg(test)]
mod tests;
