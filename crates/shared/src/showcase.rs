//! Debug area for looking at every kind of poi: a flat plain, far beyond where
//! anyone would walk, with one poi per chunk. Each kind is a column running east
//! in `PoiKind::ALL`'s order, so new kinds join the east end, and each has
//! `VARIANTS` unruined seeds running south and, where it can be ruined, one for
//! each level of ruin running north. The plain's weather is the normal weather.
//!
//! The terrain, climate, plant cover, rivers and pois all ask here first, so
//! client and server agree. Turn off `ENABLED` and the land carries on as normal.

use crate::climate::ClimateSample;
use crate::poi::{CHUNK_SIZE, ChunkPos, Poi, PoiKind};
use crate::rng::Seed;
use crate::site::Generator;

/// Whether the showcase exists.
pub const ENABLED: bool = true;

/// The chunk row of the first pois: far past the poles of any planet, yet near
/// enough for drawing, whose floats lose detail far from the origin.
const FIRST_ROW: i64 = 1000;

/// How many levels of ruin there are, in rows running north of `FIRST_ROW`.
const RUIN_LEVELS: i64 = 5;

/// Seeds tried for each level of ruin: far more than it takes.
const RUIN_TRIES: i64 = 2000;

/// Everything from here south is plain: room to arrive and look north of the
/// northernmost row.
const PLAIN_FROM: i64 = (FIRST_ROW - RUIN_LEVELS - 4) * CHUNK_SIZE;

/// How many of each kind, running south.
pub const VARIANTS: i64 = 8;

/// The plain's height: above the highest tide of any world, below the snow.
pub const FLAT_HEIGHT: f64 = 90.0;

/// The plain's climate: grassland everywhere.
pub const CLIMATE: ClimateSample = ClimateSample {
    temperature: 0.55,
    moisture: 0.2,
};

/// Where the key takes you: in the first column, just north of the unruined
/// rows, which every kind has.
pub const ARRIVAL: (i64, i64) = (CHUNK_SIZE / 2, FIRST_ROW * CHUNK_SIZE - CHUNK_SIZE / 4);

/// For the pois' map seeds.
const SEEDS: Seed = Seed::new(0x706f_6973);

/// Whether grid row `y` is on the plain.
pub fn in_plain(y: i64) -> bool {
    ENABLED && y >= PLAIN_FROM
}

/// Whether a chunk is on the plain.
pub fn chunk_in_plain(chunk: ChunkPos) -> bool {
    in_plain(chunk.y * CHUNK_SIZE)
}

/// The poi of a chunk on the plain, if any, in the middle of it. Its seed
/// depends on where it stands in its own column, not on how many kinds there
/// are, so variants stay as they are when kinds are added.
pub fn poi_at(chunk: ChunkPos) -> Option<Poi> {
    let row = chunk.y - FIRST_ROW;
    let kind: PoiKind = *PoiKind::ALL.get(usize::try_from(chunk.x).ok()?)?;
    if !(-RUIN_LEVELS..VARIANTS).contains(&row) {
        return None;
    }
    let seed = seed_for(kind, row)?;
    Some(Poi {
        kind,
        chunk,
        x: chunk.x * CHUNK_SIZE + CHUNK_SIZE / 2,
        y: chunk.y * CHUNK_SIZE + CHUNK_SIZE / 2,
        seed,
        desert: false,
    })
}

/// The first map seed for this row making a poi of this kind exactly as ruined
/// as the row says: none south of `FIRST_ROW`, counting up to the north. `None`
/// if it can't be, as with what can't be ruined at all. Found rather than forced,
/// so the model and the site agree just as a real poi's do: both go by its seed.
fn seed_for(kind: PoiKind, row: i64) -> Option<Seed> {
    let ruin = (-row).max(0) as u8;
    (0..RUIN_TRIES)
        .map(|n| Seed::new(SEEDS.at(&[kind as i64, row, n])))
        .find(|&seed| Generator::for_site(kind, seed, false).ruin() == ruin)
}

#[cfg(test)]
mod tests;
