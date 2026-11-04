use super::*;

#[test]
fn woods_are_cleared_beside_a_river_and_back_away_from_it() {
    let mut known = KnownRivers::default();
    known.chunks.insert(
        ChunkPos { x: 0, y: 0 },
        Some(DrawnRiver {
            source: (100, 50),
            points: (0..20).map(|i| Vec2::new(i as f32, 0.0)).collect(),
            widths: vec![1.0; 20],
            temperatures: vec![0.5; 20],
        }),
    );
    assert_eq!(known.open_at(105, 50), 1.0);
    known.clear_woods((90, 40), 100_000, &KnownRoads::default());
    assert_eq!(known.open_at(105, 50), 0.0);
    let part_way = known.open_at(105, 53);
    assert!(part_way > 0.0 && part_way < 1.0, "{part_way}");
    assert_eq!(known.open_at(105, 60), 1.0);
}

#[test]
fn woods_are_cleared_only_round_pois_passed_in() {
    let mut known = KnownRivers {
        cleared_pois: vec![(100, 50)],
        ..default()
    };
    known.clear_woods((90, 40), 100_000, &KnownRoads::default());
    assert_eq!(known.open_at(100, 50), 0.0);
    assert_eq!(known.open_at(100, 70), 1.0);
    // Another character, who doesn't know of it.
    known.cleared_pois.clear();
    known.clear_woods((90, 40), 100_000, &KnownRoads::default());
    assert_eq!(known.open_at(100, 50), 1.0);
}

#[test]
fn woods_are_cleared_wider_round_larger_roads() {
    use shared::poi::ChunkPos;
    use shared::road::{Road, RoadKind, RoadLink};
    let mut roads = KnownRoads::default();
    for (kind, y) in [(RoadKind::Path, 50), (RoadKind::Highway, 80)] {
        roads.insert(Road {
            link: RoadLink::new(ChunkPos { x: 0, y }, ChunkPos { x: 1, y }),
            kind,
            points: (100..130).map(|x| (x, y)).collect(),
        });
    }
    let mut known = KnownRivers::default();
    known.clear_woods((90, 40), 100_000, &roads);
    assert_eq!(known.open_at(115, 50), 0.0);
    assert_eq!(known.open_at(115, 80), 0.0);
    assert!(known.open_at(115, 52) > known.open_at(115, 82));
    assert_eq!(known.open_at(115, 65), 1.0);
}
