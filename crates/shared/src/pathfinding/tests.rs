use super::*;
use crate::dungeon::Cell;
use crate::dungeon::Level;
use crate::movement::step_on_world_map;
use crate::movement::{
    DIAGONAL_COST, Gait, STRAIGHT_COST, Step, WorldConditions, WorldGround, step_cost,
    step_on_level, world_step_cost,
};
use crate::river::RiverPoints;
use crate::rng::Seed;
use crate::road::RoadSteps;
use crate::world::WorldParams;

fn anywhere(_: i64, _: i64) -> bool {
    true
}

/// A room of floor with nothing in it, `(0, 0)` to `(19, 19)`.
fn empty_room() -> Level {
    let mut level = Level::new(-1, -1, 22, 22);
    for y in 0..20 {
        for x in 0..20 {
            level.set(x, y, Cell::Floor);
        }
    }
    level
}

/// Every step of the path is legal, lands where it says, and adds up.
fn assert_walkable(level: &Level, from: (i64, i64), path: &Path) {
    assert_eq!(path.steps.len(), path.cells.len());
    let mut at = from;
    let mut cost = 0;
    for (&step, &cell) in path.steps.iter().zip(&path.cells) {
        assert_eq!(step_on_level(level, at, step), Some(cell), "from {at:?}");
        cost += step_cost(at, cell);
        at = cell;
    }
    assert_eq!(cost, path.cost);
}

#[test]
fn a_straight_line_takes_no_diagonals() {
    let level = empty_room();
    let path = find_path(&level, (2, 5), (12, 5), None, anywhere);
    assert!(path.reached);
    assert_eq!(path.steps, vec![Step::new(1, 0).unwrap(); 10]);
    assert_eq!(path.cost, 10 * STRAIGHT_COST);
}

#[test]
fn the_same_cell_is_no_path_at_all() {
    let path = find_path(&empty_room(), (3, 3), (3, 3), None, anywhere);
    assert!(path.reached);
    assert!(path.steps.is_empty());
}

#[test]
fn it_goes_round_walls() {
    let mut level = empty_room();
    for y in 0..15 {
        level.set(10, y, Cell::Wall);
    }
    let path = find_path(&level, (5, 2), (15, 2), None, anywhere);
    assert!(path.reached);
    assert_walkable(&level, (5, 2), &path);
    assert_eq!(path.cells.last(), Some(&(15, 2)));
    assert!(path.cells.contains(&(10, 15)), "the only way past");
}

#[test]
fn diagonals_are_only_taken_when_they_save_distance() {
    let path = find_path(&empty_room(), (0, 0), (10, 3), None, anywhere);
    let diagonals = path.steps.iter().filter(|s| s.dx() != 0 && s.dy() != 0);
    assert_eq!(diagonals.count(), 3);
    assert_eq!(path.steps.len(), 10);
    assert_eq!(path.cost, 7 * STRAIGHT_COST + 3 * DIAGONAL_COST);
}

#[test]
fn it_jumps_one_cell_of_liquid_or_hole_when_that_is_quicker() {
    for gap in [Cell::Water, Cell::Hole] {
        let mut level = empty_room();
        for y in 0..20 {
            level.set(10, y, gap);
        }
        let path = find_path(&level, (8, 5), (12, 5), None, anywhere);
        assert!(path.reached);
        assert_walkable(&level, (8, 5), &path);
        // The jump counts for the distance it covers.
        assert_eq!(path.cost, 4 * STRAIGHT_COST);
        assert_eq!(path.cells, vec![(9, 5), (11, 5), (12, 5)]);
    }
}

#[test]
fn it_stops_at_the_nearest_cell_when_it_cannot_get_there() {
    let mut level = empty_room();
    // Two wide all the way across: can't be jumped.
    for y in 0..20 {
        level.set(10, y, Cell::Lava);
        level.set(11, y, Cell::Lava);
    }
    let path = find_path(&level, (2, 5), (15, 5), None, anywhere);
    assert!(!path.reached);
    assert_eq!(path.cells.last(), Some(&(9, 5)));
    assert_walkable(&level, (2, 5), &path);
    // Clicking the wall or the liquid itself is the same.
    let path = find_path(&level, (2, 5), (10, 5), None, anywhere);
    assert!(!path.reached);
    assert_eq!(path.cells.last(), Some(&(9, 5)));
}

#[test]
fn it_keeps_to_allowed_cells() {
    let level = empty_room();
    // As if the player knows nothing beyond x = 5.
    let path = find_path(&level, (2, 2), (15, 2), None, |x, _| x <= 5);
    assert!(!path.reached);
    assert_eq!(path.cells.last(), Some(&(5, 2)));
    // The goal itself isn't let off.
    let path = find_path(&level, (2, 2), (6, 2), None, |x, _| x <= 5);
    assert!(!path.reached);
}

