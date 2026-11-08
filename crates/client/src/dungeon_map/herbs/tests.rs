use super::*;

#[test]
fn the_list_has_every_plant_once() {
    let found: std::collections::BTreeSet<_> = PLANT_LIST.into_iter().collect();
    assert_eq!(found.len(), PLANT_LIST.len());
    assert!(found.iter().all(|&(c, r)| c < 10 && (2..5).contains(&r)));
}

/// A tile moves on a stage a day, through a plant's four stages in order,
/// then on to the next plant in the list, and all the way round.
#[test]
fn a_tile_moves_on_daily_down_the_list() {
    let at = |hour: u32| tile_growth((5, 9), (1, 2), hour);
    let mut seen = std::collections::BTreeSet::new();
    let mut changes = 0;
    for hour in 0..LIST_HOURS as u32 {
        seen.insert(at(hour));
        let (from, to) = (at(hour), at(hour + 1));
        if from != to {
            changes += 1;
            let next = (from.1 + 1) % STAGES;
            assert_eq!(to.1, next);
            let plant = (from.0 + usize::from(next == 0)) % PLANT_LIST.len();
            assert_eq!(to.0, plant);
        }
    }
    assert_eq!(seen.len(), 30 * 4);
    assert_eq!(changes, 30 * 4);
    // A day later, a tile is always a stage on.
    assert_ne!(at(100), at(124));
}

/// In one patch, tiles differ, and don't all change at the same hour.
#[test]
fn tiles_grow_at_their_own_times() {
    let cells = (0..12).map(|x| (x, 0));
    let looks: std::collections::BTreeSet<_> =
        cells.clone().map(|c| tile_growth((1, 1), c, 500)).collect();
    assert!(looks.len() > 6);
    for hour in 0..24 {
        let moved = cells
            .clone()
            .filter(|&c| tile_growth((1, 1), c, hour) != tile_growth((1, 1), c, hour + 1))
            .count();
        assert!(moved < 12, "all at hour {hour}");
    }
}

#[test]
fn a_shift_is_whole_stages() {
    assert_eq!(growth(5, 2), 53);
}

#[test]
fn stages_draw_nothing_a_flat_surface_or_a_hash() {
    let mut plants = Builder::default();
    tile(&mut plants, Vec3::ZERO, (0, 2), 0);
    assert!(plants.indices.is_empty());
    tile(&mut plants, Vec3::ZERO, (0, 2), 1);
    assert_eq!(plants.indices.len(), 6);
    let mut plants = Builder::default();
    tile(&mut plants, Vec3::ZERO, (0, 2), 3);
    assert_eq!(plants.indices.len(), 4 * 6);
}

#[test]
fn only_tiles_in_sight_are_drawn() {
    let mut level = Level::new(0, 0, 40, 1);
    for x in 0..40 {
        level.set(x, 0, Cell::HerbBed);
    }
    let none = Sight::new(&level);
    assert!(
        build_plants(&level, (&none, &none), (0, 0), 3, &|_, _| 0.0)
            .indices
            .is_empty()
    );
    let mut seen = Sight::new(&level);
    for x in 0..40 {
        seen.insert(x, 0);
    }
    assert!(
        !build_plants(&level, (&seen, &seen), (0, 0), 3, &|_, _| 0.0)
            .indices
            .is_empty()
    );
}
