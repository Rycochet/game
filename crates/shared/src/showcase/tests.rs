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
/// none of them ruined, and nothing beyond them.
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
            assert_eq!(Generator::for_site(kind, poi.seed).ruin(), 0, "{kind:?}");
            seeds.insert(poi.seed.value());
        }
        assert_eq!(seeds.len() as i64, VARIANTS);
    }
    let empty = |x, y| pois.in_chunk(ChunkPos { x, y }).is_none();
    assert!(empty(0, FIRST_ROW - 1));
    assert!(empty(0, FIRST_ROW - RUIN_LEVELS - 1));
    assert!(empty(0, FIRST_ROW + VARIANTS));
    assert!(empty(PoiKind::ALL.len() as i64, FIRST_ROW));
}

/// North of the first row: each level of ruin in turn, counting up, for
/// everything that can be ruined, and nothing for what can't.
#[test]
fn ruins_run_north_counting_up() {
    let pois = PoiMap::new(&world(), Seed::new(1));
    let mut columns = 0;
    for (column, kind) in PoiKind::ALL.into_iter().enumerate() {
        let at = |ruin: i64| {
            pois.in_chunk(ChunkPos {
                x: column as i64,
                y: FIRST_ROW - ruin,
            })
        };
        if at(1).is_none() {
            assert!((1..=RUIN_LEVELS).all(|ruin| at(ruin).is_none()), "{kind:?}");
            continue;
        }
        columns += 1;
        for ruin in 1..=RUIN_LEVELS {
            let poi = at(ruin).expect("every level of ruin");
            assert_eq!(poi.kind, kind);
            let generator = Generator::for_site(kind, poi.seed);
            assert_eq!(i64::from(generator.ruin()), ruin, "{kind:?}");
        }
    }
    // Buildings, farms and huts, without tents and ice shelters.
    assert!(columns > 15, "{columns}");
    assert!(ARRIVAL.1 >= PLAIN_FROM);
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