#[test]
fn the_limit_stops_the_search() {
    let level = empty_room();
    let path = find_path(&level, (0, 0), (15, 0), Some(5 * STRAIGHT_COST), anywhere);
    assert!(!path.reached);
    assert_eq!(path.cells.last(), Some(&(5, 0)));
    assert_eq!(path.cost, 5 * STRAIGHT_COST);
}

#[test]
fn starting_outside_the_level_goes_nowhere() {
    let path = find_path(&empty_room(), (500, 500), (1, 1), None, anywhere);
    assert_eq!(path, Path::default());
}

#[test]
fn the_same_search_gives_the_same_path() {
    let mut level = empty_room();
    for y in 3..17 {
        level.set(8, y, Cell::Wall);
    }
    let a = find_path(&level, (1, 9), (18, 11), None, anywhere);
    let b = find_path(&level, (1, 9), (18, 11), None, anywhere);
    assert_eq!(a, b);
}

#[test]
fn straightening_spreads_diagonals_out() {
    let level = empty_room();
    let path = find_path(&level, (0, 0), (10, 3), None, anywhere);
    let straight = straighten(&level, (0, 0), &path, anywhere);
    assert_walkable(&level, (0, 0), &straight);
    assert_eq!(straight.cost, path.cost);
    assert!(straight.reached);
    assert_eq!(straight.cells.last(), Some(&(10, 3)));
    // No two diagonals together, and never far off the direct line.
    let diagonal = |s: &Step| s.dx() != 0 && s.dy() != 0;
    for pair in straight.steps.windows(2) {
        assert!(!(diagonal(&pair[0]) && diagonal(&pair[1])), "{straight:?}");
    }
    for (x, y) in &straight.cells {
        assert!((y * 10 - x * 3).abs() <= 10, "{x}, {y} is off the line");
    }
}

#[test]
fn straightening_goes_round_what_is_in_the_way() {
    let mut level = empty_room();
    // Pillars the direct line would cross.
    level.set(5, 2, Cell::Wall);
    level.set(6, 2, Cell::Wall);
    level.set(14, 4, Cell::Wall);
    let path = find_path(&level, (0, 0), (19, 6), None, anywhere);
    let straight = straighten(&level, (0, 0), &path, anywhere);
    assert_walkable(&level, (0, 0), &straight);
    assert_eq!(straight.cost, path.cost);
    assert_eq!(straight.cells.last(), Some(&(19, 6)));
}

#[test]
fn straightening_keeps_detours_and_jumps() {
    let mut level = empty_room();
    for y in 0..15 {
        level.set(10, y, Cell::Wall);
    }
    let detour = find_path(&level, (5, 2), (15, 2), None, anywhere);
    let straight = straighten(&level, (5, 2), &detour, anywhere);
    assert_walkable(&level, (5, 2), &straight);
    assert_eq!(straight.cost, detour.cost);
    assert!(straight.cells.contains(&(10, 15)));

    let mut level = empty_room();
    for y in 0..20 {
        level.set(10, y, Cell::Water);
    }
    let jump = find_path(&level, (2, 2), (18, 8), None, anywhere);
    let straight = straighten(&level, (2, 2), &jump, anywhere);
    assert_walkable(&level, (2, 2), &straight);
    assert_eq!(straight.cost, jump.cost);
    assert_eq!(straight.cells.last(), Some(&(18, 8)));
}

#[test]
fn straightening_keeps_to_allowed_cells() {
    let level = empty_room();
    // A known path along the edge of what is known: lines may not leave it.
    let allowed = |x: i64, y: i64| y == 0 || x == 10;
    let path = find_path(&level, (0, 0), (10, 6), None, allowed);
    assert!(path.reached);
    let straight = straighten(&level, (0, 0), &path, allowed);
    assert!(straight.cells.iter().all(|&(x, y)| allowed(x, y)));
    assert_eq!(straight.cost, path.cost);
}

fn ground() -> WorldGround {
    WorldGround::new(&WorldParams::from_seed(Seed::new(12345)))
}

/// How high the water is in these tests.
fn water() -> WorldConditions {
    WorldConditions {
        water: WorldParams::from_seed(Seed::new(12345)).high_water(),
        ice: true,
    }
}

