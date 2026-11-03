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
    known.clear_woods((90, 40), 100_000);
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
    known.clear_woods((90, 40), 100_000);
    assert_eq!(known.open_at(100, 50), 0.0);
    assert_eq!(known.open_at(100, 70), 1.0);
    // Another character, who doesn't know of it.
    known.cleared_pois.clear();
    known.clear_woods((90, 40), 100_000);
    assert_eq!(known.open_at(100, 50), 1.0);
}
