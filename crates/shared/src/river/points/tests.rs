use super::*;
use crate::world::{WorldParams, WorldSeed};

#[test]
fn every_river_point_near_is_known_once_covered() {
    let world = WorldParams::from_seed(WorldSeed::new(12345).terrain());
    let rivers = Rivers::new(&world);
    // A river somewhere, and a point along it.
    let river = (0..rivers.columns())
        .find_map(|x| rivers.in_chunk(ChunkPos { x, y: 2 }))
        .expect("a river");
    let point = river.points[river.points.len() / 2];
    let mut points = RiverPoints::default();
    assert!(!points.contains(point));
    assert!(points.cover(&rivers, point));
    assert!(points.contains(point));
    // Round the planet, the same point.
    assert!(points.contains((point.0 + rivers.width(), point.1)));
    // Covering the same place again traces nothing.
    assert!(!points.cover(&rivers, point));
}