/// Every step is legal on the world map, lands where it says, and adds up.
fn assert_walkable_on_world_map(ground: &WorldGround, from: (i64, i64), path: &Path) {
    let mut at = from;
    let mut cost = 0;
    for (&step, &cell) in path.steps.iter().zip(&path.cells) {
        assert_eq!(
            step_on_world_map(ground, water(), at, step),
            Some(cell),
            "from {at:?}"
        );
        let ends = (
            ground.elevation(at.0, at.1, water()),
            ground.elevation(cell.0, cell.1, water()),
        );
        cost += world_step_cost(at, cell, ends, None, false, Gait::UNSKILLED);
        at = cell;
    }
    assert_eq!(cost, path.cost);
}

/// A dry point, and water some way east of it with dry ground beyond.
fn across_water(ground: &WorldGround) -> ((i64, i64), (i64, i64)) {
    for y in (0..4000).step_by(16) {
        let dry = |x| ground.elevation(x, y, water()).is_some();
        for x in (0..4000).step_by(4) {
            if dry(x) && !dry(x + 20) && !dry(x + 40) && dry(x + 80) {
                return ((x, y), (x + 80, y));
            }
        }
    }
    panic!("no water to cross");
}

#[test]
fn world_routes_go_round_a_summit() {
    let world = WorldParams::from_seed(crate::world::WorldSeed::new(12345).terrain());
    let surface = crate::surface::Surface::new(&world);
    let ground = WorldGround::new(&world);
    let conditions = WorldConditions {
        water: world.high_water(),
        ice: true,
    };
    // The highest of the broad land round the start.
    let height = |(x, y): (i64, i64)| surface.broad_height(x, y);
    let (mut peak, mut top) = ((0, 0), 0.0);
    for y in (-1500..1500).step_by(16) {
        for x in (-1500..1500).step_by(16) {
            if height((x, y)) > top {
                (peak, top) = ((x, y), height((x, y)));
            }
        }
    }
    let highest = |gait| {
        let path = find_world_path(
            &ground,
            conditions,
            &RoadSteps::default(),
            &RiverPoints::default(),
            (peak.0 - 150, peak.1),
            (peak.0 + 150, peak.1),
            gait,
        );
        assert!(path.reached);
        path.cells.iter().map(|&p| height(p)).fold(0.0, f64::max)
    };
    let walker = highest(Gait::UNSKILLED);
    assert!(walker < top - 5.0, "{walker} against a summit of {top}");
    // A climber at 100% goes much nearer the top.
    let mut skills = crate::skill::Skills::default();
    let climbing = crate::skill::SkillId::find("athletics/climbing").unwrap();
    skills.practise(climbing, 1_000_000);
    let climber = highest(Gait::of(&skills));
    assert!(
        climber > walker + 3.0,
        "climber {climber}, walker {walker}, summit {top}"
    );
}

#[test]
fn world_routes_go_round_water() {
    let ground = ground();
    let (from, to) = across_water(&ground);
    let path = find_world_path(
        &ground,
        water(),
        &RoadSteps::default(),
        &RiverPoints::default(),
        from,
        to,
        Gait::UNSKILLED,
    );
    assert_walkable_on_world_map(&ground, from, &path);
    assert!(
        path.cells
            .iter()
            .all(|&(x, y)| ground.elevation(x, y, water()).is_some())
    );
    if path.reached {
        assert_eq!(path.cells.last(), Some(&to));
        assert!(path.cells.iter().any(|c| c.1 != to.1), "it had to go round");
    }
}

#[test]
fn world_routes_stop_at_the_nearest_shore_for_a_click_on_water() {
    let ground = ground();
    let (from, to) = across_water(&ground);
    let wet = (to.0 - 40, to.1);
    let path = find_world_path(
        &ground,
        water(),
        &RoadSteps::default(),
        &RiverPoints::default(),
        from,
        wet,
        Gait::UNSKILLED,
    );
    assert!(!path.reached);
    assert_walkable_on_world_map(&ground, from, &path);
    let end = path.cells.last().copied().unwrap_or(from);
    assert!(ground.elevation(end.0, end.1, water()).is_some());
}

#[test]
fn world_routes_stay_within_reach_and_repeat_exactly() {
    let ground = ground();
    let (from, _) = across_water(&ground);
    let far = (from.0 + 10 * WORLD_ROUTE_REACH, from.1);
    let path = find_world_path(
        &ground,
        water(),
        &RoadSteps::default(),
        &RiverPoints::default(),
        from,
        far,
        Gait::UNSKILLED,
    );
    assert!(!path.reached);
    assert!(
        path.cells
            .iter()
            .all(|c| (c.0 - from.0).abs() <= WORLD_ROUTE_REACH)
    );
    assert_eq!(
        path,
        find_world_path(
            &ground,
            water(),
            &RoadSteps::default(),
            &RiverPoints::default(),
            from,
            far,
            Gait::UNSKILLED
        )
    );
}

