//! Finding a way across one level: the quickest route of legal steps from one
//! point to another (A*), for the player, monsters and spells. See
//! `docs/context/shared/pathfinding.md`.
//!
//! Every move comes from `movement::step_on_level` and is priced by
//! `movement::step_cost`, so a path is one the server would accept and its
//! cost is the time it takes to walk.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::dungeon::Level;
use crate::movement::{
    DIAGONAL_COST, STRAIGHT_COST, Step, WorldConditions, WorldGround, step_cost, step_on_level,
    world_step_cost,
};

/// The order steps are tried in, so equally quick routes always come out the
/// same.
const DIRECTIONS: [(i64, i64); 8] = [
    (0, -1),
    (1, 0),
    (0, 1),
    (-1, 0),
    (1, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
];

/// A route across a level.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Path {
    /// The steps to take, in order.
    pub steps: Vec<Step>,
    /// Where each step lands (two cells on for a jump over liquid). The
    /// starting point isn't included.
    pub cells: Vec<(i64, i64)>,
    /// How long it takes to walk, in `movement::step_cost` units.
    pub cost: u32,
    /// Whether it ends on the goal. If not, it ends on the cell nearest the
    /// goal that could be reached (the start, with no steps, if that is it).
    pub reached: bool,
}

impl Path {
    /// The path through these cells, each a step on from the one before.
    fn through(from: (i64, i64), cells: Vec<(i64, i64)>, reached: bool) -> Self {
        let mut at = from;
        let mut steps = Vec::with_capacity(cells.len());
        let mut cost = 0;
        for &cell in &cells {
            let step = Step::new((cell.0 - at.0).signum(), (cell.1 - at.1).signum())
                .expect("cells of a path differ");
            steps.push(step);
            cost += step_cost(at, cell);
            at = cell;
        }
        Self {
            steps,
            cells,
            cost,
            reached,
        }
    }
}

/// The quickest route from `from` to `to`, using only legal steps that land on
/// cells `allowed` accepts (the client: cells the player knows; a monster: all
/// of them). `max_cost` stops a search going further than that, which also
/// keeps it cheap. If the goal can't be reached, the route goes to the nearest
/// cell that can, straight-line, so check `reached`.
///
/// Among equally quick routes the diagonals fall anywhere: see `straighten`.
pub fn find_path(
    level: &Level,
    from: (i64, i64),
    to: (i64, i64),
    max_cost: Option<u32>,
    allowed: impl Fn(i64, i64) -> bool,
) -> Path {
    let (min_x, min_y, width, height) = level.bounds();
    let index = |(x, y): (i64, i64)| ((y - min_y) * width + (x - min_x)) as usize;
    if !level.contains(from.0, from.1) {
        return Path::default();
    }

    // For each cell: the cheapest way found to it, and the cell it came from.
    let mut best = vec![u32::MAX; (width * height) as usize];
    let mut came_from = vec![from; best.len()];
    best[index(from)] = 0;

    // Cells to look at, cheapest total first. (`BinaryHeap` gives the largest
    // first, and `Reverse` turns that round.) The rest of the tuple settles
    // ties: closest to the goal, then by position.
    let mut open = BinaryHeap::new();
    open.push(Reverse((estimate(from, to), estimate(from, to), from)));
    let mut nearest = (distance_squared(from, to), from);

    while let Some(Reverse((total, left, at))) = open.pop() {
        let cost = total - left;
        // Reached again more cheaply since this was added.
        if cost > best[index(at)] {
            continue;
        }
        nearest = nearest.min((distance_squared(at, to), at));
        if at == to {
            break;
        }
        for (dx, dy) in DIRECTIONS {
            let step = Step::new(dx, dy).expect("directions are steps");
            let Some(next) = step_on_level(level, at, step) else {
                continue;
            };
            let next_cost = cost + step_cost(at, next);
            if next_cost >= best[index(next)]
                || max_cost.is_some_and(|max| next_cost > max)
                || !allowed(next.0, next.1)
            {
                continue;
            }
            best[index(next)] = next_cost;
            came_from[index(next)] = at;
            let left = estimate(next, to);
            open.push(Reverse((next_cost + left, left, next)));
        }
    }

    let end = nearest.1;
    let mut cells = Vec::new();
    let mut at = end;
    while at != from {
        cells.push(at);
        at = came_from[index(at)];
    }
    cells.reverse();
    Path::through(from, cells, end == to)
}

/// How far a route on the world map may go from where it starts, each way, in
/// points. More than the view shows, and the ground is infinite, so this is what
/// keeps the search's cost bounded.
pub const WORLD_ROUTE_REACH: i64 = 320;

