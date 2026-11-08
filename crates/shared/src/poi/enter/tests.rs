use super::*;
use crate::poi::{ChunkPos, PoiKind};
use crate::rng::Seed;

fn poi_at(x: i64, y: i64) -> Poi {
    Poi {
        kind: PoiKind::Village,
        chunk: ChunkPos { x: 0, y: 0 },
        x,
        y,
        seed: Seed::new(1),
        desert: false,
    }
}

#[test]
fn tokens_overlapping_by_a_fifth_are_near_enough() {
    let poi = poi_at(100, 50);
    assert!(can_enter_from(&poi, (100, 50), 1000));
    assert!(can_enter_from(&poi, (111, 50), 1000));
    assert!(can_enter_from(&poi, (107, 58), 1000), "49 + 64 = 113");
    assert!(!can_enter_from(&poi, (112, 50), 1000));
    assert!(!can_enter_from(&poi, (108, 58), 1000), "64 + 64 = 128");
}

#[test]
fn the_near_way_round_the_planet_counts() {
    let poi = poi_at(2, 50);
    assert!(can_enter_from(&poi, (995, 50), 1000));
    assert!(!can_enter_from(&poi, (990, 50), 1000));
}
