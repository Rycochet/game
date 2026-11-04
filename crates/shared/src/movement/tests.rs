use super::*;
use crate::dungeon::Cell;

#[test]
fn only_single_steps_exist() {
    assert!(Step::new(1, -1).is_some());
    for (dx, dy) in [(0, 0), (2, 0), (0, -2), (5, 5)] {
        assert!(Step::new(dx, dy).is_none(), "{dx}, {dy}");
    }
    // Not even when read from a message.
    let long = postcard::to_allocvec(&(2i8, 0i8)).unwrap();
    assert!(postcard::from_bytes::<Step>(&long).is_err());
    let fine = postcard::to_allocvec(&Step::new(-1, 1).unwrap()).unwrap();
    assert_eq!(
        postcard::from_bytes::<Step>(&fine),
        Ok(Step::new(-1, 1).unwrap())
    );
}

#[test]
fn steps_on_a_level_need_walkable_cells() {
    let mut level = Level::test_room();
    let east = Step::new(1, 0).unwrap();
    assert_eq!(step_on_level(&level, (0, 0), east), Some((1, 0)));
    level.set(1, 0, Cell::Wall);
    assert_eq!(step_on_level(&level, (0, 0), east), None);
}

#[test]
fn one_cell_of_liquid_is_jumped() {
    let mut level = Level::test_room();
    let east = Step::new(1, 0).unwrap();
    for liquid in [Cell::Water, Cell::Lava, Cell::Hole] {
        level.set(1, 0, liquid);
        assert_eq!(step_on_level(&level, (0, 0), east), Some((2, 0)));
    }
    // Diagonally too, across a corner.
    let north_west = Step::new(-1, -1).unwrap();
    for liquid in [Cell::Water, Cell::Hole] {
        level.set(3, 1, liquid);
        assert_eq!(step_on_level(&level, (4, 2), north_west), Some((2, 0)));
    }
    // Two wide can't be crossed, so it can't be stepped onto at all.
    level.set(2, 0, Cell::Water);
    assert_eq!(step_on_level(&level, (0, 0), east), None);
    // Nor with a wall to land on.
    level.set(2, 0, Cell::Wall);
    assert_eq!(step_on_level(&level, (0, 0), east), None);
    // Holes the same.
    level.set(2, 0, Cell::Hole);
    assert_eq!(step_on_level(&level, (0, 0), east), None);
}

#[test]
fn steps_cost_the_distance_they_cover() {
    assert_eq!(step_cost((0, 0), (1, 0)), STRAIGHT_COST);
    assert_eq!(step_cost((0, 0), (-1, 1)), DIAGONAL_COST);
    assert_eq!(step_cost((0, 0), (0, -2)), 2 * STRAIGHT_COST);
    assert_eq!(step_cost((0, 0), (2, 2)), 2 * DIAGONAL_COST);
}

/// How high the water is in these tests.
fn water() -> WorldConditions {
    WorldConditions {
        water: WorldParams::from_seed(crate::rng::Seed::new(12345)).high_water(),
        snow_line: 128.0,
    }
}

#[test]
fn the_tide_covers_and_uncovers_ground() {
    let world = WorldParams::from_seed(crate::rng::Seed::new(12345));
    let ground = WorldGround::new(&world);
    let (low, high) = (world.sea_level - 4.0 * world.tilt, world.high_water());
    let at = |water| WorldConditions {
        water,
        snow_line: 128.0,
    };
    let east = Step::new(1, 0).unwrap();
    // Dry ground with ground beside it that only the low tide uncovers.
    let (x, y) = (0..4000)
        .flat_map(|y| (0..4000).map(move |x| (x, y)))
        .find(|&(x, y)| {
            ground.elevation(x, y, at(high)).is_some()
                && ground.elevation(x + 1, y, at(high)).is_none()
                && ground.elevation(x + 1, y, at(low)).is_some()
        })
        .expect("a beach somewhere");
    assert_eq!(step_on_world_map(&ground, at(high), (x, y), east), None);
    assert_eq!(
        step_on_world_map(&ground, at(low), (x, y), east),
        Some((x + 1, y))
    );
}

#[test]
fn ice_can_be_walked_on() {
    let (ground, dry, wet) = shore();
    let east = Step::new(1, 0).unwrap();
    let frozen = WorldConditions {
        snow_line: 0.0,
        ..water()
    };
    assert_eq!(step_on_world_map(&ground, water(), dry, east), None);
    assert_eq!(step_on_world_map(&ground, frozen, dry, east), Some(wet));
}

/// A world's ground, with a dry point and a wet point next to each other.
fn shore() -> (WorldGround, (i64, i64), (i64, i64)) {
    let ground = WorldGround::new(&WorldParams::from_seed(crate::rng::Seed::new(12345)));
    let (y, x) = (0..4000)
        .flat_map(|y| (0..4000).map(move |x| (y, x)))
        .find(|&(y, x)| {
            ground.elevation(x, y, water()).is_some()
                && ground.elevation(x + 1, y, water()).is_none()
        })
        .expect("a shore somewhere");
    (ground, (x, y), (x + 1, y))
}

#[test]
fn water_stops_steps_on_the_world_map_but_not_out_of_it() {
    let (ground, dry, wet) = shore();
    let east = Step::new(1, 0).unwrap();
    let west = Step::new(-1, 0).unwrap();
    assert_eq!(step_on_world_map(&ground, water(), dry, east), None);
    assert_eq!(step_on_world_map(&ground, water(), wet, west), Some(dry));
    assert_eq!(
        step_on_world_map(&ground, water(), wet, east),
        Some((wet.0 + 1, wet.1))
    );
}

#[test]
fn slopes_cost_extra_either_way() {
    let flat = world_step_cost((0, 0), (1, 0), (Some(8), Some(8)), None);
    assert_eq!(flat, STRAIGHT_COST);
    let up = world_step_cost((0, 0), (1, 0), (Some(8), Some(10)), None);
    assert_eq!(up, STRAIGHT_COST + 2 * SLOPE_COST);
    assert_eq!(
        up,
        world_step_cost((1, 0), (0, 0), (Some(10), Some(8)), None)
    );
    assert_eq!(
        world_step_cost((0, 0), (1, 1), (None, Some(50)), None),
        DIAGONAL_COST
    );
}

#[test]
fn steps_between_points_add_up() {
    let steps = steps_between((0, 0), (3, -1));
    assert_eq!(steps.len(), 3);
    let end = steps.iter().fold((0, 0), |at, step| step.from(at));
    assert_eq!(end, (3, -1));
    assert!(steps_between((4, 4), (4, 4)).is_empty());
}

#[test]
fn steps_along_roads_cost_less_by_their_speed() {
    use crate::road::RoadKind;
    let up = |road| world_step_cost((0, 0), (1, 0), (Some(8), Some(10)), road);
    let open = up(None);
    assert_eq!(up(Some(RoadKind::Path)), open * 100 / 125);
    assert_eq!(up(Some(RoadKind::Highway)), open / 2);
    assert!(up(Some(RoadKind::Track)) > up(Some(RoadKind::Road)));
}