/// The quickest route across the world map in these `conditions`: legal
/// steps, slopes costing extra. If the goal can't be reached within
/// `WORLD_ROUTE_REACH`, the route goes to the nearest point that can,
/// straight-line, so check `reached`.
pub fn find_world_path(
    ground: &WorldGround,
    conditions: WorldConditions,
    from: (i64, i64),
    to: (i64, i64),
) -> Path {
    /// Not looked at yet (a height the ground never has, water being `None`).
    const UNKNOWN: Option<i32> = Some(i32::MIN);

    let side = 2 * WORLD_ROUTE_REACH + 1;
    let (min_x, min_y) = (from.0 - WORLD_ROUTE_REACH, from.1 - WORLD_ROUTE_REACH);
    let inside = |(x, y): (i64, i64)| {
        (x - from.0).abs() <= WORLD_ROUTE_REACH && (y - from.1).abs() <= WORLD_ROUTE_REACH
    };
    let index = |(x, y): (i64, i64)| ((y - min_y) * side + (x - min_x)) as usize;

    // Sampling the ground is the slow part, so each point is only sampled once.
    let mut elevations = vec![UNKNOWN; (side * side) as usize];
    let mut elevation = |point: (i64, i64)| {
        let known = &mut elevations[index(point)];
        if *known == UNKNOWN {
            *known = ground.elevation(point.0, point.1, conditions);
        }
        *known
    };

    // As in `find_path`.
    let mut best = vec![u32::MAX; (side * side) as usize];
    let mut came_from = vec![from; best.len()];
    best[index(from)] = 0;
    let mut open = BinaryHeap::new();
    open.push(Reverse((estimate(from, to), estimate(from, to), from)));
    let mut nearest = (distance_squared(from, to), from);

    while let Some(Reverse((total, left, at))) = open.pop() {
        let cost = total - left;
        if cost > best[index(at)] {
            continue;
        }
        nearest = nearest.min((distance_squared(at, to), at));
        if at == to {
            break;
        }
        let here = elevation(at);
        for (dx, dy) in DIRECTIONS {
            let next = (at.0 + dx, at.1 + dy);
            if !inside(next) {
                continue;
            }
            // The rule of `step_on_world_map`, with the samples kept.
            let there = elevation(next);
            if here.is_some() && there.is_none() {
                continue;
            }
            let next_cost = cost + world_step_cost(at, next, (here, there));
            if next_cost >= best[index(next)] {
                continue;
            }
            best[index(next)] = next_cost;
            came_from[index(next)] = at;
            let left = estimate(next, to);
            open.push(Reverse((next_cost + left, left, next)));
        }
    }

    let end = nearest.1;
    let mut cells = Vec::new();
    let mut at = end;
    while at != from {
        cells.push(at);
        at = came_from[index(at)];
    }
    cells.reverse();
    let mut steps = Vec::with_capacity(cells.len());
    let mut at = from;
    for &cell in &cells {
        steps.push(Step::new(cell.0 - at.0, cell.1 - at.1).expect("cells of a path differ"));
        at = cell;
    }
    Path {
        steps,
        cells,
        cost: best[index(end)],
        reached: end == to,
    }
}

/// The same route, as quick and as legal, with its diagonals spread out along
/// each stretch instead of bunched together, which looks like walking in a
/// line. A separate step because only what is drawn cares: anything that
/// recalculates often can skip it. Jumps stay where they are, with each
/// stretch between them straightened alone.
pub fn straighten(
    level: &Level,
    from: (i64, i64),
    path: &Path,
    allowed: impl Fn(i64, i64) -> bool,
) -> Path {
    let mut points = vec![from];
    points.extend_from_slice(&path.cells);

    // The cost of the route up to each point, to compare lines against.
    let mut cost_to = vec![0];
    for pair in points.windows(2) {
        cost_to.push(cost_to[cost_to.len() - 1] + step_cost(pair[0], pair[1]));
    }
    let is_jump = |a: (i64, i64), b: (i64, i64)| (b.0 - a.0).abs() > 1 || (b.1 - a.1).abs() > 1;

    let mut cells = Vec::with_capacity(path.cells.len());
    let mut anchor = 0;
    while anchor + 1 < points.len() {
        // Where this stretch stops: the next jump, or the end.
        let stretch_end = (anchor..points.len() - 1)
            .find(|&i| is_jump(points[i], points[i + 1]))
            .unwrap_or(points.len() - 1);
        if stretch_end == anchor {
            cells.push(points[anchor + 1]);
            anchor += 1;
            continue;
        }
        // The furthest point along it that a line can replace the route to.
        let replaced = (anchor + 2..=stretch_end).rev().find_map(|end| {
            let line = line_cells(points[anchor], points[end]);
            let budget = cost_to[end] - cost_to[anchor];
            line_is_as_good(level, points[anchor], &line, budget, &allowed).then_some((end, line))
        });
        match replaced {
            Some((end, line)) => {
                cells.extend(line);
                anchor = end;
            }
            None => {
                cells.push(points[anchor + 1]);
                anchor += 1;
            }
        }
    }
    Path::through(from, cells, path.reached)
}

