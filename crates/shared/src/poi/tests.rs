use super::*;
use crate::rng::Seed;
use crate::surface::Surface;
use crate::world::WorldParams;

// The client's fixed world: 26634 points wide, so 416 columns, the last
// 74 points wide.
fn world() -> WorldParams {
    WorldParams::from_seed(Seed::new(12345))
}

fn seed() -> Seed {
    world().seed.derive("pois")
}

fn map() -> PoiMap {
    PoiMap::new(&world(), seed())
}

// Golden values: if these change, every world's pois move.
#[test]
fn golden_pois() {
    let found: Vec<_> = (-1..=2)
        .map(|x| map().in_chunk(ChunkPos { x, y: -3 }))
        .map(|poi| poi.map(|f| (f.kind, f.x, f.y, f.seed.value())))
        .collect();
    assert_eq!(format!("{found:?}"), GOLDEN_POIS);
}

const GOLDEN_POIS: &str = "[Some((Fort, 29864, -176, 6668956521734191959)), Some((BlueTent, 33, -145, 12428200609095318837)), None, None]";

#[test]
fn about_the_right_share_of_chunks_have_one() {
    // With no sea, so every chunk has land.
    let map = PoiMap::with_water(&world(), seed(), f64::NEG_INFINITY);
    let count = (0..100)
        .flat_map(|y| (0..100).map(move |x| ChunkPos { x, y }))
        .filter(|&chunk| map.in_chunk(chunk).is_some())
        .count();
    assert!((7700..8300).contains(&count), "{count} of 10000");
}

#[test]
fn pois_stand_on_dry_land() {
    let world = world();
    let (map, surface) = (map(), Surface::new(&world));
    let found = map.in_area((-1000, -1000), (1000, 1000));
    let all_chunks = 32 * 32;
    // Some chunks are all sea, but most have land in this area.
    assert!(
        found.len() > all_chunks / 4 && found.len() < all_chunks,
        "{}",
        found.len()
    );
    for poi in found {
        assert!(surface.sample(poi.x, poi.y).height > world.high_water());
    }
}

#[test]
fn pois_stay_inside_their_chunks() {
    let map = map();
    for y in -5..5 {
        // Includes the wider last column and wrapping either side.
        for x in [-2, -1, 0, 1, 200, map.columns() - 1, map.columns()] {
            let Some(poi) = map.in_chunk(ChunkPos { x, y }) else {
                continue;
            };
            assert_eq!(map.chunk_at(poi.x, poi.y), poi.chunk);
            assert!((0..map.width()).contains(&poi.x));
            let inside_x = poi.x - poi.chunk.x * CHUNK_SIZE;
            let inside_y = poi.y - poi.chunk.y * CHUNK_SIZE;
            assert!(inside_x >= EDGE_MARGIN && inside_y >= EDGE_MARGIN);
            assert!(inside_y < CHUNK_SIZE - EDGE_MARGIN);
        }
    }
}

#[test]
fn chunks_in_an_area_wrap_and_cover_it() {
    let map = map();
    let last = map.columns() - 1;
    // Across the seam: the last column, then the first two.
    let chunks = map.chunks_in_area((-10, 0), (100, 63));
    let columns: Vec<i64> = chunks.iter().map(|c| c.x).collect();
    assert_eq!(columns, vec![0, 1, last]);
    assert!(chunks.iter().all(|c| c.y == 0));
    // Every poi found in an area is in one of its chunks.
    let width = map.width();
    let (min, max) = ((width - 300, -200), (width + 300, 200));
    let chunks = map.chunks_in_area(min, max);
    for poi in map.in_area(min, max) {
        assert!(chunks.contains(&poi.chunk));
    }
}

#[test]
fn wraps_round_the_planet() {
    let map = map();
    // An area across the seam at x = 0, seen from both sides.
    let west = map.in_area((-300, -200), (300, 200));
    let width = map.width();
    let east = map.in_area((width - 300, -200), (width + 300, 200));
    assert!(!west.is_empty());
    assert_eq!(west.len(), east.len());
    for (a, b) in west.iter().zip(&east) {
        assert_eq!((a.kind, a.chunk, a.seed), (b.kind, b.chunk, b.seed));
        assert_eq!((a.x + width, a.y), (b.x, b.y));
    }
}

#[test]
fn area_finds_exactly_the_pois_inside() {
    let map = map();
    let (min, max) = ((100, 50), (500, 400));
    let found = map.in_area(min, max);
    let expected = (-1..=10)
        .flat_map(|y| (0..=9).map(move |x| ChunkPos { x, y }))
        .filter_map(|chunk| map.in_chunk(chunk))
        .filter(|f| (min.0..=max.0).contains(&f.x) && (min.1..=max.1).contains(&f.y))
        .count();
    assert_eq!(found.len(), expected);
}

#[test]
fn world_gates_have_their_chunks_to_themselves() {
    let map = map();
    assert!(map.world_gates().complete());
    for gate in &map.world_gates().gates {
        assert_eq!(map.in_chunk(gate.chunk), None, "{gate:?}");
    }
}