#[test]
fn world_routes_start_from_water_too() {
    let ground = ground();
    let (from, to) = across_water(&ground);
    let wet = (from.0 + 20, from.1);
    let path = find_world_path(
        &ground,
        water(),
        &RoadSteps::default(),
        &RiverPoints::default(),
        wet,
        to,
        Gait::UNSKILLED,
    );
    assert_walkable_on_world_map(&ground, wet, &path);
    assert!(!path.cells.is_empty());
}

#[test]
fn nothing_to_straighten_is_fine() {
    let level = empty_room();
    let path = find_path(&level, (3, 3), (3, 3), None, anywhere);
    assert_eq!(straighten(&level, (3, 3), &path, anywhere), path);
}

#[test]
fn a_distance_map_agrees_with_a_found_path_and_leads_both_ways() {
    let level = Level::test_room();
    let goal = (5, 2);
    let map = DistanceMap::to(&level, goal);
    assert_eq!(map.cost(goal), Some(0));
    for from in [(-6, -2), (0, 3), (-3, 0)] {
        let path = find_path(&level, from, goal, None, |_, _| true);
        assert_eq!(map.cost(from), Some(path.cost), "{from:?}");
        // Going by it reaches the goal.
        let mut at = from;
        while at != goal {
            let step = map.towards(&level, at, |_| true).expect("nearer");
            at = step_on_level(&level, at, step).expect("legal");
        }
        // Going away gets further.
        if let Some(step) = map.away(&level, from, |_| true) {
            let to = step_on_level(&level, from, step).unwrap();
            assert!(map.cost(to) > map.cost(from));
        }
    }
    // A step isn't offered onto a cell that isn't free.
    let blocked = step_on_level(
        &level,
        (0, 3),
        map.towards(&level, (0, 3), |_| true).unwrap(),
    );
    let other = map.towards(&level, (0, 3), |cell| Some(cell) != blocked);
    assert_ne!(
        other.and_then(|step| step_on_level(&level, (0, 3), step)),
        blocked
    );
    // Walls can't be reached.
    assert_eq!(map.cost((-2, 0)), None);
    assert_eq!(map.cost((100, 100)), None);
}

#[test]
fn distance_maps_never_go_through_world_gates() {
    // A wall across the room with a World Gate as the only way through.
    let mut level = Level::test_room();
    for y in -3..=3 {
        level.set(2, y, Cell::Wall);
    }
    level.set(2, 0, Cell::WorldGate);
    let goal = (5, 0);
    // A player could walk through it,
    assert!(find_path(&level, (0, 0), goal, None, |_, _| true).reached);
    // but mobs can't: the far side is out of their reach, and none stands on it.
    let map = DistanceMap::to(&level, goal);
    assert_eq!(map.cost((0, 0)), None);
    assert_eq!(map.cost((2, 0)), None);
    assert_eq!(map.towards(&level, (1, 0), |_| true), None);
}

#[test]
fn routes_go_round_costly_ground_when_that_is_quicker() {
    let level = empty_room();
    // A ridge across x = 10 from the top to y = 14, costly to cross.
    let cost_of = |from: (i64, i64), to: (i64, i64)| {
        let ridge = to.0 == 10 && to.1 < 15;
        step_cost(from, to) + if ridge { 2000 } else { 0 }
    };
    let path = find_path_by(&level, (5, 2), (15, 2), None, anywhere, cost_of, 1000);
    assert!(path.reached);
    assert!(path.cells.iter().all(|&(x, y)| x != 10 || y >= 15));
    let straight = straighten_by(&level, (5, 2), &path, anywhere, cost_of);
    assert_eq!(straight.cost, path.cost, "straightening keeps it as quick");
    assert!(straight.cells.iter().all(|&(x, y)| x != 10 || y >= 15));
    // Plain distance would go straight across.
    let flat = find_path(&level, (5, 2), (15, 2), None, anywhere);
    assert_eq!(flat.cells.len(), 10);
}

#[test]
fn routes_take_ground_quicker_than_walking_when_told_it_can_be() {
    let level = empty_room();
    // A strip along y = 10 a good swimmer crosses at half the cost.
    let cost_of = |from: (i64, i64), to: (i64, i64)| {
        let quick = from.1 == 10 && to.1 == 10;
        if quick {
            step_cost(from, to) / 2
        } else {
            step_cost(from, to)
        }
    };
    let path = find_path_by(&level, (0, 9), (19, 9), None, anywhere, cost_of, 500);
    assert!(path.reached);
    assert!(path.cells.iter().filter(|&&(_, y)| y == 10).count() > 10);
}