/// The fewest-steps line from one point to another, without the first: as many
/// diagonals as it takes, spread evenly among the straight steps.
fn line_cells(from: (i64, i64), to: (i64, i64)) -> Vec<(i64, i64)> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let long = dx.abs().max(dy.abs());
    let short = dx.abs().min(dy.abs());
    (1..=long)
        .map(|k| {
            // `k * short / long`, rounded to nearest, in whole numbers.
            let across = (2 * k * short + long) / (2 * long);
            if dx.abs() >= dy.abs() {
                (from.0 + k * dx.signum(), from.1 + across * dy.signum())
            } else {
                (from.0 + across * dx.signum(), from.1 + k * dy.signum())
            }
        })
        .collect()
}

/// Whether a line can be walked, one cell a step, and costs exactly `budget`.
fn line_is_as_good(
    level: &Level,
    from: (i64, i64),
    line: &[(i64, i64)],
    budget: u32,
    allowed: &impl Fn(i64, i64) -> bool,
) -> bool {
    let mut at = from;
    let mut cost = 0;
    for &cell in line {
        let Some(step) = Step::new(cell.0 - at.0, cell.1 - at.1) else {
            return false;
        };
        if step_on_level(level, at, step) != Some(cell) || !allowed(cell.0, cell.1) {
            return false;
        }
        cost += step_cost(at, cell);
        at = cell;
    }
    cost == budget
}

/// The least a route between two points can cost: what walking it would, with
/// nothing in the way. Never more than the real cost, which A* relies on to
/// find the quickest route. (If terrain faster than floor is ever added, this
/// must assume it.)
fn estimate(from: (i64, i64), to: (i64, i64)) -> u32 {
    let (dx, dy) = ((to.0 - from.0).abs(), (to.1 - from.1).abs());
    let (long, short) = (dx.max(dy) as u32, dx.min(dy) as u32);
    (long - short) * STRAIGHT_COST + short * DIAGONAL_COST
}

fn distance_squared(a: (i64, i64), b: (i64, i64)) -> i64 {
    (a.0 - b.0).pow(2) + (a.1 - b.1).pow(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::Cell;
    use crate::movement::step_on_world_map;
    use crate::rng::Seed;
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
            water: WorldParams::from_seed(Seed::new(12345)).water_level(0.0),
            snow_line: 128.0,
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
            cost += world_step_cost(at, cell, ends);
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
            water: world.water_level(0.0),
            snow_line: 128.0,
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
        let path = find_world_path(
            &ground,
            conditions,
            (peak.0 - 150, peak.1),
            (peak.0 + 150, peak.1),
        );
        assert!(path.reached);
        let highest = path.cells.iter().map(|&p| height(p)).fold(0.0, f64::max);
        assert!(highest < top - 5.0, "{highest} against a summit of {top}");
    }

    #[test]
    fn world_routes_go_round_water() {
        let ground = ground();
        let (from, to) = across_water(&ground);
        let path = find_world_path(&ground, water(), from, to);
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
        let path = find_world_path(&ground, water(), from, wet);
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
        let path = find_world_path(&ground, water(), from, far);
        assert!(!path.reached);
        assert!(
            path.cells
                .iter()
                .all(|c| (c.0 - from.0).abs() <= WORLD_ROUTE_REACH)
        );
        assert_eq!(path, find_world_path(&ground, water(), from, far));
    }

    #[test]
    fn world_routes_start_from_water_too() {
        let ground = ground();
        let (from, to) = across_water(&ground);
        let wet = (from.0 + 20, from.1);
        let path = find_world_path(&ground, water(), wet, to);
        assert_walkable_on_world_map(&ground, wet, &path);
        assert!(!path.cells.is_empty());
    }

    #[test]
    fn nothing_to_straighten_is_fine() {
        let level = empty_room();
        let path = find_path(&level, (3, 3), (3, 3), None, anywhere);
        assert_eq!(straighten(&level, (3, 3), &path, anywhere), path);
    }
}
