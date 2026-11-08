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
fn only_players_step_onto_world_gates() {
    let mut level = Level::test_room();
    let east = Step::new(1, 0).unwrap();
    level.set(1, 0, Cell::WorldGate);
    assert_eq!(step_on_level(&level, (0, 0), east), Some((1, 0)));
    assert_eq!(mob_step_on_level(&level, (0, 0), east), None);
    assert!(!mob_can_stand(&level, 1, 0));
    // Nor landing on one by a jump.
    level.set(1, 0, Cell::Water);
    level.set(2, 0, Cell::WorldGate);
    assert_eq!(step_on_level(&level, (0, 0), east), Some((2, 0)));
    assert_eq!(mob_step_on_level(&level, (0, 0), east), None);
    // Anywhere else, the same as anyone.
    let south = Step::new(0, 1).unwrap();
    assert_eq!(mob_step_on_level(&level, (0, 0), south), Some((0, 1)));
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
        ice: true,
    }
}

#[test]
fn the_tide_covers_and_uncovers_ground() {
    let world = WorldParams::from_seed(crate::rng::Seed::new(12345));
    let ground = WorldGround::new(&world);
    let (low, high) = (world.sea_level - world.tide_reach(), world.high_water());
    let at = |water| WorldConditions { water, ice: true };
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
fn frozen_sea_can_be_walked_on_but_not_roads_laid_on() {
    let mut world = WorldParams::from_seed(crate::rng::Seed::new(12345));
    world.generator = crate::surface::SurfaceGenerator::Continents;
    let ground = WorldGround::new(&world);
    let water = world.sea_level;
    let walking = WorldConditions { water, ice: true };
    let roads = WorldConditions { water, ice: false };
    // Sea near a pole, frozen over.
    let pole = (world.circumference / 1000) as i64 / 4;
    let frozen = (pole * 7 / 10..pole * 8 / 9)
        .flat_map(|y| (0..200).map(move |x| (x * 50, y)))
        .find(|&(x, y)| {
            ground.elevation(x, y, roads).is_none() && ground.elevation(x, y, walking).is_some()
        });
    assert!(frozen.is_some(), "no frozen sea near the pole");
}

#[test]
fn nobody_walks_past_the_ice() {
    let world = WorldParams::from_seed(crate::rng::Seed::new(12345));
    let ground = WorldGround::new(&world);
    let conditions = WorldConditions {
        water: world.sea_level,
        ice: true,
    };
    // The first row past 80°: 2/9 of the way round, rounded up.
    let edge = ((world.circumference / 1000) as i64 * 2 + 8) / 9;
    assert!(!ground.past_the_ice(edge - 1) && ground.past_the_ice(edge));
    assert!(ground.past_the_ice(-edge));
    assert_eq!(ground.elevation(0, edge, conditions), None);
    // Not even from open water, where any other step is allowed.
    let north = Step::new(0, 1).unwrap();
    let from = (0..5000)
        .map(|x| (x, edge - 1))
        .find(|&(x, y)| ground.elevation(x, y, conditions).is_none());
    if let Some(from) = from {
        assert_eq!(step_on_world_map(&ground, conditions, from, north), None);
    }
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
    let flat = world_step_cost((0, 0), (1, 0), (Some(8), Some(8)), None, Gait::UNSKILLED);
    assert_eq!(flat, STRAIGHT_COST);
    let up = world_step_cost((0, 0), (1, 0), (Some(8), Some(10)), None, Gait::UNSKILLED);
    assert_eq!(up, STRAIGHT_COST + 2 * SLOPE_COST);
    assert_eq!(
        up,
        world_step_cost((1, 0), (0, 0), (Some(10), Some(8)), None, Gait::UNSKILLED)
    );
    assert_eq!(
        world_step_cost((0, 0), (1, 1), (None, Some(50)), None, Gait::UNSKILLED),
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
    let up = |road| world_step_cost((0, 0), (1, 0), (Some(8), Some(10)), road, Gait::UNSKILLED);
    let open = up(None);
    assert_eq!(up(Some(RoadKind::Path)), open * 100 / 125);
    assert_eq!(up(Some(RoadKind::Highway)), open / 2);
    assert!(up(Some(RoadKind::Track)) > up(Some(RoadKind::Road)));
}
