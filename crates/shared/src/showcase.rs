//! Debug area for looking at every kind of poi: a flat plain, far beyond where
//! anyone would walk, with one poi per chunk. Each kind is a column running east
//! in `PoiKind::ALL`'s order, so new kinds join the east end, and each has
//! `VARIANTS` seeds running south. The plain's weather is the normal weather.
//!
//! The terrain, climate, plant cover, rivers and pois all ask here first, so
//! client and server agree. Turn off `ENABLED` and the land carries on as normal.

use crate::climate::ClimateSample;
use crate::poi::{CHUNK_SIZE, ChunkPos, Poi, PoiKind};
use crate::rng::Seed;

/// Whether the showcase exists.
pub const ENABLED: bool = true;

/// The chunk row of the first pois: far past the poles of any planet, yet near
/// enough for drawing, whose floats lose detail far from the origin.
const FIRST_ROW: i64 = 1000;

/// Everything from here south is plain: room to arrive and look north of the
/// first row.
const PLAIN_FROM: i64 = (FIRST_ROW - 4) * CHUNK_SIZE;

/// How many of each kind, running south.
pub const VARIANTS: i64 = 8;

/// The plain's height: above the highest tide of any world, below the snow.
pub const FLAT_HEIGHT: f64 = 90.0;

/// The plain's climate: grassland everywhere.
pub const CLIMATE: ClimateSample = ClimateSample {
    temperature: 0.55,
    moisture: 0.2,
};

/// Where the key takes you: just north of the first poi.
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
    if !(0..VARIANTS).contains(&row) {
        return None;
    }
    let kind: PoiKind = *PoiKind::ALL.get(usize::try_from(chunk.x).ok()?)?;
    Some(Poi {
        kind,
        chunk,
        x: chunk.x * CHUNK_SIZE + CHUNK_SIZE / 2,
        y: chunk.y * CHUNK_SIZE + CHUNK_SIZE / 2,
        seed: Seed::new(SEEDS.at(&[kind as i64, row])),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::Climate;
    use crate::poi::PoiMap;
    use crate::river::Rivers;
    use crate::surface::Surface;
    use crate::vegetation::Vegetation;
    use crate::world::WorldParams;

    fn world() -> WorldParams {
        WorldParams::from_seed(Seed::new(12345))
    }

    /// Every kind in order running east, with 8 different seeds running south,
    /// and nothing beyond them.
    #[test]
    fn every_kind_has_a_column_of_variants() {
        let pois = PoiMap::new(&world(), Seed::new(1));
        for (column, kind) in PoiKind::ALL.into_iter().enumerate() {
            let mut seeds = std::collections::BTreeSet::new();
            for row in 0..VARIANTS {
                let chunk = ChunkPos {
                    x: column as i64,
                    y: FIRST_ROW + row,
                };
                let poi = pois.in_chunk(chunk).expect("a poi in every chunk");
                assert_eq!((poi.kind, poi.chunk), (kind, chunk));
                assert_eq!(pois.chunk_at(poi.x, poi.y), chunk);
                seeds.insert(poi.seed.value());
            }
            assert_eq!(seeds.len() as i64, VARIANTS);
        }
        let empty = |x, y| pois.in_chunk(ChunkPos { x, y }).is_none();
        assert!(empty(0, FIRST_ROW - 1));
        assert!(empty(0, FIRST_ROW + VARIANTS));
        assert!(empty(PoiKind::ALL.len() as i64, FIRST_ROW));
    }

    #[test]
    fn the_plain_is_flat_dry_and_bare() {
        let world = world();
        let surface = Surface::new(&world);
        let climate = Climate::new(&world);
        let plants = Vegetation::new(&world);
        let rivers = Rivers::new(&world);
        for (x, y) in [ARRIVAL, (500, PLAIN_FROM), (7, PLAIN_FROM + 9000)] {
            let sample = surface.sample(x, y);
            assert_eq!((sample.height, sample.jitter), (FLAT_HEIGHT, 0.0));
            assert_eq!(surface.broad_height(x, y), FLAT_HEIGHT);
            assert!(FLAT_HEIGHT > world.high_water());
            assert_eq!(climate.sample(x, y, FLAT_HEIGHT), CLIMATE);
            assert_eq!(plants.cover(x, y, FLAT_HEIGHT, CLIMATE).trees, 0.0);
        }
        for x in 0..50 {
            let y = FIRST_ROW;
            assert!(rivers.in_chunk(ChunkPos { x, y }).is_none());
        }
    }
}
